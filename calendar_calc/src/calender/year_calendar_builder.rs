use std::collections::HashMap;

use chrono::{Datelike, NaiveDate};
use types::{ArcStr, DayDescription};

use crate::{
    calender::{
        feast_rank::{BVMOnSaturdayResult, FeastRankResolver},
        generic_calendar::{CalendarType, FeastRule, SeasonRule},
        year_calendar::YearCalendar,
        DayType, LiturgicalContext, LiturgicalUnit,
    },
    date_calc::{
        get_following_sunday, get_preceding_sunday, num_sundays_after_date_inclusive,
        num_weeks_after_date, to_month_string, to_roman_numeral,
    },
};

#[derive(Debug, Clone)]
pub struct YearCalendarBuilder {
    pub year: i32,
    #[cfg(test)]
    pub name: ArcStr,
    pub seasons: Vec<SeasonRule<NaiveDate>>,
    pub feasts: HashMap<NaiveDate, Vec<FeastRule<NaiveDate>>>,
    pub first_advent: NaiveDate,
    pub next_first_advent: NaiveDate,
    pub calendar_type: CalendarType,
    pub octaves: Vec<SeasonRule<NaiveDate>>,
}

impl YearCalendarBuilder {
    pub fn generate_year_calendar<R>(&self) -> YearCalendar<R::FeastRankDescriptor>
    where
        R: FeastRankResolver,
    {
        let mut days = Vec::new();
        // Diagnostic: print instantiated seasons and octaves to help debug range coverage
        // The start date should be the first Sunday of Advent
        let start = self.first_advent;

        // The last day is the Saturday before the first Sunday of Advent of the current year
        let next_first_advent = self.next_first_advent;
        let end = next_first_advent.pred_opt().unwrap();

        let mut transfer: Option<(R, LiturgicalUnit<R::FeastRankDescriptor>)> = None;

        for date in start.iter_days().take_while(|&d| d <= end) {
            let season_desc = self.get_season_descriptor(&date);
            // season rank should be based only on the season (do not let octaves replace season days)
            let season_rank: R = self.season_day_to_feast_rank(&date);
            let season_liturgical_unit = LiturgicalUnit {
                desc: season_desc.clone(),
                rank: season_rank.clone().descriptor(),
                date,
                color: self.get_season_color(&date),
            };

            // When the rank implementation opts into transferring vigils from Sunday to Saturday,
            // do not include Vigil-type feasts on the Sunday itself; they will be added as
            // transferred competitors on the previous Saturday instead.
            let feast_competitors: Vec<_> = {
                let feasts = self.get_feasts_on_date(&date);
                let feasts = if date.weekday() == chrono::Weekday::Sun
                    && R::transfers_vigil_from_sunday_to_saturday()
                {
                    feasts
                        .into_iter()
                        .filter(|f| {
                            !matches!(
                                f.day_type.as_ref().unwrap_or(&DayType::Feast),
                                DayType::Vigil
                            )
                        })
                        .collect::<Vec<_>>()
                } else {
                    feasts
                };

                feasts
                    .into_iter()
                    .map(|f| (f.get_feastrank::<R>(), f.into_liturgical_unit::<R>(date)))
                    .collect()
            };

            let has_ferial_or_sunday = feast_competitors
                .iter()
                .any(|(r, _)| r.is_ferial_or_sunday_rank());

            // If an octave exists on this date, create an octave competitor but do not replace the season
            let octave_competitors: Vec<(R, LiturgicalUnit<R::FeastRankDescriptor>)> = self
                .octaves
                .iter()
                .filter(|s| date >= *s.begin() && date <= *s.end())
                .map(|oct| {
                    let ctx = LiturgicalContext::new()
                        .season(oct.name())
                        .of_lent(oct.is_of_lent())
                        .feast(self.get_octave_descriptor(&date, oct))
                        .octave_day(date == *oct.end())
                        .first_3_days(date <= *oct.begin() + chrono::Duration::days(1));
                    let ctx = if date.weekday() == chrono::Weekday::Sun {
                        ctx.also_sunday()
                    } else {
                        ctx.also_ferial()
                    };
                    let rank = oct.octave_rank().as_deref().unwrap_or("I");
                    let r = R::new_with_context(rank, &DayType::Octave, &ctx);
                    let unit = LiturgicalUnit {
                        desc: self.get_octave_descriptor(&date, oct),
                        rank: r.clone().descriptor(),
                        date,
                        color: oct.color().into(),
                    };
                    (r, unit)
                })
                .collect();

            let has_high_festival = feast_competitors.iter().any(|(r, _)| r.is_high_festial())
                || octave_competitors.iter().any(|(r, _)| r.is_high_festial());

            let competitors: Vec<_> = feast_competitors
                .into_iter()
                // Add season rank if no ferial or sunday competitors exist
                .chain(
                    (!has_ferial_or_sunday)
                        .then(|| (season_rank.clone(), season_liturgical_unit.clone())),
                )
                // Add octave competitor if present (do not replace season)
                .chain(octave_competitors.into_iter())
                // Add transfer if present and no high festival competitors exist
                .chain(
                    transfer
                        .clone()
                        .filter(|_| !has_high_festival)
                        .map(|(rank, unit)| (rank, unit.transfered())),
                )
                // If the current date is a Saturday, and the feast-rank implementation
                // opts into transferring vigils that fall on Sunday, then look for
                // vigils defined on the following day (Sunday) and add them as transferred
                // competitors so they appear on Saturday.
                .chain(
                    (date.weekday() == chrono::Weekday::Sat
                        && R::transfers_vigil_from_sunday_to_saturday())
                    .then(|| {
                        // Look for feasts that occur on the following day which are vigils
                        let sunday = date + chrono::Duration::days(1);
                        self.get_feasts_on_date(&sunday)
                            .into_iter()
                            .filter(|f| {
                                matches!(
                                    f.day_type.as_ref().unwrap_or(&DayType::Feast),
                                    DayType::Vigil
                                )
                            })
                            .map(|f| {
                                let mut unit = f.clone().into_liturgical_unit::<R>(sunday);
                                // mark as transferred so the calendar builder treats it correctly
                                unit = unit.transfered();
                                // The transferred unit should occur on the previous Saturday
                                unit.date = date;
                                (f.get_feastrank::<R>(), unit)
                            })
                            .collect::<Vec<_>>()
                    })
                    .into_iter()
                    .flatten(),
                )
                .collect();

            // Only consume the transfer if we actually used it
            if transfer.is_some() && !has_high_festival {
                transfer = None;
            }

            let mut result = R::resolve_conflicts(&competitors).unwrap();

            // Add BVM on Saturday commemoration for ferial Saturdays
            let is_ferial_saturday = date.weekday() == chrono::Weekday::Sat;

            if is_ferial_saturday {
                match result.winner_rank.admits_bvm_on_saturday() {
                    BVMOnSaturdayResult::NotAdmitted => {}
                    BVMOnSaturdayResult::Admitted => {
                        // Add BVM on Saturday
                        bvm_on_saturday::<R>(&mut result.winner);
                    }
                    BVMOnSaturdayResult::Commemorated => {
                        result
                            .commemorations
                            .push(bvm_on_saturday_commemoration::<R>(date));
                    }
                    BVMOnSaturdayResult::OtherCommemorated => {
                        result.commemorations.push(result.winner.clone());
                        bvm_on_saturday::<R>(&mut result.winner);
                    }
                }
            }

            // if winner is a sunday in an octave, change its description to reflect that
            if let Some(oct) = self
                .octaves
                .iter()
                .find(|s| date >= *s.begin() && date < *s.end())
            {
                if date.weekday() == chrono::Weekday::Sun
                    && result.winner_rank.is_ferial_or_sunday_rank()
                {
                    let oct_rank = R::new_with_context(
                        oct.octave_rank().as_deref().unwrap_or("I"),
                        &DayType::Octave,
                        &LiturgicalContext::new()
                            .season(oct.name())
                            .of_lent(oct.is_of_lent())
                            .octave_day(date == *oct.end())
                            .also_sunday(),
                    );
                    if oct_rank.is_high_festial() {
                        result.winner.desc = self.get_octave_descriptor(&date, oct);
                    }
                }
            }

            days.push(DayDescription {
                date,
                day_in_season: season_desc,
                day_rank: result.winner.rank.clone(),
                day: result.winner,
                commemorations: result.commemorations,
                debug_trace: result.debug_trace.join(" | ").into(),
            });

            // Only carry a transfer forward if it's not already a transferred unit
            // (i.e., one we created because the feast originally fell on Sunday
            // and was moved to Saturday). This prevents a vigil that we moved
            // to Saturday from being transferred again to Sunday.
            if transfer.is_none() {
                if let Some((rank, unit)) = result.transferred {
                    if !unit.desc.contains("(transferred)") {
                        transfer = Some((rank, unit));
                    }
                }
            }
        }
        YearCalendar {
            year: self.year,
            #[cfg(test)]
            name: self.name.clone(),
            days: days.into_boxed_slice(),
        }
    }
    pub fn get_season_color(&self, date: &NaiveDate) -> ArcStr {
        let season = self.get_season(date);
        season.color().into()
    }

