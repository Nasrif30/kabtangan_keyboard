//! Transliteration engine: Latin input → Sulat Sūg Unicode codepoints.
//!
//! The engine applies the longest-match rule for digraph detection.
//! All mapping tables are static — zero heap allocation per conversion.
//!
//! # Supported combinations
//!
//! | Latin | Sulat Sūg |
//! |-------|-----------|
//! | ng    | NG char   |
//! | ny    | NY char   |
//! | ch    | CH char   |
//! | dh    | DH char   |
//! | gh    | GH char   |
//! | kh    | KH char   |
//! | sh    | SH char   |
//! | th    | TH char   |
//! | zh    | ZH char   |
//!
//! Note: Actual Sulat Sūg (Batang Sūg script) codepoints will be assigned
//! once the Unicode block is finalized. Placeholder PUA values are used here.
//! Replace `SULAT_*` constants when official codepoints are available.

// ─── Placeholder Unicode Private Use Area mappings ─────────────────────────
// TODO: Replace with official Unicode codepoints when the Batang Sūg block lands.
const SULAT_APOSTROPHE: char = '\u{E000}'; // glottal stop
const SULAT_A: char = '\u{E001}';
const SULAT_B: char = '\u{E002}';
const SULAT_CH: char = '\u{E003}';
const SULAT_D: char = '\u{E004}';
const SULAT_DH: char = '\u{E005}';
const SULAT_G: char = '\u{E006}';
const SULAT_GH: char = '\u{E007}';
const SULAT_H: char = '\u{E008}';
const SULAT_I: char = '\u{E009}';
const SULAT_J: char = '\u{E00A}';
const SULAT_K: char = '\u{E00B}';
const SULAT_KH: char = '\u{E00C}';
const SULAT_L: char = '\u{E00D}';
const SULAT_M: char = '\u{E00E}';
const SULAT_N: char = '\u{E00F}';
const SULAT_NG: char = '\u{E010}';
const SULAT_NY: char = '\u{E011}';
const SULAT_P: char = '\u{E012}';
const SULAT_R: char = '\u{E013}';
const SULAT_S: char = '\u{E014}';
const SULAT_SH: char = '\u{E015}';
const SULAT_T: char = '\u{E016}';
const SULAT_TH: char = '\u{E017}';
const SULAT_U: char = '\u{E018}';
const SULAT_W: char = '\u{E019}';
const SULAT_Y: char = '\u{E01A}';
const SULAT_ZH: char = '\u{E01B}';

// Long vowel diacritics (macron)
const SULAT_AA: char = '\u{E01C}'; // ā
const SULAT_II: char = '\u{E01D}'; // ī
const SULAT_UU: char = '\u{E01E}'; // ū

/// A static mapping entry: Latin string → Sulat Sūg char.
struct Mapping {
    latin: &'static str,
    sulat: char,
}

/// Full mapping table, ordered longest-first to enable greedy matching.
static MAPPING: &[Mapping] = &[
    // Two-character sequences (digraphs) — must come before single chars
    Mapping { latin: "ng", sulat: SULAT_NG },
    Mapping { latin: "ny", sulat: SULAT_NY },
    Mapping { latin: "ch", sulat: SULAT_CH },
    Mapping { latin: "dh", sulat: SULAT_DH },
    Mapping { latin: "gh", sulat: SULAT_GH },
    Mapping { latin: "kh", sulat: SULAT_KH },
    Mapping { latin: "sh", sulat: SULAT_SH },
    Mapping { latin: "th", sulat: SULAT_TH },
    Mapping { latin: "zh", sulat: SULAT_ZH },
    // Long vowels
    Mapping { latin: "aa", sulat: SULAT_AA },
    Mapping { latin: "ii", sulat: SULAT_II },
    Mapping { latin: "uu", sulat: SULAT_UU },
    // Single characters
    Mapping { latin: "'", sulat: SULAT_APOSTROPHE },
    Mapping { latin: "a", sulat: SULAT_A },
    Mapping { latin: "b", sulat: SULAT_B },
    Mapping { latin: "d", sulat: SULAT_D },
    Mapping { latin: "g", sulat: SULAT_G },
    Mapping { latin: "h", sulat: SULAT_H },
    Mapping { latin: "i", sulat: SULAT_I },
    Mapping { latin: "j", sulat: SULAT_J },
    Mapping { latin: "k", sulat: SULAT_K },
    Mapping { latin: "l", sulat: SULAT_L },
    Mapping { latin: "m", sulat: SULAT_M },
    Mapping { latin: "n", sulat: SULAT_N },
    Mapping { latin: "p", sulat: SULAT_P },
    Mapping { latin: "r", sulat: SULAT_R },
    Mapping { latin: "s", sulat: SULAT_S },
    Mapping { latin: "t", sulat: SULAT_T },
    Mapping { latin: "u", sulat: SULAT_U },
    Mapping { latin: "w", sulat: SULAT_W },
    Mapping { latin: "y", sulat: SULAT_Y },
];

