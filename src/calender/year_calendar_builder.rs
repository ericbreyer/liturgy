use std::collections::HashMap;

use chrono::{Datelike, NaiveDate};

use crate::{
    calender::{
        DayType, LiturgicalContext, LiturgicalUnit, feast_rank::{BVMOnSaturdayResult, FeastRank}, generic_calendar::{CalendarType, FeastRule, SeasonRule}, year_calendar::{DayDescription, YearCalendar}
    },
    date_calc::{
        get_following_sunday, get_preceding_sunday, num_sundays_after_date_inclusive, num_weeks_after_date, to_month_string, to_roman_numeral
    },
};

#[derive(Debug, Clone)]
pub struct YearCalendarBuilder {
    pub year: i32,
    #[cfg(test)]
    pub name: String,
    pub seasons: Vec<SeasonRule<NaiveDate>>,
    pub feasts: HashMap<NaiveDate, Vec<FeastRule<NaiveDate>>>,
    pub first_advent: NaiveDate,
    pub next_first_advent: NaiveDate,
    pub calendar_type: CalendarType,
}

impl YearCalendarBuilder {
    pub fn generate_year_calendar<R>(&self) -> YearCalendar<R>
    where
        R: FeastRank,
    {
        let mut days = Vec::new();
        // The start date should be the first Sunday of Advent
        let start = self.first_advent;

        // The last day is the Saturday before the first Sunday of Advent of the current year
        let next_first_advent = self.next_first_advent;
        let end = next_first_advent.pred_opt().unwrap();

        let mut transfer: Option<(R, LiturgicalUnit)> = None;

        for date in start.iter_days().take_while(|&d| d <= end) {
            let season_desc = self.get_season_descriptor(&date);
            let season_rank: R = self.season_day_to_feast_rank(&date);
            let season_liturgical_unit = LiturgicalUnit {
                desc: season_desc.clone(),
                rank: season_rank.clone().get_rank_string(),
                date,
                color: self.get_season_color(&date),
            };

            let feast_competitors: Vec<_> = self
                .get_feasts_on_date(&date)
                .into_iter()
                .map(|f| (f.get_feastrank::<R>(), f.into_liturgical_unit::<R>(date)))
                .collect();

            let has_ferial_or_sunday = feast_competitors
                .iter()
                .any(|(r, _)| r.is_ferial_or_sunday_rank());

            let has_high_festival = feast_competitors.iter().any(|(r, _)| r.is_high_festial());

            let competitors: Vec<_> = feast_competitors
                .into_iter()
                // Add season rank if no ferial or sunday competitors exist
                .chain(
                    (!has_ferial_or_sunday)
                        .then(|| (season_rank.clone(), season_liturgical_unit.clone())),
                )
                // Add transfer if present and no high festival competitors exist
                .chain(
                    transfer
                        .clone()
                        .filter(|_| !has_high_festival)
                        .map(|(rank, unit)| (rank, unit.transfered())),
                )
                .collect();

            // Only consume the transfer if we actually used it
            if transfer.is_some() && !has_high_festival {
                transfer = None;
            }

            let mut result = R::resolve_conflicts(&competitors);

            // Add BVM on Saturday commemoration for ferial Saturdays
            let is_ferial_saturday =
                date.weekday() == chrono::Weekday::Sat;

            if is_ferial_saturday {
                match result.winner_rank.admits_bvm_on_saturday() {
                    BVMOnSaturdayResult::NotAdmitted => {}
                    BVMOnSaturdayResult::Admitted => {
                        // Add BVM on Saturday as a commemoration
                        result.winner.bvm_on_saturday();
                    }
                    BVMOnSaturdayResult::Commemorated => {
                        result
                            .commemorations
                            .push(LiturgicalUnit::bvm_on_saturday_commemoration::<R>(date));
                    }
                }
            }

            days.push(DayDescription {
                date,
                day_in_season: season_desc,
                day_rank: result.winner.rank.clone(),
                day: result.winner,
                commemorations: result.commemorations,
            });

            transfer = transfer.or(result.transferred);
        }
        YearCalendar {
            year: self.year,
            #[cfg(test)]
            name: self.name.clone(),
            days: days.into_boxed_slice(),
            __marker: std::marker::PhantomData,
        }
    }

    pub fn get_season_color(&self, date: &NaiveDate) -> String {
        let season = self.get_season(date);
        season.color().to_string()
    }

    pub fn get_season_descriptor(&self, date: &chrono::NaiveDate) -> String {
        let season = self.get_season(date);

        let weekday = date.weekday().number_from_monday();
        let feria = match weekday {
            6 => "Sabbato".to_owned(),
            7 => "Dominica".to_owned(),
            n => format!("Feria {}", to_roman_numeral((n + 1).try_into().unwrap())),
        };

        let week_ordinal = self.get_week_ordinal_for_season(season, date);

        let suffix = if weekday == 7 {
            season.get_count_sundays_suffix()
        } else {
            season.get_count_ferias_suffix()
        }
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("of {}", season.name()));

