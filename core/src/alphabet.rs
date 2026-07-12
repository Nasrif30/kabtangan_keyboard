//! Bahasa Sūg alphabet ordering and sorting.
//!
//! The official Bahasa Sūg alphabet is NOT the same as Unicode order.
//! Rules:
//! - The apostrophe (ʼ / glottal stop) is the FIRST letter.
//! - `ng` is treated as a single letter (between `n` and `p`).
//! - `ny` is treated as a single letter (between `ng` and `p`).
//! - Long vowels (ā, ī, ū) are ignored during sorting (treated same as short vowels).
//! - Digraphs: ch, dh, gh, kh, sh, th, zh are each single letters.

/// The canonical alphabet index for Bahasa Sūg sorting.
///
/// Each entry is a grapheme cluster that maps to a sort key (index).
/// Lowercase-normalized before comparison.
pub const ALPHABET: &[&str] = &[
    "'", // 0  — glottal stop (apostrophe, hamzah)
    "a", // 1
    "b", // 2
    "ch", // 3
    "d", // 4
    "dh", // 5
    "g", // 6
    "gh", // 7
    "h", // 8
    "i", // 9
    "j", // 10
    "k", // 11
    "kh", // 12
    "l", // 13
    "m", // 14
    "n", // 15
    "ng", // 16
    "ny", // 17
    "p", // 18
    "r", // 19
    "s", // 20
    "sh", // 21
    "t", // 22
    "th", // 23
    "u", // 24
    "w", // 25
    "y", // 26
    "zh", // 27
];

/// Maps long vowels to their short equivalents for sort normalization.
/// ā → a, ī → i, ū → u
const LONG_VOWEL_MAP: &[(&str, &str)] = &[("ā", "a"), ("ī", "i"), ("ū", "u")];

/// Tokenize a word into alphabet grapheme clusters for sorting.
///
/// Uses greedy longest-match: `ng` is consumed before `n`.
/// Iterates by `char` (not byte) so that multi-byte long vowels (ā ī ū)
/// are correctly normalized before sort-key lookup.
pub fn tokenize(word: &str) -> Vec<&'static str> {
    let lowered = word.to_lowercase();
    let mut chars = lowered.char_indices().peekable();
    let mut result = Vec::new();

    while let Some((i, c)) = chars.next() {
        // Build a 1-char string for the current character
        let char_str = &lowered[i..i + c.len_utf8()];

        // Try 2-char ASCII digraph (only possible when current char is ASCII)
        if c.is_ascii() {
            if let Some(&(j, c2)) = chars.peek() {
                if c2.is_ascii() {
                    let pair = &lowered[i..j + c2.len_utf8()];
                    if let Some(tok) = ALPHABET.iter().find(|&&a| a == pair) {
                        result.push(*tok);
                        chars.next(); // consume second char
                        continue;
                    }
                }
            }
        }

        // Normalize long vowels → short equivalent, then look up sort key
        let normalized = LONG_VOWEL_MAP
            .iter()
            .find(|(long, _)| *long == char_str)
            .map(|(_, short)| *short)
            .unwrap_or(char_str);

        if let Some(tok) = ALPHABET.iter().find(|&&a| a == normalized) {
            result.push(*tok);
        }
        // Unknown characters are skipped in sort key but preserved in the word.
    }

    result
}

/// Returns the sort key for a grapheme (its index in ALPHABET).
/// Returns `usize::MAX` for unknown graphemes (sorts last).
pub fn sort_key(grapheme: &str) -> usize {
    ALPHABET
        .iter()
        .position(|&a| a == grapheme)
        .unwrap_or(usize::MAX)
}

/// Compare two Bahasa Sūg words using the official alphabet order.
///
/// Use this as the comparator in sort operations.
pub fn compare(a: &str, b: &str) -> std::cmp::Ordering {
    let tokens_a = tokenize(a);
    let tokens_b = tokenize(b);

    for (ta, tb) in tokens_a.iter().zip(tokens_b.iter()) {
        let ka = sort_key(ta);
        let kb = sort_key(tb);
        match ka.cmp(&kb) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }

    tokens_a.len().cmp(&tokens_b.len())
}

/// Sort a slice of words in-place using Bahasa Sūg alphabet order.
pub fn sort_words(words: &mut [String]) {
    words.sort_by(|a, b| compare(a.as_str(), b.as_str()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apostrophe_sorts_first() {
        let mut words = vec!["aku".to_string(), "'ampu".to_string(), "batu".to_string()];
        sort_words(&mut words);
        assert_eq!(words[0], "'ampu");
    }

    #[test]
    fn ng_is_single_letter() {
        let tokens = tokenize("ngaran");
        assert_eq!(tokens[0], "ng");
    }

    #[test]
    fn ny_is_single_letter() {
        let tokens = tokenize("nyaan");
        assert_eq!(tokens[0], "ny");
    }

    #[test]
    fn long_vowels_ignored_in_sort() {
        // "aku" and "āku" should compare as equal position
        assert_eq!(compare("aku", "āku"), std::cmp::Ordering::Equal);
    }

    #[test]
    fn digraph_longest_match() {
        // "ng" before "n" in "ngaran"
        let t = tokenize("ngaran");
        assert_eq!(t, vec!["ng", "a", "r", "a", "n"]);
    }

    #[test]
    fn alphabetical_order() {
        let mut words = vec!["batu".to_string(), "aku".to_string(), "chupa".to_string()];
        sort_words(&mut words);
        assert_eq!(words, vec!["aku", "batu", "chupa"]);
    }
}
