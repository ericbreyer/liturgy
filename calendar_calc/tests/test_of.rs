use std::path::Path;

use calendar_calc::GenericCalendarHandle;
use cross_proc_cache::FsCache;
use insta::{assert_snapshot, with_settings};
use test_case::test_matrix;

#[derive(Debug, Clone, Copy)]
enum CalendarType {
    Of,
    UsExtended,
}

fn init_of_for_year(year: usize) -> Vec<String> {
    let raw_calendar =
        std::fs::read_to_string("calendar_data/of.toml").expect("Failed to read calendar data");

    let calendar: GenericCalendarHandle =
        GenericCalendarHandle::load_from_str(&raw_calendar).expect("Failed to parse calendar data");

    calendar
        .create_year_calendar_of(year as i32)
        .generate_csv()
        .lines()
        .skip(1)
        .map(|s| s.to_string())
        .collect()
}

fn init_us_extended_for_year(year: usize) -> Vec<String> {
    let calendar = GenericCalendarHandle::load_with_extensions(
        "calendar_data/of.toml",
        &["calendar_data/of-us-extensions.toml"],
    )
    .expect("Failed to load calendar with US extensions");

    calendar
        .create_year_calendar_of(year as i32)
        .generate_csv()
        .lines()
        .skip(1)
        .map(|s| s.to_string())
        .collect()
}

#[test_matrix(
    2025..=2026,
    0..=366,
    [CalendarType::Of, CalendarType::UsExtended]
)]
fn test_calendar_for_year(year: usize, day: u32, cal: CalendarType) {
    let line = match cal {
        CalendarType::Of => {
            let cache = FsCache::new(
                &Path::new(
                    format!(
                        "{}{}",
                        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cache/cache_of"),
                        year
                    )
                    .as_str(),
                ),
                env!("TEST_FPRINT"),
            )
            .unwrap();
            let lines = cache.load(|| init_of_for_year(year)).unwrap();
            lines
                .get(day as usize)
                .map(|s| s.to_string())
                .unwrap_or_default()
        }
        CalendarType::UsExtended => {
            let cache = FsCache::new(
                &Path::new(
                    format!(
                        "{}{}",
                        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cache/cache_of_us"),
                        year
                    )
                    .as_str(),
                ),
                env!("TEST_FPRINT"),
            )
            .unwrap();
            let lines = cache.load(|| init_us_extended_for_year(year)).unwrap();
            lines
                .get(day as usize)
                .map(|s| s.to_string())
                .unwrap_or_default()
        }
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
        {snapshot_suffix => format!("_{}_of_{:?}", date, cal), description => split_line.1
    },
        {
            assert_snapshot!(split_line.0);
        }
    );
}
