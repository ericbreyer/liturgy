//! Title classification logic for determining liturgical commons.
//!
//! This module extracts the fragile substring matching into a configurable,
//! order-dependent classification system that maps feast titles to common categories.

/// Represents a pattern that matches against saint titles to determine commons.
#[derive(Debug, Clone)]
struct Pattern {
    /// The substring to match (case-insensitive)
    substring: String,
    /// The common category this pattern indicates (e.g., "Virgins", "Martyrs")
    common: String,
    /// Priority/precedence; higher numbers take precedence
    priority: usize,
}

/// Classifies feast titles into liturgical commons (Virgins, Martyrs, Confessors, etc.)
#[derive(Debug)]
pub struct TitleClassifier {
    /// Patterns in precedence order (highest priority first)
    patterns: Vec<Pattern>,
}

impl Default for TitleClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl TitleClassifier {
    /// Creates a new classifier with built-in patterns.
    ///
    /// Patterns are ordered by priority. Higher-priority patterns are checked first.
    /// This handles cases like "virgin-martyr" (virgin before martyr to disambiguate).
    pub fn new() -> Self {
        // Patterns are listed in priority order (highest first).
        // This explicitly handles tricky cases like virgin-martyr.
        let patterns = vec![
            // Special case: Blessed Virgin Mary must be checked first, overrides all other titles
            Pattern {
                substring: "blessed virgin mary".to_string(),
                common: "the Blessed Virgin Mary".to_string(),
                priority: 1000,
            },
            // Composite patterns: check these before single patterns
            // Virgin-martyr: prefer "Virgins" over "Martyrs" (liturgical tradition)
            Pattern {
                substring: "virgin".to_string(),
                common: "Virgins".to_string(),
                priority: 500,
            },
            // Confessor Bishops: check before general Confessors
            Pattern {
                substring: "bishop".to_string(),
                common: "Confessor Bishops".to_string(),
                priority: 400,
            },
            Pattern {
                substring: "pope".to_string(),
                common: "Confessor Bishops".to_string(),
                priority: 400,
            },
            // Martyrs (but not if bishop was already matched)
            Pattern {
                substring: "martyr".to_string(),
                common: "Martyrs".to_string(),
                priority: 300,
            },
            // General Confessors (abbot aliased to confessor)
            Pattern {
                substring: "confessor".to_string(),
                common: "Confessors (Non-Bishop)".to_string(),
                priority: 200,
            },
            Pattern {
                substring: "abbot".to_string(),
                common: "Confessors (Non-Bishop)".to_string(),
                priority: 200,
            },
            // Apostles
            Pattern {
                substring: "apostle".to_string(),
                common: "Apostles".to_string(),
                priority: 100,
            },
        ];

        Self { patterns }
    }

    /// Classify a day based on its titles and description.
    ///
    /// Returns the calculated common, or None if no pattern matched.
    /// The algorithm:
    /// 1. Check if "Blessed Virgin Mary" is in the description (highest priority)
    /// 2. Check if "bishop" or "pope" is in titles (Confessor Bishops)
    /// 3. Check if "virgin" is in titles (Virgins)
    /// 4. Check if "martyr" is in titles AND "bishop" was not matched (Martyrs)
    /// 5. Check if "confessor" or "abbot" is in titles (Confessors)
    /// 6. Check if "apostle" is in titles (Apostles)
    ///
    /// The key insight: we maintain a set of matched attributes and use them to
    /// disambiguate. For example, if both "virgin" and "martyr" match, we return
    /// "Virgins" (not "Martyrs") because virgin has higher priority.
    pub fn classify<T: AsRef<str>>(
        &self,
        titles: &[T],
        description: &str,
        day_desc: &str,
    ) -> Option<String> {
        let desc_lower = day_desc.to_lowercase();
        let titles_lower: Vec<String> = titles.iter().map(|t| t.as_ref().to_lowercase()).collect();
        let all_text = format!("{} {} {}", desc_lower, titles_lower.join(" "), description);

        // Collect all matched titles with their priorities
        let mut matches: Vec<(String, usize)> = Vec::new();

        for pattern in &self.patterns {
            if all_text.contains(&pattern.substring) {
                matches.push((pattern.common.clone(), pattern.priority));
            }
        }

        // Return highest priority match
        matches.sort_by(|a, b| b.1.cmp(&a.1));
        matches.first().map(|(common, _)| common.clone())
    }

    /// Classify with just titles and day description (backward compatibility).
    pub fn classify_simple<T: AsRef<str>>(&self, titles: &[T], day_desc: &str) -> Option<String> {
        self.classify(titles, "", day_desc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blessed_virgin_mary() {
        let classifier = TitleClassifier::new();
        let result = classifier.classify_simple(
            &["Virgin".to_string()],
            "Feast of the Blessed Virgin Mary",
        );
        assert_eq!(result, Some("the Blessed Virgin Mary".to_string()));
    }

    #[test]
    fn test_virgin_martyr_prioritizes_virgin() {
        let classifier = TitleClassifier::new();
        let result = classifier.classify_simple(
            &[
                "Virgin".to_string(),
                "Martyr".to_string(),
                "Saint Agnes".to_string(),
            ],
            "St. Agnes, Virgin and Martyr",
        );
        assert_eq!(result, Some("Virgins".to_string()));
    }

    #[test]
    fn test_bishop_confessor() {
        let classifier = TitleClassifier::new();
        let result =
            classifier.classify_simple(&["Bishop".to_string()], "St. Robert Bellarmine");
        assert_eq!(result, Some("Confessor Bishops".to_string()));
    }

    #[test]
    fn test_plain_martyr() {
        let classifier = TitleClassifier::new();
        let result = classifier.classify_simple(&["Martyr".to_string()], "St. Stephen");
        assert_eq!(result, Some("Martyrs".to_string()));
    }

    #[test]
    fn test_ordinary_confessor() {
        let classifier = TitleClassifier::new();
        let result =
            classifier.classify_simple(&["Confessor".to_string()], "St. Anselm");
        assert_eq!(result, Some("Confessors (Non-Bishop)".to_string()));
    }

    #[test]
    fn test_abbot_aliased_to_confessor() {
        let classifier = TitleClassifier::new();
        let result = classifier.classify_simple(&["Abbot".to_string()], "St. Anthony");
        assert_eq!(result, Some("Confessors (Non-Bishop)".to_string()));
    }

    #[test]
    fn test_apostle() {
        let classifier = TitleClassifier::new();
        let result =
            classifier.classify_simple(&["Apostle".to_string()], "St. James");
        assert_eq!(result, Some("Apostles".to_string()));
    }

    #[test]
    fn test_no_match() {
        let classifier = TitleClassifier::new();
        let titles: Vec<String> = vec![];
        let result = classifier.classify_simple(&titles, "Unknown Saint");
        assert_eq!(result, None);
    }
}
