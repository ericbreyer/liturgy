use std::collections::HashMap;

use chrono::NaiveDate;
pub use feast_rule::FeastRule;
pub use season_rule::SeasonRule;
use serde::{Deserialize, Serialize};

use crate::calender::{
    feast_rank::{FeastRank, FeastRank54, FeastRank62, FeastRankOf},
    year_calendar::YearCalendar,
    year_calendar_builder::YearCalendarBuilder,
    DateRule,
    fuzzy_search::fuzzy_search_best_n,
};

mod feast_rule;
mod season_rule;

/// Calendar system type identifier
#[derive(Debug, Clone, PartialEq)]
pub enum CalendarType {
    /// 1954 Roman Calendar (Pre-Pius XII reforms)
    Calendar1954,
    /// 1962 Roman Calendar (Extraordinary Form)
    Calendar1962,
    /// Ordinary Form Calendar (Post-Vatican II)  
    OrdinaryForm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericCalendar {
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_commemoration_interpretation")]
    pub commemoration_interpretation: String,
    #[serde(default)]
    pub seasons: Vec<SeasonRule<DateRule>>,
    pub feasts: Vec<FeastRule<DateRule>>,
}

fn default_commemoration_interpretation() -> String {
    "Commemoration".to_string()
}

impl GenericCalendar {
    /// Load a calendar from TOML string content
    pub fn from_toml_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    /// Load a calendar from a TOML file
    pub fn from_toml_file<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let calendar = Self::from_toml_str(&content)?;
        Ok(calendar)
    }
    #[cfg(test)]
    /// Get the name of this calendar
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Determine the calendar type based on the name
    pub fn calendar_type(&self) -> CalendarType {
        if self.name.to_lowercase().contains("1954") {
            CalendarType::Calendar1954
        } else if self.name.to_lowercase().contains("1962")
            || self.name.to_lowercase().contains("extraordinary")
            || self.name.to_lowercase().contains("tridentine")
        {
            CalendarType::Calendar1962
        } else {
            CalendarType::OrdinaryForm
        }
    }

    #[cfg(test)]
    /// Get the seasons defined in this calendar
    pub fn seasons(&self) -> &[SeasonRule<DateRule>] {
        &self.seasons
    }

    #[cfg(test)]
    /// Get the feasts defined in this calendar
    pub fn feasts(&self) -> &[FeastRule<DateRule>] {
        &self.feasts
    }

    /// Merge additional feasts from another calendar into this one
    pub fn merge_feasts(&mut self, other: GenericCalendar) {
        // merge strategy:
        // 1. if a feast with the same name and date_rule exists, replace it
        // 2. otherwise, add the new feast to the list
        for new_feast in other.feasts {
            if let Some(pos) = self.feasts.iter().position(|f| f.name == new_feast.name) {
                let mut details = String::new();
                if new_feast.date_rule != self.feasts[pos].date_rule {
                    details.push_str(" (transfered)");
                }
                if new_feast.rank != self.feasts[pos].rank {
                    if !details.is_empty() {
                        details.push_str(", ");
                    }
                    details.push_str(" (rank changed)");
                }

                self.feasts[pos] =
                    new_feast;//.add_extensions_prefix(&format!("{}{}", other.name, details));
            } else {
                self.feasts
                    .push(new_feast);//.add_extensions_prefix(&other.name));
            }
        }

        self.name = format!("{} with {} Extensions", self.name, other.name);
    }

