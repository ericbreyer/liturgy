use std::path::Path;

use calendar_calc::GenericCalendarHandle;
use cross_proc_cache::FsCache;
use insta::{assert_snapshot, with_settings};
use test_case::test_matrix;

fn init_for_year(year: usize) -> Vec<String> {
    let raw_calendar =
        std::fs::read_to_string("calendar_data/ef.toml").expect("Failed to read calendar data");

    let calendar: GenericCalendarHandle =
        GenericCalendarHandle::load_from_str(&raw_calendar).expect("Failed to parse calendar data");

    calendar
        .create_year_calendar_62(year as i32)
        .generate_csv()
        .lines()
        .skip(1)
        .map(|s| s.to_string())
        .collect()
}

#[test_matrix(
    2025..=2027,
    0..=366
)]
fn test_calendar_for_year_62(year: usize, day: u32) {
    let calendars = FsCache::new(
        Path::new(
            format!(
                "{}{}",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cache/cache62"),
                year
            )
            .as_str(),
        ),
        env!("TEST_FPRINT"),
    )
    .unwrap();
    let line = {
        let lines = calendars.load(|| init_for_year(year)).unwrap();
        lines
            .get(day as usize)
            .map(|s| s.to_string())
            .unwrap_or_default()
    };

    if line.is_empty() {
        return;
    }

    let date = line.split('|').next().unwrap();
    with_settings!(
        {snapshot_suffix => format!("_{}", date)
    },
        {
            assert_snapshot!(line);
        }
    );
}
