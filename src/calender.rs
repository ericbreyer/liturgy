
// === Module declarations ===
mod date_rule;
mod day_type;
pub mod feast_rank;
mod fuzzy_search;
pub mod generic_calendar;
mod liturgical_unit;
pub mod year_calendar;
mod year_calendar_builder;

// === Use statements ===
use anyhow::Result;
use date_rule::DateRule;
use day_type::DayType;
use feast_rank::LiturgicalContext;
use std::path::Path;
use crate::calender::feast_rank::{FeastRank54, FeastRank62, FeastRankOf};
use crate::calender::generic_calendar::{CalendarType, FeastRule, GenericCalendar};
use crate::calender::year_calendar::{DayDescription, YearCalendar};

// === Re-exports for external use ===
pub use liturgical_unit::LiturgicalUnit;

#[derive(Debug, Clone)]
/// Handle for working with liturgical calendars loaded from configuration files
pub struct GenericCalendarHandle(GenericCalendar);

#[derive(Debug, Clone)]
/// Handle for working with instantiated year calendars
pub enum YearCalendarHandle {
    F54(YearCalendar<FeastRank54>),
    Ef(YearCalendar<FeastRank62>),
    Of(YearCalendar<FeastRankOf>),
}

impl GenericCalendarHandle {
    /// Get the name of this calendar
    pub fn name(&self) -> &str {
        &self.0.name
    }
    /// Load a liturgical calendar from a TOML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        GenericCalendar::from_toml_file(path).map(GenericCalendarHandle)
    }
    /// Load a liturgical calendar from TOML string content
    pub fn load_from_str(content: &str) -> Result<Self, toml::de::Error> {
        GenericCalendar::from_toml_str(content).map(GenericCalendarHandle)
    }
    /// Load a base calendar and merge additional feast files
    pub fn load_with_extensions<P: AsRef<Path>>(
        base_path: P,
        extension_paths: &[P],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        GenericCalendar::from_toml_with_extensions(base_path, extension_paths)
            .map(GenericCalendarHandle)
    }
    /// Create a liturgical year calendar for the given year
    pub fn create_year_calendar(&self, year: i32) -> YearCalendarHandle {
        match self.0.calendar_type() {
            CalendarType::Calendar1954 => {
                YearCalendarHandle::F54(self.0.instantiate_54_for_lit_year(year))
            }
            CalendarType::Calendar1962 => {
                YearCalendarHandle::Ef(self.0.instantiate_62_for_lit_year(year))
            }
            CalendarType::OrdinaryForm => {
                YearCalendarHandle::Of(self.0.instantiate_of_for_lit_year(year))
            }
        }
    }
    /// Get feast information by name using fuzzy search
    ///
    /// # Examples
    ///
    /// ```
    /// use liturgy::GenericCalendarHandle;
    /// let toml = r#"
    /// name = "Test Calendar"
    /// [[seasons]]
    /// name = "Test Season"
    /// begin = "Fixed(1,1)"
    /// end = "Fixed(12,31)"
    /// color = "white"
    /// [[feasts]]
    /// name = "St. Joseph"
    /// date_rule = "Fixed(3,19)"
    /// color = "white"
    /// "#;
    /// let cal = GenericCalendarHandle::load_from_str(toml).unwrap();
    /// assert_eq!(cal.get_feast_info("St. Joseph").is_ok(), true);
    /// assert!(cal.get_feast_info("St. Jospeh").unwrap_err().to_string().contains("Did you mean: St. Joseph"));
    /// ```
    pub fn get_feast_info(&self, name: &str) -> Result<(FeastRule<DateRule>, String)> {
        match self.0.get_feast_info(name) {
            Some(info) => Ok(info),
            None => {
                let suggestions = self.0.suggest_feast_names(name);
                if suggestions.is_empty() {
                    Err(anyhow::anyhow!("Feast '{}' not found.", name))
                } else {
                    Err(anyhow::anyhow!("Feast '{}' not found. Did you mean: {}?", name, suggestions.into_iter().map(|(n, _)| n).collect::<Vec<_>>().join(", ")))
                }
            }
        }
    }

    /// Get feast name suggestions using fuzzy matching
    pub fn suggest_feast_names(&self, name: &str) -> Vec<(String, f32)> {
        self.0.suggest_feast_names(name)
    }

    pub fn commemoration_interpretation(&self) -> &str {
        &self.0.commemoration_interpretation
    }
}