    /// Load and merge additional feasts from a TOML file
    fn load_and_merge_feasts_from_file<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        self.load_and_merge_feasts_from_str(&content)?;
        Ok(())
    }

    /// Load and merge additional feasts from TOML string content
    fn load_and_merge_feasts_from_str(&mut self, content: &str) -> Result<(), toml::de::Error> {
        // Try to parse as a full calendar first
        let additional_calendar = Self::from_toml_str(content)?;
        self.merge_feasts(additional_calendar);
        Ok(())
    }

    /// Create a new calendar by loading a base calendar and merging additional feast files
    pub fn from_toml_with_extensions<P: AsRef<std::path::Path>>(
        base_path: P,
        extension_paths: &[P],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut calendar = Self::from_toml_file(base_path)?;

        for extension_path in extension_paths {
            calendar.load_and_merge_feasts_from_file(extension_path)?;
        }

        Ok(calendar)
    }

    /// Create a year calendar for a specific liturgical year
    pub fn instantiate_62_for_lit_year(&self, lit_year: i32) -> YearCalendar<FeastRank62> {
        // First, figure out when Advent starts to determine which feasts belong to which year
        let advent_season = self
            .seasons
            .iter()
            .find(|s| s.name().to_lowercase().contains("advent"));
        let advent = advent_season.expect("No Advent season found in calendar");
        let first_advent = advent.begin().to_day(lit_year).unwrap();
        let next_first_advent = advent.begin().to_day(lit_year + 1).unwrap();

        let seasons = {
            // Create a mapping of season names to season objects for parent lookups
            let season_map: std::collections::HashMap<String, &SeasonRule<DateRule>> = self
                .seasons
                .iter()
                .map(|s| (s.name().to_string(), s))
                .collect();

            // Helper function to recursively resolve hierarchy
            fn resolve_hierarchy_chain(
                season: &SeasonRule<DateRule>,
                season_map: &std::collections::HashMap<String, &SeasonRule<DateRule>>,
                lit_year: i32,
                visited: &mut std::collections::HashSet<String>,
            ) -> SeasonRule<NaiveDate> {
                // Prevent infinite loops
                if visited.contains(season.name()) {
                    return season.instantiate_for_lit_year(lit_year);
                }
                visited.insert(season.name().to_string());

                let parent_season = season
                    .parent_season()
                    .as_ref()
                    .and_then(|parent_name| season_map.get(parent_name))
                    .map(|parent| resolve_hierarchy_chain(parent, season_map, lit_year, visited));

                let result = season.instantiate_with_hierarchy(lit_year, parent_season.as_ref());
                visited.remove(season.name());
                result
            }

            // Instantiate seasons with proper hierarchy resolution
            self.seasons
                .iter()
                .map(|s| {
                    let mut visited = std::collections::HashSet::new();
                    resolve_hierarchy_chain(s, &season_map, lit_year, &mut visited)
                })
                .collect()
        };
        let feasts = self
            .feasts
            .iter()
            .map(|f| f.instantiate_for_lit_year_with_advent(lit_year))
            .fold(HashMap::new(), |mut acc: HashMap<_, Vec<_>>, feast| {
                acc.entry(feast.date_rule).or_default().push(feast);
                acc
            });

        YearCalendarBuilder {
            year: lit_year,
            #[cfg(test)]
            name: self.name.clone(),
            seasons,
            feasts,
            first_advent,
            next_first_advent,
            calendar_type: CalendarType::Calendar1962,
        }
        .generate_year_calendar::<FeastRank62>()
    }

    /// Create a 1954 calendar year calendar for a specific liturgical year
    pub fn instantiate_54_for_lit_year(&self, lit_year: i32) -> YearCalendar<FeastRank54> {
        // First, figure out when Advent starts to determine which feasts belong to which year
        let advent_season = self
            .seasons
            .iter()
            .find(|s| s.name().to_lowercase().contains("advent"));
        let advent = advent_season.expect("No Advent season found in calendar");
        let first_advent = advent.begin().to_day(lit_year).unwrap();
        let next_first_advent = advent.begin().to_day(lit_year + 1).unwrap();

        let seasons = {
            // Create a mapping of season names to season objects for parent lookups
            let season_map: std::collections::HashMap<String, &SeasonRule<DateRule>> = self
                .seasons
                .iter()
                .map(|s| (s.name().to_string(), s))
                .collect();

            // Helper function to recursively resolve hierarchy
            fn resolve_hierarchy_chain(
                season: &SeasonRule<DateRule>,
                season_map: &std::collections::HashMap<String, &SeasonRule<DateRule>>,
                lit_year: i32,
                visited: &mut std::collections::HashSet<String>,
            ) -> SeasonRule<NaiveDate> {
                // Prevent infinite loops
                if visited.contains(season.name()) {
                    return season.instantiate_for_lit_year(lit_year);
                }
                visited.insert(season.name().to_string());

                let parent_season = season
                    .parent_season()
                    .as_ref()
                    .and_then(|parent_name| season_map.get(parent_name))
                    .map(|parent| resolve_hierarchy_chain(parent, season_map, lit_year, visited));

                let result = season.instantiate_with_hierarchy(lit_year, parent_season.as_ref());
                visited.remove(season.name());
                result
            }

            // Instantiate seasons with proper hierarchy resolution
            self.seasons
                .iter()
                .map(|s| {
                    let mut visited = std::collections::HashSet::new();
                    resolve_hierarchy_chain(s, &season_map, lit_year, &mut visited)
                })
                .collect()
        };
        let feasts = self
            .feasts
            .iter()
            .map(|f| f.instantiate_for_lit_year_with_advent(lit_year))
            .fold(HashMap::new(), |mut acc: HashMap<_, Vec<_>>, feast| {
                acc.entry(feast.date_rule).or_default().push(feast);
                acc
            });

        YearCalendarBuilder {
            year: lit_year,
            #[cfg(test)]
            name: self.name.clone(),
            seasons,
            feasts,
            first_advent,
            next_first_advent,
            calendar_type: CalendarType::Calendar1954,
        }
        .generate_year_calendar::<FeastRank54>()
    }

    /// Create an Ordinary Form year calendar for a specific liturgical year
    pub fn instantiate_of_for_lit_year(&self, lit_year: i32) -> YearCalendar<FeastRankOf> {
        // First, figure out when Advent starts to determine which feasts belong to which year
        let advent_season = self
            .seasons
            .iter()
            .find(|s| s.name().to_lowercase().contains("advent"));
        let advent = advent_season.expect("No Advent season found in calendar");
        let first_advent = advent.begin().to_day(lit_year).unwrap();
        let next_first_advent = advent.begin().to_day(lit_year + 1).unwrap();

        let seasons = {
            // Create a mapping of season names to season objects for parent lookups
            let season_map: std::collections::HashMap<String, &SeasonRule<DateRule>> = self
                .seasons
                .iter()
                .map(|s| (s.name().to_string(), s))
                .collect();

            // Helper function to recursively resolve hierarchy
            fn resolve_hierarchy_chain(
                season: &SeasonRule<DateRule>,
                season_map: &std::collections::HashMap<String, &SeasonRule<DateRule>>,
                lit_year: i32,
                visited: &mut std::collections::HashSet<String>,
            ) -> SeasonRule<NaiveDate> {
                // Prevent infinite loops
                if visited.contains(season.name()) {
                    return season.instantiate_for_lit_year(lit_year);
                }
                visited.insert(season.name().to_string());

                let parent_season = season
                    .parent_season()
                    .as_ref()
                    .and_then(|parent_name| season_map.get(parent_name))
                    .map(|parent| resolve_hierarchy_chain(parent, season_map, lit_year, visited));

                let result = season.instantiate_with_hierarchy(lit_year, parent_season.as_ref());
                visited.remove(season.name());
                result
            }

            // Instantiate seasons with proper hierarchy resolution
            self.seasons
                .iter()
                .map(|s| {
                    let mut visited = std::collections::HashSet::new();
                    resolve_hierarchy_chain(s, &season_map, lit_year, &mut visited)
                })
                .collect()
        };
        let feasts = self
            .feasts
            .iter()
            .map(|f| f.instantiate_for_lit_year_with_advent(lit_year))
            .fold(HashMap::new(), |mut acc: HashMap<_, Vec<_>>, feast| {
                acc.entry(feast.date_rule).or_default().push(feast);
                acc
            });

        YearCalendarBuilder {
            year: lit_year,
            #[cfg(test)]
            name: self.name.clone(),
            seasons,
            feasts,
            first_advent,
            next_first_advent,
            calendar_type: CalendarType::OrdinaryForm,
        }
        .generate_year_calendar::<FeastRankOf>()
    }

    /// Get feast info by exact name match (case-insensitive)
    pub fn get_feast_info(&self, name: &str) -> Option<(FeastRule<DateRule>, String)> {
        let name_lower = name.to_lowercase();
        self.feasts
            .iter()
            .find(|f| f.name.to_lowercase() == name_lower)
            .map(|f| {
                (
                    f.clone(),
                    match self.calendar_type() {
                        CalendarType::Calendar1954 => f.get_feastrank::<FeastRank54>().get_rank_string(),
                        CalendarType::Calendar1962 => f.get_feastrank::<FeastRank62>().get_rank_string(),
                        CalendarType::OrdinaryForm => f.get_feastrank::<FeastRankOf>().get_rank_string()
                    },
                )
            })
            
    }

    /// Suggest feast names using fuzzy matching (case-insensitive, substring or fuzzy search)
    pub fn suggest_feast_names(&self, name: &str) -> Vec<(String, f32)> {
        let feast_names_and_titles: Vec<String> = self
            .feasts
            .iter()
            .map(|f| format!("{}\\{}", f.name, f.titles.join(",")))
            .collect();
        let feast_names: Vec<String> = self.feasts.iter().map(|f| f.name.clone()).collect();
        let feast_names_str: Vec<&str> = feast_names.iter().map(|s| s.as_str()).collect();

        // Use rust-fuzzy-search for better fuzzy matching, get top 8 results
        let binding = feast_names_and_titles
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>();
        let fuzzy_results_titles = fuzzy_search_best_n(name, &binding, 8);
        let fuzzy_results = fuzzy_search_best_n(name, &feast_names_str, 8);

        // Merge and deduplicate results from both searches, taking the best score
        let mut combined_results: HashMap<String, f32> = HashMap::new();
        for (feast_name, score) in fuzzy_results_titles.iter().chain(fuzzy_results.iter()) {
            let entry = combined_results
                .entry(feast_name.split("\\").nth(0).unwrap().to_owned())
                .or_insert(*score);
            if *score > *entry {
                *entry = *score;
            }
        }

        // Keep results with scores for proper ordering
        let mut scored_results: Vec<(String, f32)> = combined_results
            .iter()
            .map(|(feast_name, score)| (feast_name.to_string(), *score))
            .collect();

        // If we have very few results, try some manual matching for partial words
        if scored_results.len() < 5 {
            let name_lower = name.to_lowercase();
            let query_words: Vec<&str> = name_lower.split_whitespace().collect();

            for feast in &self.feasts {
                if scored_results.len() >= 8 {
                    break;
                }

                let feast_name = feast.name.to_lowercase();
                let feast_words: Vec<&str> = feast_name.split_whitespace().collect();

                // Skip if already in fuzzy results
                if scored_results
                    .iter()
                    .any(|(r, _)| r.to_lowercase() == feast_name)
                {
                    continue;
                }

                // Check if all query words have some match in feast words
                let word_matches = query_words
                    .iter()
                    .filter(|&query_word| {
                        feast_words.iter().any(|&feast_word| {
                            feast_word.contains(query_word)
                                || query_word.contains(feast_word)
                                || feast_word.starts_with(query_word)
                                || query_word.starts_with(feast_word)
                        })
                    })
                    .count();

                // If most words match, include it with a lower score
                if word_matches >= query_words.len().saturating_sub(1) && word_matches > 0 {
                    let partial_score = 0.3 - (word_matches as f32 * 0.1); // Lower score for partial matches
                    scored_results.push((feast.name.clone(), partial_score));
                }
            }
        }

        // Sort by score descending (higher scores are better matches)
        scored_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Return just the feast names, ordered by score
        scored_results
            .into_iter()
            .filter(|(_, score)| *score > 0.2)
            .take(5)
            .collect()
    }
}

