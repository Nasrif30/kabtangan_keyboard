//! Grammar suggestion engine for Bahasa Sūg.
//!
//! Provides non-invasive, advisory grammar suggestions based on
//! official Bahasa Sūg writing conventions.
//!
//! Rules:
//! - Suggestions are NEVER applied automatically.
//! - The user always decides whether to accept or reject.
//! - Rules are compiled as a static table — no dynamic loading required.

/// A grammar rule: a pattern and its suggestion.
#[derive(Debug)]
pub struct GrammarRule {
    /// Human-readable rule identifier.
    pub id: &'static str,
    /// Short description of what this rule checks.
    pub description: &'static str,
    /// The function that applies the check.
    check: fn(&str) -> Option<GrammarSuggestion>,
}

/// A suggestion produced by a grammar rule.
#[derive(Debug, Clone)]
pub struct GrammarSuggestion {
    pub rule_id: &'static str,
    pub message: String,
    /// The corrected form, if applicable.
    pub correction: Option<String>,
}

/// All registered grammar rules.
/// Add new rules here; they are applied in order.
static RULES: &[GrammarRule] = &[
    GrammarRule {
        id: "double-space",
        description: "Detects double spaces between words.",
        check: |text| {
            if text.contains("  ") {
                Some(GrammarSuggestion {
                    rule_id: "double-space",
                    message: "Remove extra spaces between words.".to_string(),
                    correction: Some(text.split_whitespace().collect::<Vec<_>>().join(" ")),
                })
            } else {
                None
            }
        },
    },
    GrammarRule {
        id: "sentence-start-capital",
        description: "Sentence should begin with a capital letter.",
        check: |text| {
            let trimmed = text.trim_start();
            let first = trimmed.chars().next();
            match first {
                Some(c) if c.is_alphabetic() && c.is_lowercase() => {
                    let mut chars = trimmed.chars();
                    let corrected = chars
                        .next()
                        .map(|c| c.to_uppercase().to_string())
                        .unwrap_or_default()
                        + chars.as_str();
                    Some(GrammarSuggestion {
                        rule_id: "sentence-start-capital",
                        message: "Sentences should start with a capital letter.".to_string(),
                        correction: Some(corrected),
                    })
                }
                _ => None,
            }
        },
    },
];

/// Analyze a text fragment and return all applicable grammar suggestions.
///
/// Returns an empty list if no issues are found.
/// Never modifies the input — only returns advisory suggestions.
pub fn analyze(text: &str) -> Vec<GrammarSuggestion> {
    RULES
        .iter()
        .filter_map(|rule| (rule.check)(text))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_double_space() {
        let suggestions = analyze("aku  ka");
        assert!(suggestions.iter().any(|s| s.rule_id == "double-space"));
    }

    #[test]
    fn no_issue_clean_text() {
        let suggestions = analyze("Aku ka.");
        assert!(suggestions.is_empty());
    }

    #[test]
    fn lowercase_sentence_start() {
        let suggestions = analyze("aku ka");
        assert!(suggestions
            .iter()
            .any(|s| s.rule_id == "sentence-start-capital"));
    }

    #[test]
    fn correction_provided_for_double_space() {
        let suggestions = analyze("aku  ka");
        let suggestion = suggestions
            .iter()
            .find(|s| s.rule_id == "double-space")
            .unwrap();
        assert_eq!(suggestion.correction.as_deref(), Some("aku ka"));
    }
}