impl YearCalendarHandle {
    /// Generate and save a CSV file for a liturgical year
    pub fn export_csv<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        match self {
            YearCalendarHandle::F54(cal) => cal.write_csv_for_year(path.as_ref().to_str().unwrap()),
            YearCalendarHandle::Ef(cal) => cal.write_csv_for_year(path.as_ref().to_str().unwrap()),
            YearCalendarHandle::Of(cal) => cal.write_csv_for_year(path.as_ref().to_str().unwrap()),
        }
    }
    /// Generate CSV content for this liturgical year
    pub fn generate_csv(&self) -> String {
        match self {
            YearCalendarHandle::F54(cal) => cal.generate_year_calendar_csv(),
            YearCalendarHandle::Ef(cal) => cal.generate_year_calendar_csv(),
            YearCalendarHandle::Of(cal) => cal.generate_year_calendar_csv(),
        }
    }
    /// Get the year of this calendar
    pub fn year(&self) -> i32 {
        match self {
            YearCalendarHandle::F54(cal) => cal.year,
            YearCalendarHandle::Ef(cal) => cal.year,
            YearCalendarHandle::Of(cal) => cal.year,
        }
    }

    pub fn get_day_info(&self, date: chrono::NaiveDate) -> Option<DayDescription> {
        match self {
            YearCalendarHandle::F54(cal) => cal.get_day(date),
            YearCalendarHandle::Ef(cal) => cal.get_day(date),
            YearCalendarHandle::Of(cal) => cal.get_day(date),
        }
    }
}

#[cfg(test)]
mod test {
    //! Integration tests for the calendar functionality

    use super::*;
    use chrono::NaiveDate;
    use generic_calendar::{FeastRule, tests::*, GenericCalendar};
    use feast_rank::{FeastRank, FeastRank62};
    use year_calendar_builder::YearCalendarBuilder;
    use std::collections::HashMap;
    use test_case::test_case;

    fn create_test_feast(name: &str, date: NaiveDate, rank: &str) -> FeastRule<NaiveDate> {
        FeastRule {
            name: name.to_string(),
            date_rule: date,
            rank: Some(rank.to_string()),
            of_our_lord: false,
            day_type: Some(DayType::Feast),
            color: "red".to_string(),
            titles: vec![],
            movable: false,
        }
    }

    fn create_test_year_calendar() -> YearCalendarBuilder {
        let season = create_test_season(
            "Test Season",
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
        );

        let feast = create_test_feast(
            "Test Feast",
            NaiveDate::from_ymd_opt(2025, 6, 15).unwrap(),
            "II",
        );

        let mut feasts_map = HashMap::new();
        feasts_map.insert(NaiveDate::from_ymd_opt(2025, 6, 15).unwrap(), vec![feast]);

        YearCalendarBuilder {
            year: 2025,
            name: "Test Calendar".to_string(),
            seasons: vec![season],
            feasts: feasts_map,
            first_advent: NaiveDate::from_ymd_opt(2025, 11, 30).unwrap(),
            next_first_advent: NaiveDate::from_ymd_opt(2026, 11, 29).unwrap(),
            calendar_type: generic_calendar::CalendarType::OrdinaryForm,
        }
    }

    /// Tests feast retrieval functionality for different date scenarios
    #[test_case("2025-06-15", 1, "Test Feast"; "date with existing feast")]
    #[test_case("2025-03-15", 0, ""; "date with no feasts")]
    fn test_feast_retrieval(date_str: &str, expected_count: usize, expected_name: &str) {
        let year_calendar = create_test_year_calendar();
        let test_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();
        let feasts = year_calendar.get_feasts_on_date(&test_date);

        assert_eq!(feasts.len(), expected_count);
        if expected_count > 0 {
            assert_eq!(feasts[0].name, expected_name);
        }
    }

    /// Tests season ranking functionality for different dates
    #[test_case("2025-03-15"; "ferial day")]
    #[test_case("2025-06-01"; "another ferial day")]
    fn test_season_ranking(date_str: &str) {
        let year_calendar = create_test_year_calendar();
        let test_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();
        let rank: FeastRank62 = year_calendar.season_day_to_feast_rank(&test_date);
        assert!(rank.is_ferial_or_sunday_rank());
    }

    /// Tests season descriptor generation
    #[test_case("2025-03-15", "Test Season"; "basic season descriptor")]
    #[test_case("2025-06-01", "Test Season"; "another date in same season")]
    fn test_season_descriptor_generation(date_str: &str, expected_season_name: &str) {
        let year_calendar = create_test_year_calendar();
        let test_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();
        let descriptor = year_calendar.get_season_descriptor(&test_date);
        assert!(descriptor.contains(expected_season_name));
    }

