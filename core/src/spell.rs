//! Spell checker for Bahasa Sūg — v1.
//!
//! A word is "correct" if it exists in the static dictionary
//! OR in the user's personal dictionary.
//!
//! For incorrect words, the closest alternatives (by Levenshtein distance)
//! are returned as advisory suggestions.
//!
//! The checker NEVER auto-corrects. The user always decides.

use strsim::levenshtein;

use crate::dictionary::{PersonalDictionaryRepository, StaticDictionary};
use crate::error::CoreResult;
use crate::storage::Storage;

/// Maximum Levenshtein distance to include a suggestion.
const MAX_EDIT_DISTANCE: usize = 2;

/// Maximum suggestions to return.
const MAX_SUGGESTIONS: usize = 5;

/// Result of a spell check for a single word.
#[derive(Debug)]
pub struct SpellCheckResult {
    pub word: String,
    pub is_correct: bool,
    /// Ranked suggestions (only populated when `is_correct` is false).
    pub suggestions: Vec<String>,
}

/// The spell checker.
pub struct SpellChecker<'a> {
    dict: &'a StaticDictionary,
    storage: &'a Storage,
}

impl<'a> SpellChecker<'a> {
    pub fn new(dict: &'a StaticDictionary, storage: &'a Storage) -> Self {
        Self { dict, storage }
    }

    /// Check if `word` is spelled correctly.
    pub fn check(&self, word: &str) -> CoreResult<SpellCheckResult> {
        let in_static = self.dict.find(word).is_some();
        let in_personal = PersonalDictionaryRepository::new(self.storage).contains(word)?;
        let is_correct = in_static || in_personal;

        let suggestions = if is_correct {
            Vec::new()
        } else {
            self.suggest(word)
        };

        Ok(SpellCheckResult {
            word: word.to_string(),
            is_correct,
            suggestions,
        })
    }

    fn suggest(&self, word: &str) -> Vec<String> {
        let word_lower = word.to_lowercase();
        let mut candidates: Vec<(usize, &str)> = self
            .dict
            .all()
            .iter()
            .filter_map(|entry| {
                let dist = levenshtein(&word_lower, &entry.word.to_lowercase());
                if dist <= MAX_EDIT_DISTANCE {
                    Some((dist, entry.word.as_str()))
                } else {
                    None
                }
            })
            .collect();

        candidates.sort_by_key(|(dist, _)| *dist);
        candidates.truncate(MAX_SUGGESTIONS);
        candidates.into_iter().map(|(_, w)| w.to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::{PersonalDictionaryRepository, StaticDictionary};
    use crate::storage::Storage;

    fn make_dict() -> StaticDictionary {
        StaticDictionary::load_str(
            r#"[{"word":"ngaran","meaning":"name","frequency":100},
                {"word":"aku","meaning":"I","frequency":200}]"#,
        )
        .unwrap()
    }

    #[test]
    fn known_word_is_correct() {
        let dict = make_dict();
        let storage = Storage::open_in_memory().unwrap();
        let checker = SpellChecker::new(&dict, &storage);
        let result = checker.check("ngaran").unwrap();
        assert!(result.is_correct);
        assert!(result.suggestions.is_empty());
    }

    #[test]
    fn unknown_word_gives_suggestion() {
        let dict = make_dict();
        let storage = Storage::open_in_memory().unwrap();
        let checker = SpellChecker::new(&dict, &storage);
        let result = checker.check("ngarn").unwrap(); // 1 edit from "ngaran"
        assert!(!result.is_correct);
        assert!(result.suggestions.contains(&"ngaran".to_string()));
    }

    #[test]
    fn personal_dict_word_is_correct() {
        let dict = make_dict();
        let storage = Storage::open_in_memory().unwrap();
        PersonalDictionaryRepository::new(&storage)
            .add("Jolo")
            .unwrap();
        let checker = SpellChecker::new(&dict, &storage);
        let result = checker.check("Jolo").unwrap();
        assert!(result.is_correct);
    }
}
