use insta::{assert_snapshot, with_settings};
use liturgy::{GenericCalendarHandle, YearCalendarHandle};
use rayon::prelude::*;
use std::cell::OnceCell;
use test_case::test_matrix;

thread_local! {
    static CALENDARS: OnceCell<Vec<YearCalendarHandle>> = const { OnceCell::new() };
}

const START: i32 = 2020;
const END: i32 = 2030;

pub fn initialize() {
    CALENDARS.with(|cell| {
        cell.get_or_init(|| {
            let raw_calendar = std::fs::read_to_string("calendar_data/54.toml")
                .expect("Failed to read calendar data");

            let calendar: GenericCalendarHandle =
                GenericCalendarHandle::load_from_str(&raw_calendar)
                    .expect("Failed to parse calendar data");

            (START..=END)
                .par_bridge()
                .map(|year| calendar.create_year_calendar(year))
                .collect()
        });
    });
}

#[test_matrix(
    2025..=2025
)]
fn test_calendar_for_year_54(year: i32) {
    initialize();

    let desc = CALENDARS.with(|cell| {
        let calendars = cell.get().unwrap();
        let calendar = calendars.iter().find(|c| c.year() == year).unwrap();

        calendar.generate_csv()
    });

    desc.lines().skip(1).par_bridge().for_each(|line| {
        let date = line.split('|').next().unwrap();

        with_settings!(
            {snapshot_suffix => format!("_{}", date)},
            {
                assert_snapshot!(line);
            }
        );
    });
}
