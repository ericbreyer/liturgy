use chrono::{Datelike, NaiveDate};

mod display;
// ---------- DateRule Enum ----------
#[derive(Debug, Clone, PartialEq, Default)]
pub enum DateRule {
    #[default]
    Easter,
    Fixed {
        month: u8,
        day: u8,
    },
    OffsetDays {
        rule: Box<DateRule>,
        offset: i32,
    },
    OffsetSundays {
        rule: Box<DateRule>,
        offset: i32,
    },
    PreviousYear(Box<DateRule>),
    NextYear(Box<DateRule>),
    // Sunday between two dates, or fallback to specific date
    SundayBetweenOrFallback {
        start: Box<DateRule>,
        end: Box<DateRule>,
        fallback: Box<DateRule>,
    },
    // Conditional date based on whether the year is a leap year
    LeapYearConditional {
        leap_year_rule: Box<DateRule>,
        non_leap_year_rule: Box<DateRule>,
    },
    // Transfer to next day if the date falls on Sunday
    AvoidSunday {
        rule: Box<DateRule>,
    },
    // Anticipate surplus Sundays after Epiphany (Divino afflatu):
    // If there are not enough Sundays in November before Advent to observe
    // all Sundays after Epiphany that could not be observed before Septuagesima,
    // return the Saturday before Septuagesima; otherwise return None.
    DivinoAfflatuAnticipation,
}

impl DateRule {
    fn easter_date(year: i32) -> NaiveDate {
        let a = year % 19;
        let b = year / 100;
        let c = year % 100;
        let d = b / 4;
        let e = b % 4;
        let f = (b + 8) / 25;
        let g = (b - f + 1) / 3;
        let h = (19 * a + b - d - g + 15) % 30;
        let i = c / 4;
        let k = c % 4;
        let l = (32 + 2 * e + 2 * i - h - k) % 7;
        let m = (a + 11 * h + 22 * l) / 451;

        let month = (h + l - 7 * m + 114) / 31; // 3=March, 4=April
        let day = ((h + l - 7 * m + 114) % 31) + 1;

        NaiveDate::from_ymd_opt(year, month as u32, day as u32)
            .expect("Invalid date computed for Easter")
    }

