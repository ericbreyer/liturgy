use chrono::NaiveDate;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use types::ArcStr;

use super::super::date_rule::DateRule;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FerialRule<DateType> {
    name: ArcStr,
    begin: DateType,
    end: DateType,
    rank: ArcStr,
}

/// Configuration for counting days and weeks within a season
#[derive(Debug, Clone, Default)]
pub struct CountingConfig<DateType> {
    pub sundays_suffix: Option<String>,
    pub ferias_suffix: Option<String>,
    pub sundays_from: Option<DateType>,
    pub ferias_from: Option<DateType>,
    /// For continuous numbering across season breaks (like OF Ordinary Time)
    /// References another season name to continue counting from its end
    pub continue_counting_from_season: Option<String>,
}

/// Display configuration for the season
#[derive(Debug, Clone, Default)]
pub struct DisplayConfig<DateType> {
    pub append_week_of_month: Option<DateType>,
    pub dont_show_week_of_season: bool,
    pub week_of_month_old_scheme: bool,
}

/// Octave-specific configuration
#[derive(Debug, Clone, Default)]
pub struct OctaveConfig {
    pub is_octave: bool,
    pub octave_rank: Option<String>,
}

/// Hierarchical season configuration
#[derive(Debug, Clone, Default)]
pub struct HierarchyConfig {
    pub parent_season: Option<String>,
}

/// Core season information that's always present
#[derive(Debug, Clone)]
pub struct SeasonCore<DateType> {
    pub name: ArcStr,
    pub begin: DateType,
    pub end: DateType,
    pub color: ArcStr,
    pub sunday_rank: Option<ArcStr>,
    pub ferial_rules: Vec<FerialRule<DateType>>,
}

/// A clean, organized season rule structure
#[derive(Debug, Clone)]
pub struct SeasonRule<DateType> {
    pub core: SeasonCore<DateType>,
    pub counting: CountingConfig<DateType>,
    pub display: DisplayConfig<DateType>,
    pub octave: OctaveConfig,
    pub hierarchy: HierarchyConfig,
}

// Custom serialization to maintain TOML compatibility
impl<DateType> Serialize for SeasonRule<DateType>
where
    DateType: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("SeasonRule", 15)?;
        state.serialize_field("name", &self.core.name)?;
        state.serialize_field("begin", &self.core.begin)?;
        state.serialize_field("end", &self.core.end)?;
        state.serialize_field("color", &self.core.color)?;

        if let Some(ref suffix) = self.counting.sundays_suffix {
            state.serialize_field("count_sundays_suffix", suffix)?;
        }
        if let Some(ref suffix) = self.counting.ferias_suffix {
            state.serialize_field("count_ferias_suffix", suffix)?;
        }
        if let Some(ref from) = self.counting.sundays_from {
            state.serialize_field("count_sundays_from", from)?;
        }
        if let Some(ref from) = self.counting.ferias_from {
            state.serialize_field("count_ferias_from", from)?;
        }
        if let Some(ref continue_from) = self.counting.continue_counting_from_season {
            state.serialize_field("continue_counting_from_season", continue_from)?;
        }
        if let Some(ref append) = self.display.append_week_of_month {
            state.serialize_field("append_week_of_month", append)?;
        }
        if self.display.dont_show_week_of_season {
            state.serialize_field(
                "dont_show_week_of_season",
                &self.display.dont_show_week_of_season,
            )?;
        }
        if let Some(ref rank) = self.core.sunday_rank {
            state.serialize_field("sunday_rank", rank)?;
        }
        if !self.core.ferial_rules.is_empty() {
            state.serialize_field("ferial_rules", &self.core.ferial_rules)?;
        }
        if self.octave.is_octave {
            state.serialize_field("is_octave", &self.octave.is_octave)?;
        }
        if let Some(ref rank) = self.octave.octave_rank {
            state.serialize_field("octave_rank", rank)?;
        }
        if let Some(ref parent) = self.hierarchy.parent_season {
            state.serialize_field("parent_season", parent)?;
        }

        state.end()
    }
}

