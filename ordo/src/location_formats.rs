//! Location format strings and parsing helpers.
//!
//! This module centralizes the hard-coded location token format strings and
//! provides clear, named constants for location prefixes to reduce magic strings
//! scattered throughout the codebase.

/// Prefix for octave locations (e.g., "Octave of Christmas")
pub const OCTAVE_PREFIX: &str = "Octave of ";

/// Prefix for common locations (e.g., "Common of Martyrs")
pub const COMMON_PREFIX: &str = "Common of ";

/// Prefix for ordinary locations (e.g., "Ordinary of Advent")
pub const ORDINARY_PREFIX: &str = "Ordinary of ";

/// Literal location string for "Proper"
pub const PROPER: &str = "Proper";

/// Literal location string for "Psalter" (default psalms)
pub const PSALTER: &str = "Psalter";

/// Literal location string for generic "Common"
pub const COMMON: &str = "Common";

/// Literal location strings for ordinary/ferial (all treated the same)
/// These are accepted synonyms for backward compatibility.
pub const FERIA: &str = "Feria";
pub const FERIAL: &str = "Ferial";
pub const ORDINARY: &str = "Ordinary";

/// Prefix for Sunday locations (e.g., "Sunday in Advent")
pub const SUNDAY_PREFIX: &str = "Sunday";

/// Validates and extracts the name from an octave location string.
///
/// Valid formats:
/// - "Octave of Christmas" → Some("Christmas")
/// - "Octave of Epiphany" → Some("Epiphany")
///
/// Returns None if the input doesn't start with OCTAVE_PREFIX.
pub fn extract_octave_name(loc: &str) -> Option<&str> {
    loc.strip_prefix(OCTAVE_PREFIX).map(|s| s.trim())
}

/// Validates and extracts the name from a common location string.
///
/// Valid formats:
/// - "Common of Martyrs" → Some("Martyrs")
/// - "Common of Virgins" → Some("Virgins")
///
/// Returns None if the input doesn't start with COMMON_PREFIX.
pub fn extract_common_name(loc: &str) -> Option<&str> {
    loc.strip_prefix(COMMON_PREFIX).map(|s| s.trim())
}

/// Validates and extracts the name from an ordinary location string.
///
/// Valid formats:
/// - "Ordinary of Advent" → Some("Advent")
/// - "Ordinary of Lent" → Some("Lent")
///
/// Returns None if the input doesn't start with ORDINARY_PREFIX.
pub fn extract_ordinary_name(loc: &str) -> Option<&str> {
    loc.strip_prefix(ORDINARY_PREFIX).map(|s| s.trim())
}

/// Validates and extracts the descriptor from a Sunday location string.
///
/// Valid formats:
/// - "Sunday in Advent" → Some(" in Advent")
/// - "Sunday" → Some("")
///
/// Returns None if the input doesn't start with "Sunday".
pub fn extract_sunday_descriptor(loc: &str) -> Option<&str> {
    loc.strip_prefix(SUNDAY_PREFIX).map(|s| s.trim())
}

/// Checks if a location string looks like a "ferial" location.
///
/// These are locations that reference ordinary weekday (feria) psalms.
pub fn is_ferial_location(loc: &str) -> bool {
    loc == FERIA || loc == FERIAL || loc == ORDINARY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_octave_name() {
        assert_eq!(extract_octave_name("Octave of Christmas"), Some("Christmas"));
        assert_eq!(
            extract_octave_name("Octave of Epiphany"),
            Some("Epiphany")
        );
        assert_eq!(extract_octave_name("Common of Martyrs"), None);
        assert_eq!(extract_octave_name("Proper"), None);
    }

    #[test]
    fn test_extract_common_name() {
        assert_eq!(extract_common_name("Common of Martyrs"), Some("Martyrs"));
        assert_eq!(extract_common_name("Common of Virgins"), Some("Virgins"));
        assert_eq!(extract_common_name("Octave of Christmas"), None);
    }

    #[test]
    fn test_extract_ordinary_name() {
        assert_eq!(extract_ordinary_name("Ordinary of Advent"), Some("Advent"));
        assert_eq!(
            extract_ordinary_name("Ordinary of Lent"),
            Some("Lent")
        );
        assert_eq!(extract_ordinary_name("Common of Martyrs"), None);
    }

    #[test]
    fn test_extract_sunday_descriptor() {
        assert_eq!(extract_sunday_descriptor("Sunday in Advent"), Some("in Advent"));
        assert_eq!(extract_sunday_descriptor("Sunday"), Some(""));
        assert_eq!(extract_sunday_descriptor("Ordinary"), None);
    }

    #[test]
    fn test_is_ferial_location() {
        assert!(is_ferial_location("Feria"));
        assert!(is_ferial_location("Ferial"));
        assert!(is_ferial_location("Ordinary"));
        assert!(!is_ferial_location("Proper"));
        assert!(!is_ferial_location("Psalter"));
    }
}
