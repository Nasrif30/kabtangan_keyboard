//! Dictionary module — v1 (offline, static dictionary).
//!
//! Architecture:
//! - Main dictionary: loaded from a bundled static JSON file (read-only).
//! - Personal dictionary: user-added words stored in SQLite (mutable).
//!
//! The static dictionary is loaded into memory once at engine startup and
//! queried directly. No SQLite table is used for the main dictionary.
//! This keeps the schema simple and startup fast.
//!
//! All results are sorted using the official Bahasa Sūg alphabet order.

use serde::{Deserialize, Serialize};
use rusqlite::params;

use crate::alphabet;
use crate::error::CoreResult;
use crate::storage::Storage;

// ─── Data Model ─────────────────────────────────────────────────────────────

/// A single dictionary entry.
///
/// Loaded from the bundled static JSON file at startup.
/// This struct is the canonical representation across all engine modules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryEntry {
    /// The word itself (Bahasa Sūg, Latin script).
    pub word: String,

    /// Sulat Sūg equivalent Unicode string.
    /// Set to `None` until official codepoints are assigned.
    pub sulat: Option<String>,

    /// Bahasa Sūg meaning / definition.
    pub meaning: Option<String>,

    /// Pronunciation guide (phonemic or IPA).
    pub pronunciation: Option<String>,

    /// An example sentence using this word in context.
    pub example: Option<String>,

    /// Root word for derived forms.
    pub root_word: Option<String>,

    /// Grammatical category (noun, verb, adjective, etc.).
    pub category: Option<String>,

    /// Pre-computed frequency for prediction ranking.
    /// Higher = more common. Never modified at runtime.
    pub frequency: u32,
}

// ─── Static Dictionary ───────────────────────────────────────────────────────

/// In-memory representation of the bundled static dictionary.
///
/// Loaded once at engine startup from the bundled JSON file.
/// Queried by prefix or exact match. Never written to.
pub struct StaticDictionary {
    /// All entries in Bahasa Sūg alphabet order.
    entries: Vec<DictionaryEntry>,
}

impl StaticDictionary {
    /// Load the dictionary from a JSON byte slice (bundled via `include_bytes!`).
    ///
    /// Parses the JSON and sorts all entries in official alphabet order.
    pub fn load(json_bytes: &[u8]) -> CoreResult<Self> {
        let mut entries: Vec<DictionaryEntry> = serde_json::from_slice(json_bytes)?;
        entries.sort_by(|a, b| alphabet::compare(&a.word, &b.word));
        Ok(Self { entries })
    }

    /// Load from a JSON string (convenience for tests).
    pub fn load_str(json: &str) -> CoreResult<Self> {
        Self::load(json.as_bytes())
    }

    /// Exact lookup by word (case-insensitive).
    pub fn find(&self, word: &str) -> Option<&DictionaryEntry> {
        let lower = word.to_lowercase();
        self.entries
            .iter()
            .find(|e| e.word.to_lowercase() == lower)
    }

    /// Prefix search — returns entries whose word starts with `prefix`.
    /// Results are in Bahasa Sūg alphabet order, limited to `limit`.
    pub fn search_prefix(&self, prefix: &str, limit: usize) -> Vec<&DictionaryEntry> {
        let lower = prefix.to_lowercase();
        self.entries
            .iter()
            .filter(|e| e.word.to_lowercase().starts_with(&lower))
            .take(limit)
            .collect()
    }

    /// All entries (already sorted).
    pub fn all(&self) -> &[DictionaryEntry] {
        &self.entries
    }

    /// Total number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// ─── Personal Dictionary ─────────────────────────────────────────────────────

/// Repository for user-added personal dictionary words.
///
/// Stored in SQLite (`personal_dictionary` table).
/// Users can add names, local places, abbreviations, and frequently used words.
pub struct PersonalDictionaryRepository<'a> {
    storage: &'a Storage,
}

impl<'a> PersonalDictionaryRepository<'a> {
    pub fn new(storage: &'a Storage) -> Self {
        Self { storage }
    }

    /// Add a word to the personal dictionary.
    /// Silently ignored if the word already exists.
    pub fn add(&self, word: &str) -> CoreResult<()> {
        self.storage.connection().execute(
            "INSERT OR IGNORE INTO personal_dictionary (word) VALUES (?1)",
            params![word],
        )?;
        Ok(())
    }