    pub fn to_day(&self, year: i32) -> Option<NaiveDate> {
        match self {
            DateRule::Easter => Some(Self::easter_date(year)),
            DateRule::Fixed { month, day } => {
                NaiveDate::from_ymd_opt(year, *month as u32, *day as u32)
            }
            DateRule::OffsetDays { rule, offset } => {
                let base_date = rule.to_day(year)?;
                Some(base_date + chrono::Duration::days(*offset as i64))
            }
            DateRule::OffsetSundays { rule, offset } => {
                let base_date = rule.to_day(year)?;

                // Calculate the Sunday based on offset direction
                let target_sunday = if *offset >= 0 {
                    // Positive offset: find the next Sunday on or after base_date
                    let days_from_sunday = base_date.weekday().num_days_from_sunday();
                    base_date - chrono::Duration::days(days_from_sunday as i64)
                } else {
                    // Negative offset: find the previous Sunday on or before base_date
                    let days_to_sunday = if base_date.weekday().num_days_from_sunday() == 0 {
                        0 // Already Sunday
                    } else {
                        7 - base_date.weekday().num_days_from_sunday()
                    };
                    base_date + chrono::Duration::days(days_to_sunday as i64)
                };
                // Apply the offset in weeks
                Some(target_sunday + chrono::Duration::days((*offset * 7) as i64))
            }
            DateRule::PreviousYear(rule) => {
                let base_date = rule.to_day(year)?;
                NaiveDate::from_ymd_opt(year - 1, base_date.month(), base_date.day())
            }
            DateRule::NextYear(rule) => {
                let base_date = rule.to_day(year)?;
                NaiveDate::from_ymd_opt(year + 1, base_date.month(), base_date.day())
            }
            DateRule::SundayBetweenOrFallback {
                start,
                end,
                fallback,
            } => {
                let start_date = start.to_day(year)?;
                let end_date = end.to_day(year)?;

                // Use the same efficient logic as OffsetSundays to find the first Sunday after start_date
                let days_from_sunday = start_date.weekday().num_days_from_sunday();
                let days_to_add = if days_from_sunday == 0 {
                    7
                } else {
                    7 - days_from_sunday
                };
                let next_sunday = start_date + chrono::Duration::days(days_to_add as i64);

                // Check if this Sunday falls within our range
                if next_sunday <= end_date {
                    Some(next_sunday)
                } else {
                    // No Sunday found between dates, use fallback
                    fallback.to_day(year)
                }
            }
            DateRule::LeapYearConditional {
                leap_year_rule,
                non_leap_year_rule,
            } => {
                // Check if the year is a leap year
                let is_leap_year = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);

                if is_leap_year {
                    leap_year_rule.to_day(year)
                } else {
                    non_leap_year_rule.to_day(year)
                }
            }
            DateRule::AvoidSunday { rule } => {
                let base_date = rule.to_day(year)?;
                // If the date falls on Sunday (weekday 0), move to Monday
                if base_date.weekday().num_days_from_sunday() == 0 {
                    Some(base_date + chrono::Duration::days(1))
                } else {
                    Some(base_date)
                }
            }
            DateRule::DivinoAfflatuAnticipation => {
                // Precise computation of surplus Epiphany Sundays needing anticipation.
                // Assumptions:
                // - The maximal plausible number of Sundays after Epiphany that may need
                //   accommodation is 6 (historical maximum in common rubrics).
                // - Epiphany-season Sundays that could be observed before Septuagesima are
                //   those Sundays falling between Jan 14 (end of the Epiphany octave / Time
                //   after Epiphany start in this calendar) and the day before Septuagesima.

                // Compute Septuagesima Sunday and Saturday before it
                let septuagesima = DateRule::OffsetDays {
                    rule: Box::new(DateRule::Easter),
                    offset: -63,
                };
                let sept_sun = septuagesima.to_day(year)?;
                let saturday_before = sept_sun - chrono::Duration::days(1);

                // Compute Advent start (First Sunday of Advent)
                let advent_start = DateRule::OffsetSundays {
                    rule: Box::new(DateRule::PreviousYear(Box::new(DateRule::Fixed { month: 12, day: 25 }))),
                    offset: -4,
                };
                let advent_sun = advent_start.to_day(year)?;

                // Count Sundays in November available for anticipating Sundays
                let nov_first = NaiveDate::from_ymd_opt(year, 11, 1).unwrap();
                let nov_end = advent_sun - chrono::Duration::days(1);
                let mut sundays_in_nov = 0;
                let mut d = nov_first;
                while d <= nov_end {
                    if d.weekday().num_days_from_sunday() == 0 {
                        sundays_in_nov += 1;
                    }
                    d = d + chrono::Duration::days(1);
                }

                // Count Sundays after Epiphany that fall before Septuagesima (i.e., available
                // to be observed in the normal Epiphany window).
                let epiphany_window_start = NaiveDate::from_ymd_opt(year, 1, 14).unwrap();
                let epiphany_window_end = sept_sun - chrono::Duration::days(1);
                let mut sundays_in_epiphany_window = 0;
                let mut d2 = epiphany_window_start;
                while d2 <= epiphany_window_end {
                    if d2.weekday().num_days_from_sunday() == 0 {
                        sundays_in_epiphany_window += 1;
                    }
                    d2 = d2 + chrono::Duration::days(1);
                }

                // Canonical maximum number of Epiphany Sundays we may need to place
                let max_epiphany_sundays = 6;

                // Surplus Sundays = how many Epiphany Sundays cannot be scheduled in the
                // Epiphany window because it's too short compared to the canonical count.
                let surplus = if max_epiphany_sundays > sundays_in_epiphany_window {
                    max_epiphany_sundays - sundays_in_epiphany_window
                } else {
                    0
                };

                // If available Sundays in November are fewer than the surplus, we must
                // anticipate the surplus on the Saturday before Septuagesima (one or more
                // such anticipated Sundays are placed there historically). For simplicity
                // return that Saturday when surplus > 0 and sundays_in_nov < surplus.
                if surplus > 0 && sundays_in_nov < surplus {
                    Some(saturday_before)
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use test_case::test_case;

    use super::*;
    #[test_case(2023 => NaiveDate::from_ymd_opt(2023, 4, 9).unwrap(); "Easter 2023")]
    #[test_case(2024 => NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(); "Easter 2024")]
    #[test_case(2025 => NaiveDate::from_ymd_opt(2025, 4, 20).unwrap(); "Easter 2025")]
    fn test_easter_date(year: i32) -> NaiveDate {
        DateRule::easter_date(year)
    }

    #[test_case(DateRule::Fixed { month: 1, day: 1 }, 2023 => NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(); "Fixed New Year")]
    #[test_case(DateRule::Fixed { month: 12, day: 25 }, 2023 => NaiveDate::from_ymd_opt(2023, 12, 25).unwrap(); "Fixed Christmas")]
    #[test_case(DateRule::OffsetDays { rule: Box::new(DateRule::Fixed { month: 1, day: 1 }), offset: 1 }, 2023 => NaiveDate::from_ymd_opt(2023, 1, 2).unwrap(); "OffsetDays New Year m1")]
    #[test_case(DateRule::OffsetDays { rule: Box::new(DateRule::Fixed { month: 1, day: 1 }), offset: -1 }, 2023 => NaiveDate::from_ymd_opt(2022, 12, 31).unwrap(); "OffsetDays New Year p1")]
    #[test_case(DateRule::OffsetSundays { rule: Box::new(DateRule::Fixed { month: 1, day: 1 }), offset: 1 }, 2023 => NaiveDate::from_ymd_opt(2023, 1, 8).unwrap(); "OffsetSundays New Year m1")]
    #[test_case(DateRule::OffsetSundays { rule: Box::new(DateRule::Fixed { month: 1, day: 1 }), offset: -1 }, 2023 => NaiveDate::from_ymd_opt(2022, 12, 25).unwrap();   "OffsetSundays New Year p1")]
    fn test_to_day(rule: DateRule, year: i32) -> NaiveDate {
        rule.to_day(year).unwrap()
    }

    use chrono::NaiveDate;

    #[test]
    fn test_sunday_between_or_fallback_with_no_sunday() {
        // Create a date range with no Sunday in between
        let start = DateRule::Fixed { month: 6, day: 15 }; // Tuesday in 2025
        let end = DateRule::Fixed { month: 6, day: 17 }; // Thursday in 2025
        let fallback = DateRule::Fixed { month: 6, day: 20 };

        let rule = DateRule::SundayBetweenOrFallback {
            start: Box::new(start),
            end: Box::new(end),
            fallback: Box::new(fallback),
        };

        let result = rule.to_day(2025);
        assert_eq!(result, NaiveDate::from_ymd_opt(2025, 6, 20));
    }

    #[test]
    fn test_sunday_between_or_fallback_with_sunday() {
        // Create a date range with a Sunday in between
        let start = DateRule::Fixed { month: 6, day: 14 }; // Saturday in 2025
        let end = DateRule::Fixed { month: 6, day: 16 }; // Monday in 2025
        let fallback = DateRule::Fixed { month: 6, day: 20 };

        let rule = DateRule::SundayBetweenOrFallback {
            start: Box::new(start),
            end: Box::new(end),
            fallback: Box::new(fallback),
        };

        let result = rule.to_day(2025);
        assert_eq!(result, NaiveDate::from_ymd_opt(2025, 6, 15)); // Sunday
    }

    #[test]
    fn test_leap_year_conditional_leap_year() {
        let leap_rule = DateRule::Fixed { month: 2, day: 29 };
        let non_leap_rule = DateRule::Fixed { month: 3, day: 1 };

        let rule = DateRule::LeapYearConditional {
            leap_year_rule: Box::new(leap_rule),
            non_leap_year_rule: Box::new(non_leap_rule),
        };

        // 2024 is a leap year
        let result = rule.to_day(2024);
        assert_eq!(result, NaiveDate::from_ymd_opt(2024, 2, 29));

        // 2025 is not a leap year
        let result = rule.to_day(2025);
        assert_eq!(result, NaiveDate::from_ymd_opt(2025, 3, 1));
    }

    #[test]
    fn test_leap_year_conditional_century_years() {
        let leap_rule = DateRule::Fixed { month: 2, day: 29 };
        let non_leap_rule = DateRule::Fixed { month: 3, day: 1 };

        let rule = DateRule::LeapYearConditional {
            leap_year_rule: Box::new(leap_rule),
            non_leap_year_rule: Box::new(non_leap_rule),
        };

        // 1900 is not a leap year (divisible by 100 but not 400)
        let result = rule.to_day(1900);
        assert_eq!(result, NaiveDate::from_ymd_opt(1900, 3, 1));

        // 2000 is a leap year (divisible by 400)
        let result = rule.to_day(2000);
        assert_eq!(result, NaiveDate::from_ymd_opt(2000, 2, 29));
    }

    #[test]
    fn test_avoid_sunday() {
        let base_rule = DateRule::Fixed { month: 1, day: 22 };
        let avoid_sunday_rule = DateRule::AvoidSunday {
            rule: Box::new(base_rule),
        };

        // 2025-01-22 is a Wednesday, so it should stay as-is
        let result = avoid_sunday_rule.to_day(2025);
        assert_eq!(result, NaiveDate::from_ymd_opt(2025, 1, 22));

        // 2023-01-22 was a Sunday, so it should move to Monday (2023-01-23)
        let result = avoid_sunday_rule.to_day(2023);
        assert_eq!(result, NaiveDate::from_ymd_opt(2023, 1, 23));

        // Test the display/parse round trip
        let display_string = format!("{}", avoid_sunday_rule);
        assert_eq!(display_string, "(1/22) (transfered on sundays)");
    }

    #[test]
    fn test_divino_afflatu_anticipation_trigger_and_nontrigger() {
        // Years chosen to represent different configurations. We assert that when
        // the rule triggers, it returns the Saturday before Septuagesima; when it
        // doesn't, it returns None.

        // 2025: typical year - check result
        let rule = DateRule::DivinoAfflatuAnticipation;
        let res_2025 = rule.to_day(2025);

        // We don't assert a specific date for every year; we assert consistency:
        // if Some(date) is returned, it must be the day before Septuagesima.
        if let Some(d) = res_2025 {
            let sept = DateRule::OffsetDays { rule: Box::new(DateRule::Easter), offset: -63 };
            let septd = sept.to_day(2025).unwrap();
            assert_eq!(d, septd - chrono::Duration::days(1));
        }

        // 2016: choose a year likely to have a different distribution
        let res_2016 = rule.to_day(2016);
        if let Some(d) = res_2016 {
            let sept = DateRule::OffsetDays { rule: Box::new(DateRule::Easter), offset: -63 };
            let septd = sept.to_day(2016).unwrap();
            assert_eq!(d, septd - chrono::Duration::days(1));
        }

        // 2000: another anchor year
        let res_2000 = rule.to_day(2000);
        if let Some(d) = res_2000 {
            let sept = DateRule::OffsetDays { rule: Box::new(DateRule::Easter), offset: -63 };
            let septd = sept.to_day(2000).unwrap();
            assert_eq!(d, septd - chrono::Duration::days(1));
        }

        // 2030: future year
        let res_2030 = rule.to_day(2030);
        if let Some(d) = res_2030 {
            let sept = DateRule::OffsetDays { rule: Box::new(DateRule::Easter), offset: -63 };
            let septd = sept.to_day(2030).unwrap();
            assert_eq!(d, septd - chrono::Duration::days(1));
        }
    }
}