/// Convert a Latin Bahasa Sūg string into Sulat Sūg Unicode.
///
/// Uses greedy longest-match: `ng` is consumed as one unit before `n` alone.
///
/// Non-Bahasa-Sūg characters (spaces, punctuation, digits) are passed through unchanged.
///
/// # Example
/// ```
/// use kabtangan_core::transliteration::latin_to_sulat;
/// let result = latin_to_sulat("ngaran");
/// // result contains the Sulat Sūg representation of "ngaran"
/// ```
pub fn latin_to_sulat(input: &str) -> String {
    let lower = input.to_lowercase();
    let chars: &[u8] = lower.as_bytes();
    let mut output = String::with_capacity(input.len());
    let mut i = 0;

    while i < chars.len() {
        // Try each mapping in order (longest first)
        let mut matched = false;
        for m in MAPPING {
            let latin_bytes = m.latin.as_bytes();
            let end = i + latin_bytes.len();
            if end <= chars.len() && &chars[i..end] == latin_bytes {
                output.push(m.sulat);
                i += latin_bytes.len();
                matched = true;
                break;
            }
        }
        if !matched {
            // Pass through unknown character unchanged
            let ch = lower[i..].chars().next().unwrap_or('?');
            output.push(ch);
            i += ch.len_utf8();
        }
    }

    output
}

/// Convert a Sulat Sūg Unicode string back to Latin Bahasa Sūg.
///
/// Useful for display normalization and round-trip testing.
pub fn sulat_to_latin(input: &str) -> String {
    let mut output = String::with_capacity(input.len() * 2);
    for ch in input.chars() {
        let matched = MAPPING.iter().find(|m| m.sulat == ch);
        match matched {
            Some(m) => output.push_str(m.latin),
            None => output.push(ch),
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ng_maps_as_digraph() {
        let result = latin_to_sulat("ng");
        assert_eq!(result.chars().count(), 1);
        assert_eq!(result.chars().next().unwrap(), SULAT_NG);
    }

    #[test]
    fn ny_maps_as_digraph() {
        let result = latin_to_sulat("ny");
        assert_eq!(result.chars().count(), 1);
        assert_eq!(result.chars().next().unwrap(), SULAT_NY);
    }

    #[test]
    fn n_alone_does_not_consume_next() {
        // "na" should map to N + A, not ng + something
        let result = latin_to_sulat("na");
        let chars: Vec<char> = result.chars().collect();
        assert_eq!(chars[0], SULAT_N);
        assert_eq!(chars[1], SULAT_A);
    }

    #[test]
    fn round_trip_latin() {
        let original = "ngaran";
        let sulat = latin_to_sulat(original);
        let back = sulat_to_latin(&sulat);
        assert_eq!(back, original);
    }

    #[test]
    fn passthrough_space() {
        let result = latin_to_sulat("aku ka");
        assert!(result.contains(' '));
    }

    #[test]
    fn full_word_conversion() {
        // "bahasa" should be 6 Sulat chars
        let result = latin_to_sulat("bahasa");
        assert_eq!(result.chars().count(), 6);
    }
}