    /// Remove a word from the personal dictionary.
    pub fn remove(&self, word: &str) -> CoreResult<()> {
        self.storage.connection().execute(
            "DELETE FROM personal_dictionary WHERE word = ?1 COLLATE NOCASE",
            params![word],
        )?;
        Ok(())
    }

    /// Returns true if the word is in the personal dictionary.
    pub fn contains(&self, word: &str) -> CoreResult<bool> {
        let count: i64 = self.storage.connection().query_row(
            "SELECT COUNT(*) FROM personal_dictionary WHERE word = ?1 COLLATE NOCASE",
            params![word],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// List all personal dictionary words in Bahasa Sūg alphabet order.
    pub fn list_all(&self) -> CoreResult<Vec<String>> {
        let conn = self.storage.connection();
        let mut stmt = conn.prepare("SELECT word FROM personal_dictionary")?;
        let mut words: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        alphabet::sort_words(&mut words);
        Ok(words)
    }
}

// ─── Recent Words ─────────────────────────────────────────────────────────────

/// Tracks recently typed words to improve prediction ranking.
///
/// Stored in SQLite (`word_usage` table).
/// This is local usage data only — never sent anywhere.
pub struct RecentWordsRepository<'a> {
    storage: &'a Storage,
}

impl<'a> RecentWordsRepository<'a> {
    pub fn new(storage: &'a Storage) -> Self {
        Self { storage }
    }

    /// Record that `word` was typed. Increments its local usage count.
    pub fn record(&self, word: &str) -> CoreResult<()> {
        self.storage.connection().execute(
            "INSERT INTO word_usage (word, count, last_used)
             VALUES (?1, 1, datetime('now'))
             ON CONFLICT(word) DO UPDATE SET
               count    = count + 1,
               last_used = datetime('now')",
            params![word],
        )?;
        Ok(())
    }

    /// Get the local usage count for `word`.
    pub fn count(&self, word: &str) -> CoreResult<u32> {
        let count: u32 = self
            .storage
            .connection()
            .query_row(
                "SELECT count FROM word_usage WHERE word = ?1 COLLATE NOCASE",
                params![word],
                |row| row.get(0),
            )
            .unwrap_or(0);
        Ok(count)
    }

    /// Return the N most recently used words.
    pub fn recent(&self, limit: usize) -> CoreResult<Vec<String>> {
        let conn = self.storage.connection();
        let mut stmt = conn.prepare(
            "SELECT word FROM word_usage ORDER BY last_used DESC LIMIT ?1",
        )?;
        let words: Vec<String> = stmt
            .query_map(params![limit as i64], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(words)
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Storage;

    fn sample_json() -> &'static str {
        r#"[
            {"word":"ngaran","meaning":"name","frequency":100},
            {"word":"aku","meaning":"I / me","frequency":200},
            {"word":"batu","meaning":"stone","frequency":50}
        ]"#
    }

    #[test]
    fn loads_and_sorts_alphabetically() {
        let dict = StaticDictionary::load_str(sample_json()).unwrap();
        assert_eq!(dict.all()[0].word, "aku");   // a before b before ng
        assert_eq!(dict.all()[1].word, "batu");
        assert_eq!(dict.all()[2].word, "ngaran");
    }

    #[test]
    fn exact_lookup() {
        let dict = StaticDictionary::load_str(sample_json()).unwrap();
        assert!(dict.find("ngaran").is_some());
        assert!(dict.find("NGARAN").is_some()); // case-insensitive
        assert!(dict.find("nonexistent").is_none());
    }

    #[test]
    fn prefix_search() {
        let dict = StaticDictionary::load_str(sample_json()).unwrap();
        let results = dict.search_prefix("ng", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].word, "ngaran");
    }

    #[test]
    fn personal_dict_add_contains_remove() {
        let storage = Storage::open_in_memory().unwrap();
        let repo = PersonalDictionaryRepository::new(&storage);
        repo.add("Jolo").unwrap();
        assert!(repo.contains("Jolo").unwrap());
        repo.remove("Jolo").unwrap();
        assert!(!repo.contains("Jolo").unwrap());
    }

    #[test]
    fn recent_words_record_and_count() {
        let storage = Storage::open_in_memory().unwrap();
        let repo = RecentWordsRepository::new(&storage);
        repo.record("aku").unwrap();
        repo.record("aku").unwrap();
        assert_eq!(repo.count("aku").unwrap(), 2);
    }
}
