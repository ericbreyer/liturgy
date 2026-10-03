//! Abstraction layer for retrieving liturgical rules.
//!
//! This module decouples the rule resolution logic from any specific data source
//! (TOML files, databases, APIs, etc.). A `RuleProvider` implementation can be swapped
//! in to support different rule storage and retrieval mechanisms.

use anyhow::Result;
use types::DayRank62Office;

#[derive(Debug, Clone)]
pub struct MatchedProperRule {
    pub id: String,
    pub common: Option<String>,
    pub toml_bytes: Vec<u8>,
}

/// A source-agnostic interface for retrieving office rules.
///
/// Implementations can load rules from TOML files, databases, APIs, or other sources.
pub trait RuleProvider: Send + Sync {
    /// Retrieve a proper office rule for a specific day.
    ///
    /// This is called with feast keys (slugified identifiers) in priority order,
    /// and should return the first matching rule along with its explicit common classification.
    ///
    /// # Arguments
    /// - `feast_keys`: Priority-ordered list of feast identifiers (e.g., ["christmas", "nativity"])
    ///
    /// # Returns
    /// - `Ok(Some(rule))` with metadata and complete TOML data if a rule matches
    /// - `Ok(None)` if no rule matches these feast keys
    /// - `Err` if retrieval failed
    fn get_proper_rule(
        &self,
        feast_keys: &[String],
    ) -> Result<Option<MatchedProperRule>>;

    /// Retrieve an office template by its canonical key.
    ///
    /// Office keys are derived from DayRank62Office variants (e.g., "office-sunday", "office-ferial").
    ///
    /// # Arguments
    /// - `key`: The canonical office key (e.g., "office-sunday")
    ///
    /// # Returns
    /// - `Ok(office_data)` if the office exists
    /// - `Err` if the office doesn't exist or retrieval failed
    fn get_office_rule(&self, key: &str) -> Result<Vec<u8>>;

    /// List all available proper rule keys.
    ///
    /// Useful for discovery, debugging, and validation.
    fn list_proper_keys(&self) -> Result<Vec<String>>;

    /// List all available office rule keys.
    ///
    /// Useful for discovery, debugging, and validation.
    fn list_office_keys(&self) -> Result<Vec<String>>;
}

/// Helper to construct a standard office key from a DayRank62Office variant.
pub fn office_key_for_rank(office: &DayRank62Office) -> &'static str {
    match office {
        DayRank62Office::Sunday => "office-sunday",
        DayRank62Office::Feastial => "office-feastial",
        DayRank62Office::Semifestial => "office-semifestial",
        DayRank62Office::Ordinary => "office-ordinary",
        DayRank62Office::Ferial => "office-ferial",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_office_key_for_rank() {
        assert_eq!(office_key_for_rank(&DayRank62Office::Sunday), "office-sunday");
        assert_eq!(office_key_for_rank(&DayRank62Office::Ferial), "office-ferial");
    }
}
