//! Word prediction engine — v1.
//!
//! Produces ranked word suggestions from:
//! 1. Static dictionary prefix match (primary source)
//! 2. Personal dictionary prefix match (always prioritised)
//! 3. Local usage frequency boost (from `word_usage` table)
//! 4. Fuzzy fallback using Jaro-Winkler (when prefix matches < threshold)
//!
//! All prediction runs in <30 ms. No network. No keystroke logging.

use strsim::jaro_winkler;

use crate::dictionary::{
    DictionaryEntry, PersonalDictionaryRepository, RecentWordsRepository, StaticDictionary,
};

use crate::storage::Storage;

/// A single prediction candidate.
#[derive(Debug, Clone)]
pub struct Prediction {
    pub word: String,
    /// Relevance score in [0.0, 1.0]. Higher is better.
    pub score: f64,
    /// True if the word came from the user's personal dictionary.
    pub from_personal_dict: bool,
}

/// Configuration for the prediction engine.
pub struct PredictionConfig {
    /// Maximum suggestions to return.
    pub max_suggestions: usize,
    /// Minimum Jaro-Winkler score to include a fuzzy candidate.
    pub fuzzy_threshold: f64,
    /// Include personal dictionary in results.
    pub include_personal: bool,
}

impl Default for PredictionConfig {
    fn default() -> Self {
        Self {
            max_suggestions: 5,
            fuzzy_threshold: 0.80,
            include_personal: true,
        }
    }
}

/// The prediction engine.
///
/// Holds a reference to the static dictionary (shared, read-only)
/// and uses SQLite for local usage boosting.
pub struct PredictionEngine<'a> {
    dict: &'a StaticDictionary,
    storage: &'a Storage,
    config: PredictionConfig,
}

impl<'a> PredictionEngine<'a> {
    pub fn new(
        dict: &'a StaticDictionary,
        storage: &'a Storage,
        config: PredictionConfig,
    ) -> Self {
        Self { dict, storage, config }
    }

    /// Predict words for `partial` input.
    ///
    /// Returns candidates sorted by score descending.
    /// Returns an empty `Vec` on empty input or any internal failure.
    pub fn predict(&self, partial: &str) -> Vec<Prediction> {
        if partial.is_empty() {
            return Vec::new();
        }

        let mut candidates: Vec<Prediction> = Vec::new();
        let partial_lower = partial.to_lowercase();

        // ── 1. Personal dictionary (highest priority) ────────────────────────
        if self.config.include_personal {
            let personal = PersonalDictionaryRepository::new(self.storage);
            if let Ok(words) = personal.list_all() {
                for word in words {
                    if word.to_lowercase().starts_with(&partial_lower) {
                        let usage_boost = self.usage_boost(&word);
                        candidates.push(Prediction {
                            score: 0.97 + usage_boost,
                            word,
                            from_personal_dict: true,
                        });
                    }
                }
            }
        }

        // ── 2. Static dictionary prefix match ────────────────────────────────
        let prefix_matches = self
            .dict
            .search_prefix(partial, self.config.max_suggestions * 4);

        for entry in &prefix_matches {
            if candidates.iter().any(|c| c.word == entry.word) {
                continue; // already from personal dict
            }
            let score = self.score(entry, partial) + self.usage_boost(&entry.word);
            candidates.push(Prediction {
                word: entry.word.clone(),
                score: score.min(1.0),
                from_personal_dict: false,
            });
        }

        // ── 3. Fuzzy fallback ────────────────────────────────────────────────
        if candidates.len() < self.config.max_suggestions {
            for entry in self.dict.all() {
                if candidates.iter().any(|c| c.word == entry.word) {
                    continue;
                }
                let sim = jaro_winkler(&entry.word.to_lowercase(), &partial_lower);
                if sim >= self.config.fuzzy_threshold {
                    candidates.push(Prediction {
                        word: entry.word.clone(),
                        score: sim,
                        from_personal_dict: false,
                    });
                }
            }
        }

        // ── 4. Rank and trim ─────────────────────────────────────────────────
        candidates.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        candidates.truncate(self.config.max_suggestions);
        candidates
    }

    /// Score an entry by prefix similarity and static frequency.
    fn score(&self, entry: &DictionaryEntry, partial: &str) -> f64 {
        let sim = jaro_winkler(&entry.word.to_lowercase(), &partial.to_lowercase());
        let freq_bonus = (entry.frequency as f64).ln_1p() * 0.01;
        sim + freq_bonus
    }

    /// Extra score from local word usage (personal tuning).
    fn usage_boost(&self, word: &str) -> f64 {
        let count = RecentWordsRepository::new(self.storage)
            .count(word)
            .unwrap_or(0);
        (count as f64).ln_1p() * 0.005
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Storage;

    fn make_dict(json: &str) -> StaticDictionary {
        StaticDictionary::load_str(json).unwrap()
    }

    #[test]
    fn predicts_prefix_match() {
        let dict = make_dict(
            r#"[{"word":"ngaran","meaning":"name","frequency":100},
                {"word":"ngandam","meaning":"patient","frequency":50}]"#,
        );
        let storage = Storage::open_in_memory().unwrap();
        let engine = PredictionEngine::new(&dict, &storage, PredictionConfig::default());
        let results = engine.predict("ng");
        assert!(!results.is_empty());
        assert!(results.iter().any(|p| p.word == "ngaran"));
    }

    #[test]
    fn empty_input_returns_nothing() {
        let dict = make_dict(r#"[{"word":"aku","frequency":200}]"#);
        let storage = Storage::open_in_memory().unwrap();
        let engine = PredictionEngine::new(&dict, &storage, PredictionConfig::default());
        assert!(engine.predict("").is_empty());
    }

    #[test]
    fn personal_dict_prioritised() {
        let dict = make_dict(r#"[]"#);
        let storage = Storage::open_in_memory().unwrap();
        PersonalDictionaryRepository::new(&storage)
            .add("Hadji")
            .unwrap();
        let engine = PredictionEngine::new(&dict, &storage, PredictionConfig::default());
        let results = engine.predict("Had");
        assert!(results.iter().any(|p| p.word == "Hadji" && p.from_personal_dict));
    }

    #[test]
    fn usage_boost_elevates_known_word() {
        let dict = make_dict(
            r#"[{"word":"aku","frequency":10},{"word":"adlaw","frequency":10}]"#,
        );
        let storage = Storage::open_in_memory().unwrap();
        RecentWordsRepository::new(&storage).record("aku").unwrap();
        RecentWordsRepository::new(&storage).record("aku").unwrap();
        let engine = PredictionEngine::new(&dict, &storage, PredictionConfig::default());
        let results = engine.predict("a");
        // "aku" should rank above "adlaw" due to local usage boost
        let aku_pos = results.iter().position(|p| p.word == "aku").unwrap();
        let adlaw_pos = results.iter().position(|p| p.word == "adlaw").unwrap();
        assert!(aku_pos <= adlaw_pos);
    }
}
