use std::path::Path;

use calendar_calc::GenericCalendarHandle;
use cross_proc_cache::FsCache;
use insta::{assert_snapshot, with_settings};
use test_case::test_matrix;

fn init(year: usize) -> Vec<String> {
    let raw_calendar =
        std::fs::read_to_string("calendar_data/54.toml").expect("Failed to read calendar data");

    let calendar: GenericCalendarHandle =
        GenericCalendarHandle::load_from_str(&raw_calendar).expect("Failed to parse calendar data");

    calendar
        .create_year_calendar_54(year as i32)
        .generate_csv()
        .lines()
        .skip(1)
        .map(|s| s.to_string())
        .collect()
}

#[test_matrix(
    2025..=2026,
    0..=366
)]
fn test_calendar_for_year_54(year: usize, day: u32) {
    let calendars = FsCache::new(
        &Path::new(
            format!(
                "{}{}",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cache/cache54"),
                year
            )
            .as_str(),
        ),
        env!("TEST_FPRINT"),
    )
    .unwrap();
    let line = {
        let lines = calendars.load(|| init(year)).unwrap();
        lines
            .get(day as usize)
            .map(|s| s.to_string())
            .unwrap_or_default()
    };

    if line.is_empty() {
        return;
    }

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
}
