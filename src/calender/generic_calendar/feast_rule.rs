use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::calender::{
    feast_rank::FeastRank, DateRule, DayType, LiturgicalContext, LiturgicalUnit,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeastRule<DateType> {
    pub name: String,
    pub date_rule: DateType,
    pub rank: Option<String>,
    #[serde(default)]
    pub of_our_lord: bool,
    pub day_type: Option<DayType>,
    pub color: String,
    #[serde(default)]
    pub titles: Vec<String>,
    #[serde(default)]
    pub movable: bool,
}

impl<DateType> FeastRule<DateType> {
    pub fn into_liturgical_unit<R>(self, date: NaiveDate) -> LiturgicalUnit
    where
        R: FeastRank,
    {
        let rank = self.get_feastrank::<R>().get_rank_string();
        LiturgicalUnit {
            desc: self.to_string(),
            rank,
            date,
            color: self.color,
        }
    }

    /// Get the effective FeastRank, either from the new field or converted from legacy fields
    pub fn get_feastrank<R>(&self) -> R
    where
        R: FeastRank,
    {
        // Convert from legacy fields
        let rank = self.rank.as_deref().unwrap_or("III");
        let day_type = self.day_type.as_ref().unwrap_or(&DayType::Feast);

        let mut context = LiturgicalContext::new().feast(self.name.clone());

        if self.movable {
            context = context.movable();
        }

        if self.of_our_lord {
            context = context.of_our_lord();
        }

        R::new_with_context(rank, day_type, &context)
    }
}

impl FeastRule<DateRule> {
    pub fn instantiate_for_lit_year_with_advent(&self, lit_year: i32) -> FeastRule<NaiveDate> {
        // For fixed dates that occur on or after the NEXT Advent (end of liturgical year),
        // they belong to the previous liturgical year
        let mut movable = true;
        let calendar_year = match &self.date_rule {
            DateRule::Fixed { month, day } => {
                // Get the date in the current calendar year
                let current_year_date =
                    NaiveDate::from_ymd_opt(lit_year, *month as u32, *day as u32).unwrap();

                // Calculate when the NEXT Advent starts (end of this liturgical year)
                // Advent is the 4th Sunday before Christmas, so find the first Sunday of Advent for lit_year+1
                // But we need to be careful - we want Advent of the current calendar year, not liturgical year
                let next_advent_year = if *month >= 11 { lit_year } else { lit_year + 1 };

                // Find first Sunday of Advent for this calendar year
                // Use a simple approximation: Advent starts between Nov 27 and Dec 3
                let christmas = NaiveDate::from_ymd_opt(next_advent_year, 12, 25).unwrap();
                let mut advent_sunday = christmas - chrono::Duration::days(21); // Start with 3 weeks before
                while advent_sunday.weekday() != chrono::Weekday::Sun {
                    advent_sunday -= chrono::Duration::days(1);
                }
                movable = false;
                // If the feast date is on or after this Advent, it belongs to the previous liturgical year
                if current_year_date >= advent_sunday {
                    lit_year - 1
                } else {
                    lit_year
                }
            }
            _ => lit_year, // Non-fixed dates use the liturgical year as-is
        };

        let date = self.date_rule.to_day(calendar_year).unwrap();

        FeastRule {
            name: self.name.clone(),
            date_rule: date,
            rank: self.rank.clone(),
            of_our_lord: self.of_our_lord,
            day_type: self.day_type.clone(),
            color: self.color.clone(),
            titles: self.titles.clone(),
            movable,
        }
    }

    pub fn add_extensions_prefix(mut self, prefix: &str) -> Self {
        self.name = format!("{}: {}", prefix, self.name);
        self
    }
}

impl<T> std::fmt::Display for FeastRule<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let titles = if self.titles.is_empty() {
            "".to_string()
        } else {
            format!(", {}", self.titles.join(" and "))
        };
        write!(f, "{}{}", self.name, titles)
    }
}
