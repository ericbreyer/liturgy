use chrono::Datelike;
use serde::{Deserialize, Serialize};

use super::{CalendarTypeProvider, GenericCalendar};
use crate::calender::{DateRule, feast_rank::FeastRankResolver};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalendarCycle {
    Sanctoral,
    Temporal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleFeast {
    pub name: String,
    pub description: String,
    pub date_rule: String,
    pub date: Option<String>,
    pub rank: String,
    pub color: String,
    pub titles: Vec<String>,
}

fn fixed_date_rule(rule: &DateRule) -> bool {
    match rule {
        DateRule::Fixed { .. } => true,
        DateRule::PreviousYear(rule)
        | DateRule::NextYear(rule)
        | DateRule::AvoidSunday { rule }
        | DateRule::OffsetDays { rule, .. } => fixed_date_rule(rule),
        DateRule::LeapYearConditional {
            leap_year_rule,
            non_leap_year_rule,
        } => fixed_date_rule(leap_year_rule) && fixed_date_rule(non_leap_year_rule),
        _ => false,
    }
}

impl<T: FeastRankResolver + CalendarTypeProvider> GenericCalendar<T> {
    /// Resolve source feast rules within a civil year, without precedence or transfers.
    #[must_use]
    pub fn cycle_feasts(&self, cycle: CalendarCycle, year: i32) -> Vec<CycleFeast> {
        let mut feasts: Vec<_> = self
            .feasts
            .iter()
            .filter(|feast| {
                let feast_cycle = self
                    .cycle_overrides
                    .get(&feast.name)
                    .copied()
                    .unwrap_or_else(|| {
                        if fixed_date_rule(&feast.date_rule) {
                            CalendarCycle::Sanctoral
                        } else {
                            CalendarCycle::Temporal
                        }
                    });
                feast_cycle == cycle
            })
            .map(|feast| {
                let date = [year, year - 1, year + 1]
                    .into_iter()
                    .filter_map(|rule_year| feast.date_rule.to_day(rule_year))
                    .find(|date| date.year() == year);
                CycleFeast {
                    name: feast.name.to_string(),
                    description: feast.to_string(),
                    date_rule: feast.date_rule.to_string(),
                    date: date.map(|date| date.to_string()),
                    rank: feast.get_feastrank::<T>().get_rank_string().to_string(),
                    color: feast.color.to_string(),
                    titles: feast.titles.clone(),
                }
            })
            .collect();
        feasts.sort_by(|left, right| left.date.cmp(&right.date).then(left.name.cmp(&right.name)));
        feasts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calender::feast_rank::FeastRankOf;

    #[test]
    fn source_cycles_preserve_suppressed_feasts_and_explicit_overrides() {
        let calendar = GenericCalendar::<FeastRankOf>::from_toml_str(
            r#"
name = "Test"
cal_type = "OrdinaryForm"
[[feasts]]
name = "Saint One"
date_rule = "Fixed(12,25)"
rank = "III"
color = "white"
[[feasts]]
name = "Christmas"
date_rule = "Fixed(12,25)"
rank = "I"
color = "white"
[[feasts]]
name = "Easter"
date_rule = "Easter"
rank = "I"
color = "white"
[cycle_overrides]
Christmas = "temporal"
"#,
        )
        .unwrap();
        let sanctoral = calendar.cycle_feasts(CalendarCycle::Sanctoral, 2026);
        assert_eq!(sanctoral.len(), 1);
        assert_eq!(sanctoral[0].name, "Saint One");
        assert_eq!(sanctoral[0].date.as_deref(), Some("2026-12-25"));
        assert_eq!(
            calendar.cycle_feasts(CalendarCycle::Temporal, 2026).len(),
            2
        );
    }

    #[test]
    fn previous_year_rules_resolve_to_requested_civil_year() {
        let calendar = GenericCalendar::<FeastRankOf>::from_toml_str(
            r#"
cal_type = "OrdinaryForm"
[[feasts]]
name = "Christmas"
date_rule = "PreviousYear(Fixed(12,25))"
color = "white"
"#,
        )
        .unwrap();
        assert_eq!(
            calendar.cycle_feasts(CalendarCycle::Sanctoral, 2026)[0]
                .date
                .as_deref(),
            Some("2026-12-25")
        );
    }
}