    pub fn get_octave_descriptor(
        &self,
        date: &NaiveDate,
        octave: &SeasonRule<NaiveDate>,
    ) -> ArcStr {
        let day_in_octave = date.signed_duration_since(*octave.begin()).num_days() + 1 + 1;

        // if its the octave day
        if date == octave.end() {
            return format!("Octave Day of {}", octave.name()).into();
        }

        // if its a sunday in the octave
        if date.weekday() == chrono::Weekday::Sun {
            return format!("Sunday in the Octave of {} ", octave.name()).into();
        }

        // else its a feria in the octave
        format!("Day {} in the Octave of {}", day_in_octave, octave.name()).into()
    }

    pub fn get_season_descriptor(&self, date: &chrono::NaiveDate) -> ArcStr {
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
        .map(|s| s.into())
        .unwrap_or_else(|| format!("of {}", season.name()));

        let week_of_month = if let Some(lower_bound) = season.append_week_of_month().as_ref() {
            if lower_bound > date {
                "".into()
            } else {
                match season.week_of_month_old_scheme() {
                    false => {
                        // New scheme: count from the preceding Sunday
                        let preceding_sunday = get_preceding_sunday(*date);
                        let month = preceding_sunday.month();
                        let first_sunday_of_month = {
                            let first_of_month =
                                NaiveDate::from_ymd_opt(preceding_sunday.year(), month, 1).unwrap();
                            get_following_sunday(first_of_month)
                        };

                        let week_of_month = num_sundays_after_date_inclusive(
                            first_sunday_of_month,
                            preceding_sunday,
                        );
                        format!(" (Week {} of {})", week_of_month, to_month_string(month))
                    }
                    true => {
                        // Old scheme: the "first Sunday of the month" is the Sunday
                        // closest to the 1st of the calendar month (may fall in the
                        // previous month). To decide whether the current week's
                        // starting Sunday belongs to this calendar month or the next,
                        // compute the nearest Sunday to the 1st of this month and the
                        // nearest Sunday to the 1st of the next month and compare.
                        let preceding_sunday = get_preceding_sunday(*date);

                        let cur_month = date.month();
                        let cur_year = date.year();
                        let (next_month, next_year) = if cur_month == 12 {
                            (1, cur_year + 1)
                        } else {
                            (cur_month + 1, cur_year)
                        };

                        let first_of_cur = NaiveDate::from_ymd_opt(cur_year, cur_month, 1).unwrap();
                        let first_of_next =
                            NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap();

                        let cur_before = get_preceding_sunday(first_of_cur);
                        let cur_after = get_following_sunday(first_of_cur);
                        let nearest_cur =
                            if first_of_cur.signed_duration_since(cur_before).num_days()
                                <= cur_after.signed_duration_since(first_of_cur).num_days()
                            {
                                cur_before
                            } else {
                                cur_after
                            };

                        let next_before = get_preceding_sunday(first_of_next);
                        let next_after = get_following_sunday(first_of_next);
                        let nearest_next =
                            if first_of_next.signed_duration_since(next_before).num_days()
                                <= next_after.signed_duration_since(first_of_next).num_days()
                            {
                                next_before
                            } else {
                                next_after
                            };

                        // Decide which calendar month this week's Sunday counts for.
                        // If the week's starting Sunday equals the nearest Sunday of the
                        // next month, attribute it to the next month. If the week's
                        // starting Sunday is before the nearest Sunday for the current
                        // month, then it belongs to the previous month (avoid "Week 0").
                        let (prev_month, prev_year) = if cur_month == 1 {
                            (12, cur_year - 1)
                        } else {
                            (cur_month - 1, cur_year)
                        };

                        let first_of_prev =
                            NaiveDate::from_ymd_opt(prev_year, prev_month, 1).unwrap();
                        let prev_before = get_preceding_sunday(first_of_prev);
                        let prev_after = get_following_sunday(first_of_prev);
                        let nearest_prev =
                            if first_of_prev.signed_duration_since(prev_before).num_days()
                                <= prev_after.signed_duration_since(first_of_prev).num_days()
                            {
                                prev_before
                            } else {
                                prev_after
                            };

                        let (label_month, first_sunday) = if preceding_sunday == nearest_next {
                            (next_month, nearest_next)
                        } else if preceding_sunday < nearest_cur {
                            (prev_month, nearest_prev)
                        } else {
                            (cur_month, nearest_cur)
                        };

                        let week_of_month =
                            num_sundays_after_date_inclusive(first_sunday, preceding_sunday);
                        format!(
                            " (Week {} of {})",
                            week_of_month,
                            to_month_string(label_month)
                        )
                    }
                }
            }
        } else {
            "".into()
        };

        let week_ordinal_str = if season.dont_show_week_of_season() {
            "".into()
        } else if week_ordinal == 0 {
            "after start ".into()
        } else if weekday == 7 {
            format!("{} ", to_roman_numeral(week_ordinal))
        } else {
            format!("week {} ", to_roman_numeral(week_ordinal))
        };

        format!("{feria} {week_ordinal_str}{suffix}{week_of_month}").into()
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
                // If no season found, choose the nearest season by distance to avoid
                // returning an unrelated season (which can cause out-of-range panics).
                eprintln!(
                    "Warning: No season found for date {}, selecting nearest season as fallback",
                    date
                );
                // Find index of season with minimal distance (days) to the date
                let mut best_idx: usize = 0;
                let mut best_dist: i64 = i64::MAX;
                for (i, season) in self.seasons.iter().enumerate() {
                    let dist = if date < season.begin() {
                        season.begin().signed_duration_since(*date).num_days()
                    } else if date > season.end() {
                        date.signed_duration_since(*season.end()).num_days()
                    } else {
                        0
                    };
                    if dist < best_dist {
                        best_dist = dist;
                        best_idx = i;
                    }
                }
                &self.seasons[best_idx]
            })
    }

    pub fn season_day_to_feast_rank<R>(&self, date: &NaiveDate) -> R
    where
        R: FeastRankResolver,
    {
        // Determine season-only rank; octaves are handled separately by the builder as competitors
        let season = self.get_season(date);

        let weekday = date.weekday().number_from_monday();
        let _feria = match weekday {
            6 => "Sabbato".to_owned(),
            7 => "Dominica".to_owned(),
            n => format!("Feria {}", to_roman_numeral((n + 1).try_into().unwrap())),
        };

        if date.weekday() == chrono::Weekday::Sun {
            let context = LiturgicalContext::new()
                .season(season.name())
                .of_lent(season.is_of_lent());
            R::new_with_context(&season.get_sunday_rank(), &DayType::Sunday, &context)
        } else {
            let context = LiturgicalContext::new()
                .season(season.name())
                .feast(self.get_season_descriptor(date))
                .of_lent(season.is_of_lent());
            // Guard against dates that don't fall within the chosen season (nearest-season fallback)
            if date < season.begin() || date > season.end() {
                eprintln!("Warning: date {} not in season '{}' range {}..={} - using default ferial rank IV", date, season.name(), season.begin(), season.end());
                R::new_with_context("IV", &DayType::Feria, &context)
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

pub fn bvm_on_saturday<R: FeastRankResolver>(lu: &mut LiturgicalUnit<R::FeastRankDescriptor>) {
    lu.desc = "BVM on Saturday".into();
    lu.rank = R::get_bvm_on_saturday_rank().unwrap().descriptor();
}

pub fn bvm_on_saturday_commemoration<R: FeastRankResolver>(
    date: NaiveDate,
) -> LiturgicalUnit<R::FeastRankDescriptor> {
    LiturgicalUnit {
        desc: "BVM on Saturday".into(),
        rank: R::get_bvm_on_saturday_rank().unwrap().descriptor(),
        date,
        color: "white".into(),
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
            name: name.into(),
            date_rule: date,
            rank: Some(rank.into()),
            of_our_lord: false,
            day_type: Some(DayType::Feast),
            color: "red".into(),
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
            name: "Test Calendar".into(),
            seasons: vec![season],
            feasts: feasts_map,
            first_advent: NaiveDate::from_ymd_opt(2025, 11, 30).unwrap(),
            next_first_advent: NaiveDate::from_ymd_opt(2026, 11, 29).unwrap(),
            calendar_type: CalendarType::OrdinaryForm,
            octaves: vec![],
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
            name: "Coverage Test".into(),
            seasons: vec![create_test_season(
                "Coverage Season",
                NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
            )],
            feasts: HashMap::new(),
            first_advent: NaiveDate::from_ymd_opt(2025, 11, 30).unwrap(),
            next_first_advent: NaiveDate::from_ymd_opt(2026, 11, 29).unwrap(),
            calendar_type: CalendarType::OrdinaryForm,
            octaves: vec![],
        };
        let test_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();
        let rank: FeastRank62 = year_calendar.season_day_to_feast_rank(&test_date);
        assert!(rank.is_ferial_or_sunday_rank());
    }
}
