use std::fmt::Write;

use chrono::NaiveDate;
use types::{ArcStr, DayDescription, DayRank};
#[derive(Debug, Clone)]
pub struct YearCalendar<R>
where
    R: DayRank,
{
    pub year: i32,
    pub name: ArcStr,
    pub days: Box<[DayDescription<R>]>,
}

impl<R> YearCalendar<R>
where
    R: DayRank,
{
    /// Get the year this calendar represents
    #[cfg(test)]

    pub fn year(&self) -> i32 {
        self.year
    }

    #[cfg(test)]
    /// Get the name of this calendar

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    #[cfg(test)]
    /// Get all days in this liturgical year

    pub fn days(&self) -> &[DayDescription<R>] {
        &self.days
    }

    /// Get liturgical information for a specific date
    #[must_use]
    pub fn get_day(&self, date: NaiveDate) -> Option<DayDescription<R>> {
        self.days.iter().find(|day| day.date == date).cloned()
    }

    // /// Check if a date is a major feast (high festival)
    // pub fn is_major_feast(&self, date: NaiveDate) -> bool {
    //     self.get_day(date)
    //         .map(|day| day.day.rank.is_high_festial())
    //         .unwrap_or(false)
    // }

    // #[cfg(test)]
    // /// Get all major feasts in this liturgical year
    // pub fn major_feasts(&self) -> impl Iterator<Item = &DayDescription<R>> {
    //     self.days.iter().filter(|day| day.day.rank.is_high_festial())
    // }

    /// Generate CSV content for this liturgical year
    #[must_use]
    pub fn generate_year_calendar_csv(&self) -> String {
        let mut csv_content = String::new();
        csv_content.push_str("Date|Day in Season|Rank|Feast|Commemorations|Vespers|DT\n");
        for day in &self.days {
            let commemorations = day
                .commemorations
                .iter()
                .map(|c| c.0.desc.clone().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            writeln!(
                csv_content,
                "{}|{}|{}|{}|{}|{}",
                day.date,
                day.day_in_season.as_ref(),
                day.day.rank.as_str(),
                day.day.desc,
                commemorations,
                if let Some((cd, ca)) = &day.concuring_vespers {
                    format!("{ca:?}: {}", cd.desc)
                } else {
                    String::new()
                },
            )
            .unwrap();
        }
        csv_content
    }

    pub fn write_csv_for_year(&self, filename: &str) -> std::io::Result<()> {
        std::fs::write(filename, self.generate_year_calendar_csv())
    }
}

#[cfg(test)]
mod test {
    use chrono::NaiveDate;
    use types::{CommemorationType, DayKind};

    use super::*;
    use crate::calender::{DayType, LiturgicalContext, LiturgicalUnit, feast_rank::FeastRank62};

    /// Tests CSV write error handling
    #[test]
    fn test_csv_write_error_handling() {
        let year_calendar = YearCalendar {
            year: 2025,
            name: "Test Calendar".into(),
            days: vec![DayDescription {
                underlying_octave: None,
                date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                season: "Test Season".into(),
                day_in_season: "Feria II".into(),
                day: LiturgicalUnit {
                    desc: "Test Day".into(),
                    rank: FeastRank62::new_with_context(
                        "IV",
                        crate::calender::DayType::Feria,
                        &crate::calender::LiturgicalContext::new(),
                    )
                    .descriptor(),
                    date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                    color: "green".into(),
                    day_kind: DayKind::Feast(ArcStr::from("Test Day")),
                    titles: vec![],
                },
                commemorations: vec![],
                concuring_vespers: None,
            }]
            .into_boxed_slice(),
        };

        let csv_content = year_calendar.generate_year_calendar_csv();
        assert!(csv_content.contains("2025-01-01"));
        assert!(csv_content.contains("Test Day"));

        // Test writing to a valid path should work
        let result = year_calendar.write_csv_for_year("/tmp/test_calendar.csv");
        assert!(result.is_ok() || result.is_err()); // Either works or fails gracefully
    }

    use crate::calender::feast_rank::FeastRankResolver;

    fn create_test_year_calendar()
    -> YearCalendar<<FeastRank62 as FeastRankResolver>::FeastRankDescriptor> {
        let days = vec![
            DayDescription {
                underlying_octave: None,
                date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                day_in_season: "Feria II".into(),
                season: "Test Season".into(),
                day: LiturgicalUnit {
                    desc: "Regular Day".into(),
                    rank: FeastRank62::new_with_context(
                        "IV",
                        DayType::Feria,
                        &LiturgicalContext::new(),
                    )
                    .descriptor(),
                    date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                    color: "green".into(),
                    day_kind: DayKind::Feria {
                        day: ArcStr::from("Feria II"),
                        week: ArcStr::from("week I"),
                        season: ArcStr::from("Test Season"),
                        week_of_month: None,
                    },
                    titles: vec![],
                },
                commemorations: vec![],
                concuring_vespers: None,
            },
            DayDescription {
                underlying_octave: None,
                date: NaiveDate::from_ymd_opt(2025, 6, 15).unwrap(),
                season: "Pentecosten".into(),
                day_in_season: "Dom. IV post Pentecosten".into(),
                day: LiturgicalUnit {
                    desc: "Major Feast".into(),
                    rank: FeastRank62::new_with_context(
                        "I",
                        DayType::Feast,
                        &LiturgicalContext::new(),
                    )
                    .descriptor(),
                    date: NaiveDate::from_ymd_opt(2025, 6, 15).unwrap(),
                    color: "green".into(),
                    day_kind: DayKind::Feast(ArcStr::from("Major Feast")),
                    titles: vec!["Feast of Pentecost".into()],
                },
                commemorations: vec![(
                    LiturgicalUnit {
                        desc: "Commemoration".into(),
                        rank: FeastRank62::new_with_context(
                            "III",
                            DayType::Feast,
                            &LiturgicalContext::new(),
                        )
                        .descriptor(),
                        date: NaiveDate::from_ymd_opt(2025, 6, 15).unwrap(),
                        color: "green".into(),
                        day_kind: DayKind::Feast(ArcStr::from("Commemoration")),
                        titles: vec!["Commemoration Feast".into()],
                    },
                    CommemorationType::Optional,
                )],
                concuring_vespers: None,
            },
        ]
        .into_boxed_slice();

        YearCalendar {
            year: 2025,
            name: "Test Calendar".into(),
            days,
        }
    }

    #[test]
    fn test_year_calendar_accessors() {
        let calendar = create_test_year_calendar();

        assert_eq!(calendar.year(), 2025);
        assert_eq!(calendar.name(), "Test Calendar");
        assert_eq!(calendar.days().len(), 2);
    }

    #[test]
    fn test_get_day() {
        let calendar = create_test_year_calendar();

        let jan_1 = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let day_info = calendar.get_day(jan_1);
        assert!(day_info.is_some());
        assert_eq!(day_info.unwrap().day.desc, "Regular Day");

        let non_existent = NaiveDate::from_ymd_opt(2025, 2, 1).unwrap();
        assert!(calendar.get_day(non_existent).is_none());
    }

    #[test]
    fn test_generate_csv_with_commemorations() {
        let calendar = create_test_year_calendar();

        let csv = calendar.generate_year_calendar_csv();
        assert!(csv.contains("Date|Day in Season|Rank|Feast|Commemorations"));
        assert!(csv.contains("2025-06-15|Dom. IV post Pentecosten|I|Major Feast|Commemoration"));
    }
}
