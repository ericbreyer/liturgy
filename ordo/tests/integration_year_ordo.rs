use calendar_calc::GenericCalendarHandle62;
use insta::{assert_snapshot, with_settings};
use ordo::{ordo_repo::OrdoRepo, VespersOrdo};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator as _};
use types::{DayDescription, DayRank62};
use anyhow::Result;


/// Build a vespers representation for a day and return a debug string.
/// This is a small public helper used by integration tests to snapshot
/// full-year ordos.
pub fn build_vespers_snapshot(
    day: &DayDescription<DayRank62>,
    repo: &OrdoRepo,
) -> Result<(String, Vec<String>)> {
    let v = day.vespers_ordo(repo)?;
    let v_sources = day.vespers_ordo_sources(repo)?;

    Ok((format!("{}\n{}", day.date, v), v_sources))
}


#[test]
#[ignore]  // TODO: Re-enable when all seasonal rules are complete
fn build_ordos_for_year_of_2025() {
    // Resolve the calendar data file relative to the workspace root using
    // CARGO_MANIFEST_DIR
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    // Path: <workspace>/calendar_calc/calendar_data/ef.toml (1962/62 calendar)
    let path = std::path::Path::new(&manifest)
        .parent()
        .expect("workspace parent")
        .join("calendar_calc/calendar_data/ef.toml");

    let cal = GenericCalendarHandle62::load_from_file(path).expect("load calendar");
    let year = cal.create_year_calendar(2025);

    let days = year.get_all_days();
    assert!(!days.is_empty(), "expected non-empty year");

    // For every day in the covered periods, attempt to build a Vespers using OrdoRepo rules.
    // Covered periods:
    // - Advent (November-December)
    // - Christmas (December-January)  
    // - Time after Epiphany (January-February)
    // - Septuagesima-Lent (February-March)
    // Note: Time after Pentecost (April-November) rules not yet implemented
    let repo = OrdoRepo::load_from_dir("ordo/rules").expect("load ordo rules");
    days.par_iter()
        .filter(|d| {
            // Parse month from date string (YYYY-MM-DD format)
            let date_str = d.date.to_string();
            if let Some(month_str) = date_str.split('-').nth(1) {
                if let Ok(month) = month_str.parse::<u32>() {
                    // Only test Dec, Jan, Feb, Mar (covered by rules)
                    return month >= 12 || month <= 3;
                }
            }
            false
        })
        .for_each(|d| {
            let s = build_vespers_snapshot(d, &repo).unwrap();
            // Format date as YYYY_MM_DD to match snapshot naming convention
            let date_str = d.date.to_string();
            let date_parts: Vec<&str> = date_str.split('-').collect();
            let date_formatted = if date_parts.len() == 3 {
                format!("{}_{}_{}",  date_parts[0], date_parts[1], date_parts[2])
            } else {
                date_str
            };
            with_settings!(
            {snapshot_suffix => format!("_{}", date_formatted), description => format!("{:?}", s.1)}, 
            {
                assert_snapshot!("day_vespers", s.0);
            });
        })
}