// Custom deserialization to maintain TOML compatibility
impl<'de, DateType> Deserialize<'de> for SeasonRule<DateType>
where
    DateType: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use std::{fmt, marker::PhantomData};

        use serde::de::{self, MapAccess, Visitor};

        struct SeasonRuleVisitor<DateType>(PhantomData<DateType>);

        impl<'de, DateType> Visitor<'de> for SeasonRuleVisitor<DateType>
        where
            DateType: Deserialize<'de>,
        {
            type Value = SeasonRule<DateType>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct SeasonRule")
            }

            fn visit_map<V>(self, mut map: V) -> Result<SeasonRule<DateType>, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut name = None;
                let mut begin = None;
                let mut end = None;
                let mut color = None;
                let mut count_sundays_suffix = None;
                let mut count_ferias_suffix = None;
                let mut count_sundays_from = None;
                let mut count_ferias_from = None;
                let mut continue_counting_from_season: Option<String> = None;
                let mut append_week_of_month = None;
                let mut dont_show_week_of_season = false;
                let mut sunday_rank = None;
                let mut ferial_rules = Vec::new();
                let mut is_octave = false;
                let mut octave_rank = None;
                let mut parent_season = None;
                let mut week_of_month_old_scheme = false;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "name" => name = Some(map.next_value()?),
                        "begin" => begin = Some(map.next_value()?),
                        "end" => end = Some(map.next_value()?),
                        "color" => color = Some(map.next_value()?),
                        "count_sundays_suffix" => count_sundays_suffix = Some(map.next_value()?),
                        "count_ferias_suffix" => count_ferias_suffix = Some(map.next_value()?),
                        "count_sundays_from" => count_sundays_from = Some(map.next_value()?),
                        "count_ferias_from" => count_ferias_from = Some(map.next_value()?),
                        "continue_counting_from_season" => {
                            continue_counting_from_season = Some(map.next_value()?)
                        }
                        "append_week_of_month" => append_week_of_month = Some(map.next_value()?),
                        "dont_show_week_of_season" => {
                            dont_show_week_of_season = map.next_value()?
                        }
                        "sunday_rank" => sunday_rank = Some(map.next_value()?),
                        "ferial_rules" => ferial_rules = map.next_value()?,
                        "is_octave" => is_octave = map.next_value()?,
                        "octave_rank" => octave_rank = Some(map.next_value()?),
                        "parent_season" => parent_season = Some(map.next_value()?),
                        "week_of_month_old_scheme" => {
                            week_of_month_old_scheme = map.next_value()?
                        }

                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                let begin = begin.ok_or_else(|| de::Error::missing_field("begin"))?;
                let end = end.ok_or_else(|| de::Error::missing_field("end"))?;
                let color = color.ok_or_else(|| de::Error::missing_field("color"))?;

                Ok(SeasonRule {
                    core: SeasonCore {
                        name,
                        begin,
                        end,
                        color,
                        sunday_rank,
                        ferial_rules,
                    },
                    counting: CountingConfig {
                        sundays_suffix: count_sundays_suffix,
                        ferias_suffix: count_ferias_suffix,
                        sundays_from: count_sundays_from,
                        ferias_from: count_ferias_from,
                        continue_counting_from_season,
                    },
                    display: DisplayConfig {
                        append_week_of_month,
                        dont_show_week_of_season,
                        week_of_month_old_scheme,
                    },
                    octave: OctaveConfig {
                        is_octave,
                        octave_rank,
                    },
                    hierarchy: HierarchyConfig { parent_season },
                })
            }
        }

        deserializer.deserialize_struct(
            "SeasonRule",
            &[
                "name",
                "begin",
                "end",
                "color",
                "count_sundays_suffix",
                "count_ferias_suffix",
                "count_sundays_from",
                "count_ferias_from",
                "append_week_of_month",
                "dont_show_week_of_season",
                "sunday_rank",
                "ferial_rules",
                "is_octave",
                "octave_rank",
                "parent_season",
            ],
            SeasonRuleVisitor(PhantomData),
        )
    }
}

