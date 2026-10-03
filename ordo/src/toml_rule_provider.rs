//! TOML-based rule provider implementation.
//!
//! This implementation loads rules from TOML files in a standard directory structure:
//! ```
//! rules/
//!   propers/
//!     seasonal/        # *.toml files for seasonal rules
//!     common/          # *.toml files for common rules
//!   offices/           # *.toml files for office templates
//! ```

use std::{
    collections::HashMap,
    env,
    fs::{self, ReadDir},
    path::{Path, PathBuf},
};

use anyhow::{Result, bail, Context};
use serde::Deserialize;

use crate::rule_provider::{MatchedProperRule, RuleProvider};

/// TOML representation of a proper office rule.
#[derive(Debug, Deserialize)]
pub struct TomlProperRule {
    pub id: Option<String>,
    pub name: Option<String>,
    pub common: Option<String>,
}

/// TOML-based rule provider that loads rules from the filesystem.
pub struct TomlRuleProvider {
    /// Map of feast keys (slugified names) to (rule_id, rule_common, raw_toml_string)
    propers: HashMap<String, (String, Option<String>, String)>,
    /// Map of office keys to raw TOML strings
    offices: HashMap<String, String>,
}

impl TomlRuleProvider {
    /// Load rules from a directory on the filesystem.
    ///
    /// The directory should have the standard structure:
    /// - `propers/` subdirectories containing .toml files
    /// - `offices/` containing .toml files
    pub fn load_from_dir<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let mut propers = HashMap::new();
        let mut offices = HashMap::new();

        // Resolve relative paths against the crate's manifest dir so tests can use
        // relative paths.
        let rules_dir: PathBuf = if dir.as_ref().is_absolute() {
            dir.as_ref().to_path_buf()
        } else {
            let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
            let candidate1 = manifest.join(dir.as_ref());
            let candidate2 = manifest
                .parent()
                .map(|p| p.join(dir.as_ref()))
                .unwrap_or(candidate1.clone());
            // Prefer the first candidate that exists, otherwise use candidate1
            if candidate2.exists() {
                candidate2
            } else {
                candidate1
            }
        };
        eprintln!("TomlRuleProvider: loading rules from {}", rules_dir.display());

        let Ok(subdirs) = fs::read_dir(rules_dir.join("propers")) else {
            bail!(
                "failed to read propers subdirectory in {}",
                rules_dir.display()
            );
        };

        propers.extend(
            subdirs
                .into_iter()
                .flatten()
                .map(|s| s.path())
                .inspect(|p| eprintln!("TomlRuleProvider: processing proper dir {}", p.display()))
                .filter(|s| s.is_dir())
                .map(fs::read_dir)
                .filter_map(Result::ok)
                .flat_map(ReadDir::flatten)
                .map(|e| e.path())
                .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("toml"))
                .map(|p| fs::read_to_string(p.clone()).map(|s| (p, s)))
                .filter_map(Result::ok)
                .map(|(p, s)| {
                    let rule: TomlProperRule = toml::from_str(&s)
                        .context(format!("failed to parse TOML rule: {}", p.display()))?;
                    let rule_id = rule.id.clone().or_else(|| rule.name.clone())
                        .unwrap_or_else(|| p.file_stem()
                            .and_then(|osstr| osstr.to_str())
                            .unwrap_or("unknown")
                            .to_string());
                    let slug_key = slug(p.file_stem()
                        .and_then(|osstr| osstr.to_str())
                        .unwrap_or("unknown"));
                    Ok::<_, anyhow::Error>((slug_key, (rule_id, rule.common, s)))
                })
                .inspect(|r| {
                    if let Err(err) = r {
                        eprintln!("TomlRuleProvider: error loading proper rule: {err}");
                    }
                })
                .filter_map(Result::ok),
        );

        let Ok(entries) = fs::read_dir(rules_dir.join("offices")) else {
            bail!(
                "failed to read offices subdirectory in {}",
                rules_dir.display()
            );
        };
        offices.extend(
            entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("toml"))
                .map(|p| {
                    fs::read_to_string(&p).map(|s| {
                        let slug_key = slug(p
                            .file_stem()
                            .and_then(|osstr| osstr.to_str())
                            .unwrap_or("unknown"));
                        (slug_key, s)
                    })
                })
                .filter_map(Result::ok),
        );

        Ok(TomlRuleProvider { propers, offices })
    }
}

impl RuleProvider for TomlRuleProvider {
    fn get_proper_rule(
        &self,
        feast_keys: &[String],
    ) -> Result<Option<MatchedProperRule>> {
        Ok(feast_keys
            .iter()
            .find_map(|fk| {
                self.propers.get(fk).map(|(id, common, raw)| {
                    MatchedProperRule {
                        id: id.clone(),
                        common: common.clone(),
                        toml_bytes: raw.as_bytes().to_vec(),
                    }
                })
            }))
    }

    fn get_office_rule(&self, key: &str) -> Result<Vec<u8>> {
        self.offices
            .get(key)
            .map(|s| s.as_bytes().to_vec())
            .ok_or_else(|| anyhow::anyhow!("office rule not found: {}", key))
    }

    fn list_proper_keys(&self) -> Result<Vec<String>> {
        Ok(self.propers.keys().cloned().collect())
    }

    fn list_office_keys(&self) -> Result<Vec<String>> {
        Ok(self.offices.keys().cloned().collect())
    }
}

/// Convert a string to a slug (lowercase, alphanumeric, hyphens).
fn slug(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slug() {
        assert_eq!(slug("Christmas Day"), "christmas-day");
        assert_eq!(slug("Feast of St. John"), "feast-of-st--john");
        assert_eq!(slug("Office-Sunday"), "office-sunday");
    }

    #[test]
    fn proper_lookup_preserves_priority_metadata_and_complete_data() {
        let raw = "common = 'Fixture Common'\n[vespers]\ncollect = 'Proper'";
        let provider = TomlRuleProvider {
            propers: HashMap::from([
                ("specific".to_string(), ("specific-id".to_string(), Some("Fixture Common".to_string()), raw.to_string())),
                ("seasonal".to_string(), ("seasonal-id".to_string(), None, "[vespers]".to_string())),
            ]),
            offices: HashMap::new(),
        };
        let matched = provider.get_proper_rule(&["missing".to_string(), "specific".to_string(), "seasonal".to_string()]).unwrap().unwrap();
        assert_eq!(matched.id, "specific-id");
        assert_eq!(matched.common.as_deref(), Some("Fixture Common"));
        assert_eq!(matched.toml_bytes, raw.as_bytes());
        assert!(provider.get_proper_rule(&["missing".to_string()]).unwrap().is_none());
    }
}