#[cfg(test)]
pub mod tests {
    pub use super::{feast_rule::FeastRule, season_rule::test::*, GenericCalendar};

    #[test]
    fn test_generic_calendar_accessors() {
        let toml_content = r#"
name = "Test Calendar"

[[seasons]]
name = "Season 1"
begin = "Fixed(1,1)"
end = "Fixed(6,30)"
color = "white"

[[seasons]]
name = "Season 2"
begin = "Fixed(7,1)"
end = "Fixed(12,31)"
color = "green"

[[feasts]]
name = "Feast 1"
date_rule = "Fixed(3,15)"
color = "red"

[[feasts]]
name = "Feast 2"
date_rule = "Fixed(9,20)"
color = "white"
"#;

        let calendar = GenericCalendar::from_toml_str(toml_content).unwrap();

        assert_eq!(calendar.name(), "Test Calendar");
        assert_eq!(calendar.seasons().len(), 2);
        assert_eq!(calendar.feasts().len(), 2);

        assert_eq!(calendar.seasons()[0].name(), "Season 1");
        assert_eq!(calendar.seasons()[1].name(), "Season 2");
    }

    #[test]
    fn test_generic_calendar_from_toml_file_error() {
        // Test loading from non-existent file
        let result = GenericCalendar::from_toml_file("non_existent_file.toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_toml_with_extensions() {
        // This would test the file-based loading with extensions
        // For now, we'll test with empty extension arrays since we don't have test files
        let base_toml = r#"
name = "Base Calendar"

[[seasons]]
name = "Season 1"
begin = "Fixed(1,1)"
end = "Fixed(12,31)"
color = "white"

[[feasts]]
name = "Base Feast"
date_rule = "Fixed(1,1)"
color = "white"
"#;

        // Create a temporary file for testing
        use std::fs;

        let temp_dir = std::env::temp_dir();
        let base_path = temp_dir.join("test_base.toml");
        fs::write(&base_path, base_toml).unwrap();

        // Test loading with no extensions
        let calendar = GenericCalendar::from_toml_with_extensions(&base_path, &[]).unwrap();
        assert_eq!(calendar.feasts().len(), 1);

        // Clean up
        fs::remove_file(&base_path).unwrap();
    }
}