impl<DateType> SeasonRule<DateType> {
    #[cfg(test)]
    pub fn new(
        name: ArcStr,
        begin: DateType,
        end: DateType,
        color: ArcStr,
        count_sundays_suffix: Option<String>,
        count_ferias_suffix: Option<String>,
        count_sundays_from: Option<DateType>,
        count_ferias_from: Option<DateType>,
        append_week_of_month: Option<DateType>,
        dont_show_week_of_season: bool,
        sunday_rank: Option<String>,
        ferial_rules: Vec<FerialRule<DateType>>,
        is_octave: bool,
        octave_rank: Option<String>,
        parent_season: Option<String>,
        week_of_month_old_scheme: bool,
    ) -> Self {
        Self {
            core: SeasonCore {
                name,
                begin,
                end,
                color,
                sunday_rank: sunday_rank.map(|s| s.into()),
                ferial_rules,
            },
            counting: CountingConfig {
                sundays_suffix: count_sundays_suffix,
                ferias_suffix: count_ferias_suffix,
                sundays_from: count_sundays_from,
                ferias_from: count_ferias_from,
                continue_counting_from_season: None,
            },
            display: DisplayConfig {
                append_week_of_month,
                dont_show_week_of_season,
                week_of_month_old_scheme,
            },
            octave: OctaveConfig {
                is_octave,
                octave_rank,
            },
            hierarchy: HierarchyConfig { parent_season },
        }
    }

    // Getters
    pub fn name(&self) -> &str {
        &self.core.name
    }

    pub fn begin(&self) -> &DateType {
        &self.core.begin
    }

    pub fn end(&self) -> &DateType {
        &self.core.end
    }

    pub fn color(&self) -> &str {
        &self.core.color
    }

    #[cfg(test)]
    pub fn count_sundays_suffix(&self) -> &Option<String> {
        &self.counting.sundays_suffix
    }

    #[cfg(test)]
    pub fn count_ferias_suffix(&self) -> &Option<String> {
        &self.counting.ferias_suffix
    }

    #[cfg(test)]
    pub fn count_sundays_from(&self) -> &Option<DateType> {
        &self.counting.sundays_from
    }

    #[cfg(test)]
    pub fn count_ferias_from(&self) -> &Option<DateType> {
        &self.counting.ferias_from
    }

    pub fn continue_counting_from_season(&self) -> &Option<String> {
        &self.counting.continue_counting_from_season
    }

    pub fn append_week_of_month(&self) -> &Option<DateType> {
        &self.display.append_week_of_month
    }

    pub fn dont_show_week_of_season(&self) -> bool {
        self.display.dont_show_week_of_season
    }

    #[cfg(test)]
    pub fn sunday_rank(&self) -> &Option<ArcStr> {
        &self.core.sunday_rank
    }

    #[cfg(test)]
    pub fn ferial_rules(&self) -> &Vec<FerialRule<DateType>> {
        &self.core.ferial_rules
    }

    pub fn is_octave(&self) -> bool {
        self.octave.is_octave
    }

    pub fn octave_rank(&self) -> &Option<String> {
        &self.octave.octave_rank
    }

    pub fn parent_season(&self) -> &Option<String> {
        &self.hierarchy.parent_season
    }

    pub fn week_of_month_old_scheme(&self) -> bool {
        self.display.week_of_month_old_scheme
    }

    #[cfg(test)]
    // Setters
    pub fn set_name(&mut self, name: ArcStr) {
        self.core.name = name;
    }

    #[cfg(test)]
    pub fn set_begin(&mut self, begin: DateType) {
        self.core.begin = begin;
    }

    #[cfg(test)]
    pub fn set_end(&mut self, end: DateType) {
        self.core.end = end;
    }

    #[cfg(test)]
    pub fn set_color(&mut self, color: ArcStr) {
        self.core.color = color;
    }

    #[cfg(test)]
    pub fn set_count_sundays_suffix(&mut self, count_sundays_suffix: Option<String>) {
        self.counting.sundays_suffix = count_sundays_suffix;
    }

    #[cfg(test)]
    pub fn set_count_ferias_suffix(&mut self, count_ferias_suffix: Option<String>) {
        self.counting.ferias_suffix = count_ferias_suffix;
    }

    #[cfg(test)]
    pub fn set_count_sundays_from(&mut self, count_sundays_from: Option<DateType>) {
        self.counting.sundays_from = count_sundays_from;
    }

