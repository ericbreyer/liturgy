use calendar_calc::GenericCalendarHandle;
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
fn build_ordos_for_year_of_2025() {
    // Resolve the calendar data file relative to the workspace root using
    // CARGO_MANIFEST_DIR
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    // Path: <workspace>/calendar_calc/calendar_data/ef.toml (1962/62 calendar)
    let path = std::path::Path::new(&manifest)
        .parent()
        .expect("workspace parent")
        .join("calendar_calc/calendar_data/ef.toml");

    let cal = GenericCalendarHandle::load_from_file(path).expect("load calendar");
    let year = cal.create_year_calendar_62(2025);

    let days = year.get_all_days();
    assert!(!days.is_empty(), "expected non-empty year");

    // For every day, attempt to build a Vespers using OrdoRepo rules; ensure no
    // panics and a Vespers instance is produced. We also count how many days
    // produced Proper for antiphons
    let repo = OrdoRepo::load_from_dir("ordo/rules").expect("load ordo rules");
    days.par_iter().for_each(|d| {
        let s = build_vespers_snapshot(d, &repo).unwrap();
        with_settings!(
        {snapshot_suffix => format!("_{}", d.date.format("%Y_%m_%d")), description => format!("{:?}", s.1)}, 
        {
            assert_snapshot!("day_vespers", s.0);
        });
    })
}