        let week_of_month = if let Some(lower_bound) = season.append_week_of_month().as_ref() {
            if lower_bound > date {
                "".to_string()
            } else {
                let preceding_sunday = get_preceding_sunday(*date);
                let month = preceding_sunday.month();
                let first_sunday_of_month = {
                    let first_of_month =
                        NaiveDate::from_ymd_opt(preceding_sunday.year(), month, 1).unwrap();
                    get_following_sunday(first_of_month)
                };
                let week_of_month =
                    num_sundays_after_date_inclusive(first_sunday_of_month, preceding_sunday);
                format!(" (Week {} of {})", week_of_month, to_month_string(month))
            }
        } else {
            "".to_string()
        };

        let week_ordinal_str = if season.dont_show_week_of_season() {
            "".to_string()
        } else if week_ordinal == 0 {
            "after start ".to_string()
        } else if weekday == 7 {
            format!("{} ", to_roman_numeral(week_ordinal))
        } else {
            format!("week {} ", to_roman_numeral(week_ordinal))
        };

        format!("{feria} {week_ordinal_str}{suffix}{week_of_month}")
    }

    pub fn get_season(&self, date: &NaiveDate) -> &SeasonRule<NaiveDate> {
        // Find the most specific season (smallest date range that contains the date)
        self.seasons
            .iter()
            .filter(|season| date >= season.begin() && date <= season.end())
            .min_by_key(|season| {
                // Calculate the duration of the season (smaller = more specific)
                season
                    .end()
                    .signed_duration_since(*season.begin())
                    .num_days()
            })
            .unwrap_or_else(|| {
                // If no season found, provide a fallback or create a default season
                eprintln!(
                    "Warning: No season found for date {}, using first season as fallback",
                    date
                );
                &self.seasons[0]
            })
    }

    pub fn season_day_to_feast_rank<R>(&self, date: &NaiveDate) -> R
    where
        R: FeastRank,
    {
        let season = self.get_season(date);

        let weekday = date.weekday().number_from_monday();
        let _feria = match weekday {
            6 => "Sabbato".to_owned(),
            7 => "Dominica".to_owned(),
            n => format!("Feria {}", to_roman_numeral((n + 1).try_into().unwrap())),
        };

        let _week_ordinal = self.get_week_ordinal_for_season(season, date);

        let _suffix = if weekday == 7 {
            season.get_count_sundays_suffix()
        } else {
            season.get_count_ferias_suffix()
        }
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("of {}", season.name()));

        let _week_of_month = if let Some(lower_bound) = season.append_week_of_month().as_ref() {
            if lower_bound > date {
                "".to_string()
            } else {
                let preceding_sunday = get_preceding_sunday(*date);
                let month = preceding_sunday.month();
                let first_sunday_of_month = {
                    let first_of_month =
                        NaiveDate::from_ymd_opt(preceding_sunday.year(), month, 1).unwrap();
                    get_following_sunday(first_of_month)
                };
                let week_of_month =
                    num_sundays_after_date_inclusive(first_sunday_of_month, preceding_sunday);
                format!(" (Week {} of m{})", week_of_month, month)
            }
        } else {
            "".to_string()
        };

        if date.weekday() == chrono::Weekday::Sun {
            let context = LiturgicalContext::new()
                .season(season.name())
                .of_lent(season.is_of_lent());
            if season.is_octave() {
                // Sunday within an octave becomes an octave day
                let rank = season.octave_rank().as_deref().unwrap_or("I");
                R::new_with_context(rank, &DayType::Octave, &context.also_sunday())
            } else {
                R::new_with_context(&season.get_sunday_rank(), &DayType::Sunday, &context)
            }
        } else {
            let context = LiturgicalContext::new()
                .season(season.name())
                .feast(self.get_season_descriptor(date))
                .of_lent(season.is_of_lent());
            if season.is_octave() {
                // Weekday within an octave becomes an octave day
                let rank = season.octave_rank().as_deref().unwrap_or("I");
                R::new_with_context(rank, &DayType::Octave, &context.also_ferial())
            } else {
                R::new_with_context(
                    &season.get_ferial_rank_for_date(date),
                    &DayType::Feria,
                    &context,
                )
            }
        }
    }

    pub fn get_feasts_on_date(&self, date: &NaiveDate) -> Vec<FeastRule<NaiveDate>> {
        self.feasts.get(date).cloned().unwrap_or_else(Vec::new)
    }

    /// Calculate week ordinal for a season, handling continuous counting from other seasons
    /// Calculate the total number of weeks in Ordinary Time for the liturgical year
    fn get_total_ordinary_time_weeks(&self) -> i32 {
        // Find both Ordinary Time seasons
        let before_lent = self
            .seasons
            .iter()
            .find(|s| s.name().contains("Ordinary Time") && s.name().contains("before"));
        let after_pentecost = self
            .seasons
            .iter()
            .find(|s| s.name().contains("Ordinary Time") && s.name().contains("after"));

        if let (Some(before), Some(after)) = (before_lent, after_pentecost) {
            // Count Sundays in both seasons
            let before_weeks = {
                let last_sunday = get_preceding_sunday(*before.end());
                let count_from = before.get_count_sundays_from().unwrap_or(*before.begin());
                if last_sunday >= count_from {
                    num_sundays_after_date_inclusive(count_from, last_sunday)
                } else {
                    0
                }
            };

            let after_weeks = {
                let last_sunday = get_preceding_sunday(*after.end());
                let count_from = after.get_count_sundays_from().unwrap_or(*after.begin());
                if last_sunday >= count_from {
                    num_sundays_after_date_inclusive(count_from, last_sunday)
                } else {
                    0
                }
            };

            before_weeks + after_weeks
        } else {
            34 // Default fallback
        }
    }

    fn get_week_ordinal_for_season(&self, season: &SeasonRule<NaiveDate>, date: &NaiveDate) -> i32 {
        let weekday = date.weekday().number_from_monday();

        // Check if this season continues counting from another season
        if let Some(ref_season_name) = season.continue_counting_from_season() {
            // Find the referenced season
            if let Some(ref_season) = self.seasons.iter().find(|s| s.name() == ref_season_name) {
                // Calculate the total weeks from the referenced season
                let ref_season_weeks = if weekday == 7 {
                    // For Sunday counting, get the last Sunday in the referenced season
                    let last_sunday_in_ref = get_preceding_sunday(*ref_season.end());
                    let count_from = ref_season
                        .get_count_sundays_from()
                        .unwrap_or(*ref_season.begin());
                    if last_sunday_in_ref >= count_from {
                        num_sundays_after_date_inclusive(count_from, last_sunday_in_ref)
                    } else {
                        0
                    }
                } else {
                    // For weekday counting, use the end date directly
                    let count_from = ref_season
                        .get_count_ferias_from()
                        .unwrap_or(*ref_season.begin());
                    if *ref_season.end() >= count_from {
                        num_weeks_after_date(count_from, *ref_season.end())
                    } else {
                        0
                    }
                };

                // Apply the 33/34 week adjustment ONLY for Ordinary Form calendars
                let week_adjustment = if self.calendar_type == CalendarType::OrdinaryForm {
                    let total_ot_weeks = self.get_total_ordinary_time_weeks();
                    if total_ot_weeks == 33 {
                        1 // Skip first week after Pentecost if 33 weeks total
                    } else {
                        0 // Continue normally if 34 weeks total
                    }
                } else {
                    0 // No adjustment for non-OF calendars
                };

                // Add the weeks in the current season
                let current_season_weeks = if weekday == 7 {
                    num_sundays_after_date_inclusive(
                        season.get_count_sundays_from().unwrap_or(*season.begin()),
                        *date,
                    )
                } else {
                    num_weeks_after_date(
                        season.get_count_ferias_from().unwrap_or(*season.begin()),
                        *date,
                    )
                };

                ref_season_weeks + current_season_weeks + week_adjustment
            } else {
                // Fallback if referenced season not found
                self.get_standard_week_ordinal(season, date)
            }
        } else {
            // Standard week counting for seasons without continuation
            self.get_standard_week_ordinal(season, date)
        }
    }

    /// Standard week counting logic
    fn get_standard_week_ordinal(&self, season: &SeasonRule<NaiveDate>, date: &NaiveDate) -> i32 {
        let weekday = date.weekday().number_from_monday();

        if weekday == 7 {
            num_sundays_after_date_inclusive(
                season.get_count_sundays_from().unwrap_or(*season.begin()),
                *date,
            )
        } else {
            num_weeks_after_date(
                season.get_count_ferias_from().unwrap_or(*season.begin()),
                *date,
            )
        }
    }
}

#[cfg(test)]
mod test {
    use chrono::NaiveDate;
    use test_case::test_case;

    use super::*;
    use crate::calender::{feast_rank::FeastRank62, generic_calendar::tests::*};

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
            calendar_type: CalendarType::OrdinaryForm,
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
            calendar_type: CalendarType::OrdinaryForm,
        };
        let test_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();
        let rank: FeastRank62 = year_calendar.season_day_to_feast_rank(&test_date);
        assert!(rank.is_ferial_or_sunday_rank());
    }
}