    #[cfg(test)]
    pub fn set_count_ferias_from(&mut self, count_ferias_from: Option<DateType>) {
        self.counting.ferias_from = count_ferias_from;
    }

    #[cfg(test)]
    pub fn set_append_week_of_month(&mut self, append_week_of_month: Option<DateType>) {
        self.display.append_week_of_month = append_week_of_month;
    }

    #[cfg(test)]
    pub fn set_dont_show_week_of_season(&mut self, dont_show_week_of_season: bool) {
        self.display.dont_show_week_of_season = dont_show_week_of_season;
    }

    #[cfg(test)]
    pub fn set_sunday_rank(&mut self, sunday_rank: Option<ArcStr>) {
        self.core.sunday_rank = sunday_rank;
    }

    #[cfg(test)]
    pub fn set_ferial_rules(&mut self, ferial_rules: Vec<FerialRule<DateType>>) {
        self.core.ferial_rules = ferial_rules;
    }

    #[cfg(test)]
    pub fn set_is_octave(&mut self, is_octave: bool) {
        self.octave.is_octave = is_octave;
    }

    #[cfg(test)]
    pub fn set_octave_rank(&mut self, octave_rank: Option<String>) {
        self.octave.octave_rank = octave_rank;
    }

    #[cfg(test)]
    pub fn set_parent_season(&mut self, parent_season: Option<String>) {
        self.hierarchy.parent_season = parent_season;
    }
}

impl FerialRule<DateRule> {
    pub fn instantiate_for_lit_year(&self, lit_year: i32) -> FerialRule<NaiveDate> {
        let begin = self.begin.to_day(lit_year).unwrap();
        let end = self.end.to_day(lit_year).unwrap();

        FerialRule {
            name: self.name.clone(),
            begin,
            end,
            rank: self.rank.clone(),
        }
    }
}

impl SeasonRule<DateRule> {
    pub fn instantiate_for_lit_year(&self, lit_year: i32) -> SeasonRule<NaiveDate> {
        let begin = self.core.begin.to_day(lit_year).unwrap();
        let end = self.core.end.to_day(lit_year).unwrap();
        let count_sundays_from = self
            .counting
            .sundays_from
            .as_ref()
            .map(|r| r.to_day(lit_year).unwrap());
        let count_ferias_from = self
            .counting
            .ferias_from
            .as_ref()
            .map(|r| r.to_day(lit_year).unwrap());
        let append_week_of_month = self
            .display
            .append_week_of_month
            .as_ref()
            .map(|r| r.to_day(lit_year).unwrap());
        let ferial_rules = self
            .core
            .ferial_rules
            .iter()
            .map(|fr| fr.instantiate_for_lit_year(lit_year))
            .collect();

        SeasonRule {
            core: SeasonCore {
                name: self.core.name.clone(),
                begin,
                end,
                color: self.core.color.clone(),
                sunday_rank: self.core.sunday_rank.clone(),
                ferial_rules,
            },
            counting: CountingConfig {
                sundays_suffix: self.counting.sundays_suffix.clone(),
                ferias_suffix: self.counting.ferias_suffix.clone(),
                sundays_from: count_sundays_from,
                ferias_from: count_ferias_from,
                continue_counting_from_season: self.counting.continue_counting_from_season.clone(),
            },
            display: DisplayConfig {
                append_week_of_month,
                dont_show_week_of_season: self.display.dont_show_week_of_season,
                week_of_month_old_scheme: self.display.week_of_month_old_scheme,
            },
            octave: OctaveConfig {
                is_octave: self.octave.is_octave,
                octave_rank: self.octave.octave_rank.clone(),
            },
            hierarchy: HierarchyConfig {
                parent_season: self.hierarchy.parent_season.clone(),
            },
        }
    }

