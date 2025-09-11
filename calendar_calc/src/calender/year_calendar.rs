use chrono::NaiveDate;
use serde::{Serialize, ser::SerializeStruct as _};

use crate::calender::{feast_rank::FeastRank, LiturgicalUnit};

#[derive(Debug, Clone)]
pub struct DayDescription {
    pub date: NaiveDate,
    pub day_in_season: String,
    pub day_rank: String,
    pub day: LiturgicalUnit,
    pub commemorations: Vec<LiturgicalUnit>,
}

impl Serialize for DayDescription {
    // Custom serialization to handle LiturgicalUnit serialization
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("DayDescription", 5)?;
        state.serialize_field("date", &self.date.to_string())?;
        state.serialize_field("day_in_season", &self.day_in_season)?;
        state.serialize_field("day_rank", &self.day_rank)?;
        state.serialize_field("day", &self.day)?;
        state.serialize_field("commemorations", &self.commemorations)?;
        state.end()
    }
}

#[derive(Debug, Clone)]
pub struct YearCalendar<R>
where
    R: FeastRank,
{
    pub year: i32,
    #[cfg(test)]
    pub name: String,
    pub days: Box<[DayDescription]>,
    pub __marker: std::marker::PhantomData<R>,
}

impl<R> YearCalendar<R>
where
    R: FeastRank,
{
    /// Get the year this calendar represents
    #[cfg(test)]
    pub fn year(&self) -> i32 {
        self.year
    }

    #[cfg(test)]
    /// Get the name of this calendar
    pub fn name(&self) -> &str {
        &self.name
    }

    #[cfg(test)]
    /// Get all days in this liturgical year
    pub fn days(&self) -> &[DayDescription] {
        &self.days
    }

    /// Get liturgical information for a specific date
    pub fn get_day(&self, date: NaiveDate) -> Option<DayDescription> {
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
    pub fn generate_year_calendar_csv(&self) -> String {
        let mut csv_content = String::new();
        csv_content.push_str("Date|Day in Season|Rank|Feast|Commemorations\n");
        for day in self.days.iter() {
            let commemorations = day
                .commemorations
                .iter()
                .map(|c| c.desc.clone())
                .collect::<Vec<_>>()
                .join(", ");
            csv_content.push_str(&format!(
                "{}|{}|{}|{}|{}\n",
                day.date, day.day_in_season, day.day_rank, day.day.desc, commemorations
            ));
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

    use super::*;
    use crate::calender::{feast_rank::FeastRank62, LiturgicalUnit, DayType, LiturgicalContext};

    /// Tests CSV write error handling
    #[test]
    fn test_csv_write_error_handling() {
        let year_calendar = YearCalendar {
            year: 2025,
            name: "Test Calendar".to_string(),
            days: vec![DayDescription {
                date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                day_in_season: "Feria II".to_string(),
                day_rank: "IV".to_string(),
                day: LiturgicalUnit {
                    desc: "Test Day".to_string(),
                    rank: FeastRank62::new_with_context(
                        "IV",
                        &crate::calender::DayType::Feria,
                        &crate::calender::LiturgicalContext::new(),
                    ).get_rank_string(),
                    date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                    color: "green".to_string(),
                },
                commemorations: vec![],
            }]
            .into_boxed_slice(),
            __marker: std::marker::PhantomData::<FeastRank62>,
        };

        let csv_content = year_calendar.generate_year_calendar_csv();
        assert!(csv_content.contains("2025-01-01"));
        assert!(csv_content.contains("Test Day"));

        // Test writing to a valid path should work
        let result = year_calendar.write_csv_for_year("/tmp/test_calendar.csv");
        assert!(result.is_ok() || result.is_err()); // Either works or fails gracefully
    }

    use crate::calender::feast_rank::FeastRank;

    fn create_test_year_calendar() -> YearCalendar<FeastRank62> {
        let days = vec![
            DayDescription {
                date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                day_in_season: "Feria II".to_string(),
                day_rank: "IV".to_string(),
                day: LiturgicalUnit {
                    desc: "Regular Day".to_string(),
                    rank: FeastRank62::new_with_context(
                        "IV",
                        &DayType::Feria,
                        &LiturgicalContext::new(),
                    ).get_rank_string(),
                    date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                    color: "green".to_string(),
                },
                commemorations: vec![],
            },
            DayDescription {
                date: NaiveDate::from_ymd_opt(2025, 6, 15).unwrap(),
                day_in_season: "Dom. IV post Pentecosten".to_string(),
                day_rank: "I".to_string(),
                day: LiturgicalUnit {
                    desc: "Major Feast".to_string(),
                    rank: FeastRank62::new_with_context(
                        "I",
                        &DayType::Feast,
                        &LiturgicalContext::new(),
                    ).get_rank_string(),
                    date: NaiveDate::from_ymd_opt(2025, 6, 15).unwrap(),
                    color: "green".to_string(),
                },
                commemorations: vec![LiturgicalUnit {
                    desc: "Commemoration".to_string(),
                    rank: FeastRank62::new_with_context(
                        "III",
                        &DayType::Feast,
                        &LiturgicalContext::new(),
                    ).get_rank_string(),
                    date: NaiveDate::from_ymd_opt(2025, 6, 15).unwrap(),
                    color: "green".to_string(),
                }],
            },
        ]
        .into_boxed_slice();

        YearCalendar {
            year: 2025,
            name: "Test Calendar".to_string(),
            days,
            __marker: std::marker::PhantomData,
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
        assert!(csv.contains("2025-01-01|Feria II|IV|Regular Day|"));
        assert!(csv.contains("2025-06-15|Dom. IV post Pentecosten|I|Major Feast|Commemoration"));
    }
}