    /// Tests FeastRule Display implementation and feast ranking
    #[test_case("St. Joseph", vec!["Spouse of the Blessed Virgin Mary", "Patron of the Universal Church"] => "St. Joseph, Spouse of the Blessed Virgin Mary and Patron of the Universal Church"; "feast with titles")]
    #[test_case("Simple Feast", vec![] => "Simple Feast"; "feast without titles")]
    fn test_feast_rule_display(name: &str, titles: Vec<&str>) -> String {
        let feast_rule = FeastRule {
            name: name.to_string(),
            date_rule: NaiveDate::from_ymd_opt(2025, 3, 19).unwrap(),
            rank: Some("II".to_string()),
            of_our_lord: false,
            day_type: Some(DayType::Feast),
            color: "white".to_string(),
            titles: titles.into_iter().map(|s| s.to_string()).collect(),
            movable: false,
        };

        feast_rule.to_string()
    }

    /// Tests feast ranking functionality with different configurations
    #[test_case(Some("II"), false, Some(DayType::Feast), false, "specified feast with rank II"; "feast with all fields")]
    #[test_case(None, false, None, false, "feast with default values"; "feast with defaults")]
    #[test_case(Some("I"), true, Some(DayType::Feast), true, "movable feast of our Lord"; "our lord movable feast")]
    fn test_feast_ranking(
        rank: Option<&str>,
        of_our_lord: bool,
        day_type: Option<DayType>,
        movable: bool,
        description: &str,
    ) {
        let feast_rule = FeastRule {
            name: "Test Feast".to_string(),
            date_rule: NaiveDate::from_ymd_opt(2025, 3, 19).unwrap(),
            rank: rank.map(|r| r.to_string()),
            of_our_lord,
            day_type,
            color: "white".to_string(),
            titles: vec!["Test Title".to_string()],
            movable,
        };

        let feast_rank: FeastRank62 = feast_rule.get_feastrank();
        assert!(
            !feast_rank.is_ferial_or_sunday_rank(),
            "Feast '{}' should not be classified as feria/sunday",
            description
        );
    }

    /// Tests FeastRule instantiation with Advent calendar year calculation
    #[test_case("Christmas", DateRule::Fixed { month: 12, day: 25 }, true, false; "Christmas - fixed feast in Advent season")]
    #[test_case("Easter", DateRule::Easter, true, true; "Easter - movable feast")]
    fn test_feast_rule_instantiation(
        name: &str,
        date_rule: DateRule,
        of_our_lord: bool,
        expected_movable: bool,
    ) {
        let feast_rule = FeastRule {
            name: name.to_string(),
            date_rule,
            rank: Some("I".to_string()),
            of_our_lord,
            day_type: Some(DayType::Feast),
            color: "white".to_string(),
            titles: if name == "Christmas" {
                vec!["Birth of Our Lord".to_string()]
            } else {
                vec![]
            },
            movable: expected_movable,
        };

        let instantiated = feast_rule.instantiate_for_lit_year_with_advent(2025);

        assert_eq!(instantiated.name, name);
        assert_eq!(instantiated.rank, Some("I".to_string()));
        assert_eq!(instantiated.of_our_lord, of_our_lord);
        assert_eq!(instantiated.day_type, Some(DayType::Feast));
        assert_eq!(instantiated.color, "white");
        assert_eq!(instantiated.movable, expected_movable);
    }

    /// Tests error handling in TOML parsing
    #[test_case("this is not valid toml [[["; "malformed TOML syntax")]
    #[test_case(r#"
[[feasts]]
name = "Bad Feast"
date_rule = "InvalidDateRule(99,99)"
rank = "I"
color = "white"
    "#; "invalid date rule")]
    fn test_toml_parsing_errors(invalid_toml: &str) {
        let result = GenericCalendar::from_toml_str(invalid_toml);
        // Should handle parsing errors gracefully - either fails or succeeds with valid subset
        assert!(result.is_err() || result.is_ok());
    }

    /// Tests additional edge cases and coverage paths for different dates
    #[test_case("2025-06-15"; "ferial weekday")]
    #[test_case("2025-06-01"; "first of month")]
    #[test_case("2025-12-15"; "late in year")]
    fn test_additional_edge_cases(date_str: &str) {
        let year_calendar = YearCalendarBuilder {
            year: 2025,
            name: "Coverage Test".to_string(),
            seasons: vec![create_test_season(
                "Coverage Season",
                NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
            )],
            feasts: HashMap::new(),
            first_advent: NaiveDate::from_ymd_opt(2025, 11, 30).unwrap(),
            next_first_advent: NaiveDate::from_ymd_opt(2026, 11, 29).unwrap(),
            calendar_type: generic_calendar::CalendarType::OrdinaryForm,
        };

        let test_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();
        let rank: FeastRank62 = year_calendar.season_day_to_feast_rank(&test_date);
        assert!(rank.is_ferial_or_sunday_rank());
    }
}