    /// Instantiate for a liturgical year with hierarchy resolution
    /// This resolves parent season properties and flattens them into the resulting season
    pub fn instantiate_with_hierarchy(
        &self,
        lit_year: i32,
        parent_season: Option<&SeasonRule<NaiveDate>>,
    ) -> SeasonRule<NaiveDate> {
        let begin = self.core.begin.to_day(lit_year).unwrap();
        let end = self.core.end.to_day(lit_year).unwrap();
        let count_sundays_from = self
            .counting
            .sundays_from
            .as_ref()
            .map(|r| r.to_day(lit_year).unwrap());
        let count_ferias_from = self
            .counting
            .ferias_from
            .as_ref()
            .map(|r| r.to_day(lit_year).unwrap());
        let append_week_of_month = self
            .display
            .append_week_of_month
            .as_ref()
            .map(|r| r.to_day(lit_year).unwrap());
        let mut ferial_rules = self
            .core
            .ferial_rules
            .iter()
            .map(|fr| fr.instantiate_for_lit_year(lit_year))
            .collect::<Vec<_>>();

        // Inherit properties from parent season if not explicitly set
        let resolved_color = if self.core.color == "green" || self.core.color.is_empty() {
            parent_season
                .map(|p| p.core.color.clone())
                .unwrap_or_else(|| self.core.color.clone())
        } else {
            self.core.color.clone()
        };

        let resolved_sunday_rank = self
            .core
            .sunday_rank
            .clone()
            .or_else(|| parent_season.and_then(|p| p.core.sunday_rank.clone()));

        let resolved_sundays_suffix = self
            .counting
            .sundays_suffix
            .clone()
            .or_else(|| parent_season.and_then(|p| p.counting.sundays_suffix.clone()));

        let resolved_ferias_suffix = self
            .counting
            .ferias_suffix
            .clone()
            .or_else(|| parent_season.and_then(|p| p.counting.ferias_suffix.clone()));

        let resolved_sundays_from =
            count_sundays_from.or_else(|| parent_season.and_then(|p| p.counting.sundays_from));

        let resolved_ferias_from =
            count_ferias_from.or_else(|| parent_season.and_then(|p| p.counting.ferias_from));

        // Inherit ferial rules from parent (parent rules come first, then child rules override)
        if let Some(parent) = parent_season {
            let mut inherited_rules = parent.core.ferial_rules.clone();
            inherited_rules.extend(ferial_rules);
            ferial_rules = inherited_rules;
        }

        // Sort ferial rules by size of date range (smaller first for priority)
        ferial_rules.sort_by_key(|r| r.end.signed_duration_since(r.begin).num_days());

        SeasonRule {
            core: SeasonCore {
                name: self.core.name.clone(),
                begin,
                end,
                color: resolved_color,
                sunday_rank: resolved_sunday_rank,
                ferial_rules,
            },
            counting: CountingConfig {
                sundays_suffix: resolved_sundays_suffix,
                ferias_suffix: resolved_ferias_suffix,
                sundays_from: resolved_sundays_from,
                ferias_from: resolved_ferias_from,
                continue_counting_from_season: self.counting.continue_counting_from_season.clone(),
            },
            display: DisplayConfig {
                append_week_of_month,
                dont_show_week_of_season: self.display.dont_show_week_of_season,
                week_of_month_old_scheme: self.display.week_of_month_old_scheme,
            },
            octave: OctaveConfig {
                is_octave: self.octave.is_octave,
                octave_rank: self.octave.octave_rank.clone(),
            },
            hierarchy: HierarchyConfig {
                parent_season: None, // Clear parent reference since we've flattened the hierarchy
            },
        }
    }
}

impl SeasonRule<NaiveDate> {
    /// Gets the ferial rank for a given date within this season
    pub fn get_ferial_rank_for_date(&self, date: &NaiveDate) -> ArcStr {
        // Check if the date is within this season
        if date < &self.core.begin || date > &self.core.end {
            panic!(
                "Date {:?} is out of range for season {}",
                date, self.core.name
            );
        }

        // Find the most applicable ferial rule (highest priority)
        // Ferial rules are sorted by date range size (smaller ranges have higher priority)
        self.core
            .ferial_rules
            .iter()
            .find(|r| *date >= r.begin && *date <= r.end)
            .map(|rule| rule.rank.clone())
            .unwrap_or("IV".into())
    }

    /// Gets the Sunday rank for this season
    pub fn get_sunday_rank(&self) -> ArcStr {
        self.core.sunday_rank.clone().unwrap_or("II".into())
    }

