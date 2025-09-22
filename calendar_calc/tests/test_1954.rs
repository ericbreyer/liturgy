use std::cell::OnceCell;

use calendar_calc::{GenericCalendarHandle, YearCalendarHandle};
use insta::{assert_snapshot, with_settings};
use rayon::prelude::*;
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
        // split the line at the 5th '|'
        let idx_5 = line.match_indices('|').nth(4).unwrap().0;
        let (part1, part2) = line.split_at(idx_5);
        let split_line = (part1, &part2[1..]); // skip the '|'
        with_settings!(
            {snapshot_suffix => format!("_{}", date), description => split_line.1
        },
            {
                assert_snapshot!(split_line.0);
            }
        );
    });
}