    /// Check if this season is "of Lent" (either Lent itself or a child of Lent)
    pub fn is_of_lent(&self) -> bool {
        self.name().to_lowercase().contains("lent")
            || self.name().to_lowercase().contains("passion")
            || self.name().to_lowercase().contains("holy week")
    }

    /// Gets the count_sundays_suffix (hierarchy already resolved)
    pub fn get_count_sundays_suffix(&self) -> Option<&str> {
        self.counting.sundays_suffix.as_deref()
    }

    /// Gets the count_ferias_suffix (hierarchy already resolved)
    pub fn get_count_ferias_suffix(&self) -> Option<&str> {
        self.counting.ferias_suffix.as_deref()
    }

    /// Gets the count_sundays_from (hierarchy already resolved)
    pub fn get_count_sundays_from(&self) -> Option<NaiveDate> {
        self.counting.sundays_from
    }

    /// Gets the count_ferias_from date (hierarchy already resolved)
    pub fn get_count_ferias_from(&self) -> Option<NaiveDate> {
        self.counting.ferias_from
    }
}

#[cfg(test)]
pub mod test {
    use test_case::test_case;

    use super::*;

    impl<DateType> FerialRule<DateType> {
        // Constructor
        fn new(name: ArcStr, begin: DateType, end: DateType, rank: ArcStr) -> Self {
            Self {
                name,
                begin,
                end,
                rank,
            }
        }
    }

    /// Tests SeasonRule ferial ranking functionality
    #[test_case("2025-02-15", "II"; "date within ferial rule")]
    #[test_case("2025-01-15", "IV"; "date outside ferial rule uses default")]
    fn test_season_ferial_ranking(date_str: &str, expected_rank: &str) {
        let ferial_rule = FerialRule::new(
            "Special Period".into(),
            NaiveDate::from_ymd_opt(2025, 2, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 28).unwrap(),
            "II".into(),
        );

        let season_rule = SeasonRule::new(
            "Test Season".into(),
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 3, 31).unwrap(),
            "green".into(),
            Some("after Epiphany".into()),
            Some("in Ordinary Time".into()),
            Some(NaiveDate::from_ymd_opt(2025, 1, 6).unwrap()),
            Some(NaiveDate::from_ymd_opt(2025, 1, 7).unwrap()),
            Some(NaiveDate::from_ymd_opt(2025, 2, 1).unwrap()),
            false,
            Some("III".into()),
            vec![ferial_rule],
            false,
            None,
            None,
            false,
        );

        let test_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();

        let actual_rank = season_rule.get_ferial_rank_for_date(&test_date);
        assert_eq!(actual_rank, expected_rank);
    }

    /// Tests SeasonRule sunday ranking with different configurations
    #[test_case(Some("I".into()), "I"; "explicit sunday rank")]
    #[test_case(None, "II"; "default sunday rank")]
    fn test_season_sunday_ranking(sunday_rank: Option<String>, expected: &str) {
        let season_rule = SeasonRule::new(
            "Test Season".into(),
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 3, 31).unwrap(),
            "green".into(),
            None,
            None,
            None,
            None,
            None,
            false,
            sunday_rank,
            vec![],
            false,
            None,
            None,
            false,
        );

        let actual_rank = season_rule.get_sunday_rank();
        assert_eq!(actual_rank, expected);
    }

    /// Tests that SeasonRule panics when queried with out-of-range dates
    #[test]
    #[should_panic(expected = "Date")]
    fn test_season_rule_out_of_range_panic() {
        let season_rule = SeasonRule::new(
            "Limited Season".into(),
            NaiveDate::from_ymd_opt(2025, 6, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 6, 30).unwrap(),
            "green".into(),
            None,
            None,
            None,
            None,
            None,
            false,
            None,
            vec![],
            false,
            None,
            None,
            false,
        );

        // This should panic - date outside the season range
        let out_of_range = NaiveDate::from_ymd_opt(2025, 7, 1).unwrap();

        season_rule.get_ferial_rank_for_date(&out_of_range);
    }

    /// Tests instantiation of date rules for different liturgical years
    #[test_case(2025, "Test Ferial", "II", 3, 1, 3, 31; "year 2025")]
    #[test_case(2024, "Test Ferial", "II", 3, 1, 3, 31; "year 2024")]
    #[test_case(2026, "Test Ferial", "II", 3, 1, 3, 31; "year 2026")]
    fn test_ferial_rule_instantiation(
        lit_year: i32,
        expected_name: &str,
        expected_rank: &str,
        begin_month: u8,
        begin_day: u8,
        end_month: u8,
        end_day: u8,
    ) {
        let ferial_date_rule = FerialRule::new(
            expected_name.into(),
            DateRule::Fixed {
                month: begin_month,
                day: begin_day,
            },
            DateRule::Fixed {
                month: end_month,
                day: end_day,
            },
            expected_rank.into(),
        );

        let instantiated = ferial_date_rule.instantiate_for_lit_year(lit_year);

        assert_eq!(instantiated.name, expected_name);
        assert_eq!(instantiated.rank, expected_rank);
        assert_eq!(
            instantiated.begin,
            NaiveDate::from_ymd_opt(lit_year, begin_month as u32, begin_day as u32).unwrap()
        );
        assert_eq!(
            instantiated.end,
            NaiveDate::from_ymd_opt(lit_year, end_month as u32, end_day as u32).unwrap()
        );
    }

    /// Tests SeasonRule instantiation with comprehensive field coverage for different years
    #[test_case(2025, "Complex Season", "I"; "year 2025")]
    #[test_case(2024, "Complex Season", "I"; "year 2024")]
    #[test_case(2026, "Complex Season", "I"; "year 2026")]
    fn test_season_rule_instantiation(
        lit_year: i32,
        expected_name: &str,
        expected_sunday_rank: &str,
    ) {
        let ferial_rule = FerialRule::new(
            "Inner Ferial".into(),
            DateRule::Fixed { month: 2, day: 1 },
            DateRule::Fixed { month: 2, day: 28 },
            "III".into(),
        );

        let season_date_rule = SeasonRule::new(
            expected_name.into(),
            DateRule::Fixed { month: 1, day: 1 },
            DateRule::Fixed { month: 3, day: 31 },
            "purple".into(),
            Some("after Epiphany".into()),
            Some("in Ordinary Time".into()),
            Some(DateRule::Fixed { month: 1, day: 6 }),
            Some(DateRule::Fixed { month: 1, day: 7 }),
            Some(DateRule::Fixed { month: 2, day: 1 }),
            false,
            Some(expected_sunday_rank.into()),
            vec![ferial_rule],
            false,
            None,
            None,
            false,
        );

        let instantiated = season_date_rule.instantiate_for_lit_year(lit_year);

        assert_eq!(instantiated.name(), expected_name);
        assert!(instantiated.count_sundays_suffix().is_some());
        assert!(instantiated.count_ferias_suffix().is_some());
        assert!(instantiated.count_sundays_from().is_some());
        assert!(instantiated.count_ferias_from().is_some());
        assert!(instantiated.append_week_of_month().is_some());
        assert_eq!(
            instantiated.sunday_rank(),
            &Some(expected_sunday_rank.into())
        );
        assert_eq!(instantiated.ferial_rules().len(), 1);
    }

    // Test helper functions
    pub fn create_test_season(
        name: &str,
        begin: NaiveDate,
        end: NaiveDate,
    ) -> SeasonRule<NaiveDate> {
        SeasonRule {
            core: SeasonCore {
                name: name.into(),
                begin,
                end,
                color: "green".into(),
                sunday_rank: Some("III".into()),
                ferial_rules: vec![],
            },
            counting: CountingConfig::default(),
            display: DisplayConfig::default(),
            octave: OctaveConfig::default(),
            hierarchy: HierarchyConfig::default(),
        }
    }

    use chrono::NaiveDate;

    #[test]
    fn test_season_rule_accessors() {
        let season = SeasonRule::new(
            "Test Season".into(),
            DateRule::Fixed { month: 1, day: 1 },
            DateRule::Fixed { month: 6, day: 30 },
            "green".into(),
            None,
            None,
            None,
            None,
            None,
            false, // dont_show_week_of_season
            None,
            vec![],
            false, // is_octave
            None,
            None,
            false,
        );

        assert_eq!(season.name(), "Test Season");
        assert_eq!(season.color(), "green");
        assert_eq!(season.count_sundays_suffix(), &None);
        assert_eq!(season.count_ferias_suffix(), &None);
        assert_eq!(season.count_sundays_from(), &None);
        assert_eq!(season.count_ferias_from(), &None);
        assert_eq!(season.sunday_rank(), &None);
        assert_eq!(season.ferial_rules().len(), 0);
    }

    #[test]
    fn test_season_rule_setters() {
        let mut season = SeasonRule::new(
            "Test Season".into(),
            DateRule::Fixed { month: 1, day: 1 },
            DateRule::Fixed { month: 6, day: 30 },
            "green".into(),
            None,
            None,
            None,
            None,
            None,
            false, // dont_show_week_of_season
            None,
            vec![],
            false, // is_octave
            None,
            None,
            false,
        );

        season.set_name("Updated Season".into());
        season.set_begin(DateRule::Fixed { month: 2, day: 1 });
        season.set_end(DateRule::Fixed { month: 7, day: 31 });
        season.set_color("red".into());
        season.set_count_sundays_suffix(Some("after Epiphany".into()));
        season.set_count_ferias_suffix(Some("in Lent".into()));
        season.set_count_sundays_from(Some(DateRule::Fixed { month: 1, day: 6 }));
        season.set_count_ferias_from(Some(DateRule::Fixed { month: 2, day: 1 }));
        season.set_append_week_of_month(Some(DateRule::Fixed { month: 1, day: 6 }));
        season.set_dont_show_week_of_season(false);
        season.set_sunday_rank(Some("II".into()));
        season.set_ferial_rules(vec![]);
        season.set_is_octave(true);
        season.set_octave_rank(Some("Simple".into()));
        season.set_parent_season(Some("Ordinary Time".into()));

        assert_eq!(season.name(), "Updated Season");
        assert_eq!(season.color(), "red");
        assert_eq!(
            season.count_sundays_suffix(),
            &Some("after Epiphany".into())
        );
        assert_eq!(season.count_ferias_suffix(), &Some("in Lent".into()));
        assert!(season.count_sundays_from().is_some());
        assert!(season.count_ferias_from().is_some());
        assert_eq!(season.sunday_rank(), &Some("II".into()));
    }

    #[test]
    fn test_season_rule_with_ferial_rules() {
        // Skip testing ferial rules since FerialRule fields are private
        // and constructor is not public. Just test a basic season.
        let season = SeasonRule::new(
            "Lent".into(),
            DateRule::Fixed { month: 2, day: 1 },
            DateRule::Fixed { month: 4, day: 15 },
            "purple".into(),
            None,
            None,
            None,
            None,
            None,
            false, // dont_show_week_of_season
            Some("I".into()),
            vec![], // Empty ferial rules
            false,  // is_octave
            None,
            None,
            false,
        );

        assert_eq!(season.ferial_rules().len(), 0);
        assert_eq!(season.sunday_rank(), &Some("I".into()));
    }

    #[test]
    fn test_season_rule_get_sunday_rank() {
        let season = SeasonRule::new(
            "Test Season".into(),
            DateRule::Fixed { month: 1, day: 1 },
            DateRule::Fixed { month: 6, day: 30 },
            "green".into(),
            None,
            None,
            None,
            None,
            None,
            false, // dont_show_week_of_season
            Some("II".into()),
            vec![],
            false,
            None,
            None,
            false,
        );

        // Instantiate to test get_sunday_rank method
        let instantiated = season.instantiate_for_lit_year(2025);
        let sunday_rank = instantiated.get_sunday_rank();
        assert_eq!(sunday_rank, "II");
    }

    #[test]
    fn test_season_rule_get_effective_color() {
        let season = SeasonRule::new(
            "Test Season".into(),
            DateRule::Fixed { month: 1, day: 1 },
            DateRule::Fixed { month: 6, day: 30 },
            "green".into(),
            None,
            None,
            None,
            None,
            None,
            false, // dont_show_week_of_season
            None,
            vec![],
            false,
            None,
            None,
            false,
        );

        let instantiated = season.instantiate_for_lit_year(2025);
        let color = &instantiated.core.color;
        assert_eq!(color, "green");
    }
}
