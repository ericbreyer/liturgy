use std::collections::BTreeSet;
use std::path::PathBuf;

use calendar_calc::calender::feast_rank::{
    FeastRank54, FeastRank62, FeastRankOf, FeastRankResolver,
};
use calendar_calc::calender::generic_calendar::{
    CalendarCycle, CalendarTypeProvider, CycleFeast, GenericCalendar,
};
use chrono::{Datelike, NaiveDate, Weekday};

fn data_path(file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("calendar_data")
        .join(file)
}

fn load<Rank: FeastRankResolver + CalendarTypeProvider>(file: &str) -> GenericCalendar<Rank> {
    GenericCalendar::from_toml_file(data_path(file)).unwrap()
}

#[test]
fn mary_magdalene_penitent_is_a_title_not_part_of_the_name() {
    let calendar = load::<FeastRank54>("54.toml");
    let feasts = calendar.cycle_feasts(CalendarCycle::Sanctoral, 2026);
    let feast = feasts.iter().find(|feast| feast.name == "St. Mary Magdalene").unwrap();
    assert_eq!(feast.titles, vec!["Penitent"]);
    assert_eq!(feast.date.as_deref(), Some("2026-07-22"));
    assert!(!feasts.iter().any(|feast| feast.name == "St. Mary Magdalene Penitent"));
}

fn assert_partition<Rank: FeastRankResolver + CalendarTypeProvider>(
    calendar: &GenericCalendar<Rank>,
) {
    for year in [2024, 2026] {
        let sanctoral = calendar.cycle_feasts(CalendarCycle::Sanctoral, year);
        let temporal = calendar.cycle_feasts(CalendarCycle::Temporal, year);
        let saints: BTreeSet<_> = sanctoral.iter().map(|feast| feast.name.as_str()).collect();
        let seasons: BTreeSet<_> = temporal.iter().map(|feast| feast.name.as_str()).collect();
        let source: BTreeSet<_> = calendar
            .feasts
            .iter()
            .map(|feast| feast.name.as_ref())
            .collect();
        assert!(saints.is_disjoint(&seasons));
        assert_eq!(
            saints.union(&seasons).copied().collect::<BTreeSet<_>>(),
            source
        );
        assert_eq!(sanctoral.len() + temporal.len(), calendar.feasts.len());
        for (name, cycle) in &calendar.cycle_overrides {
            assert!(source.contains(name.as_ref()), "Unknown override: {name}");
            assert!(
                calendar
                    .cycle_feasts(*cycle, year)
                    .iter()
                    .any(|feast| feast.name == name.as_ref())
            );
        }
        let mut expected: Vec<_> = calendar
            .feasts
            .iter()
            .map(|definition| {
                (
                    definition.name.to_string(),
                    definition.to_string(),
                    definition.date_rule.to_string(),
                    definition
                        .get_feastrank::<Rank>()
                        .get_rank_string()
                        .to_string(),
                    definition.color.to_string(),
                    definition.titles.clone(),
                )
            })
            .collect();
        let mut actual: Vec<_> = sanctoral
            .iter()
            .chain(&temporal)
            .map(|feast| {
                (
                    feast.name.clone(),
                    feast.description.clone(),
                    feast.date_rule.clone(),
                    feast.rank.clone(),
                    feast.color.clone(),
                    feast.titles.clone(),
                )
            })
            .collect();
        expected.sort();
        actual.sort();
        assert_eq!(actual, expected);
        for feast in sanctoral.iter().chain(&temporal) {
            if let Some(date) = &feast.date {
                assert_eq!(
                    NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap().year(),
                    year
                );
            }
        }
        for cycle in [&sanctoral, &temporal] {
            assert!(
                cycle.windows(2).all(|pair| {
                    (&pair[0].date, &pair[0].name) <= (&pair[1].date, &pair[1].name)
                })
            );
        }
    }
}

fn assert_feast<Rank: FeastRankResolver + CalendarTypeProvider>(
    calendar: &GenericCalendar<Rank>,
    cycle: CalendarCycle,
    year: i32,
    name: &str,
    date: &str,
) -> CycleFeast {
    let feasts = calendar.cycle_feasts(cycle, year);
    let feast = feasts
        .into_iter()
        .find(|feast| feast.name == name)
        .unwrap_or_else(|| panic!("Missing {name} in {cycle:?} for {year}"));
    assert_eq!(feast.date.as_deref(), Some(date), "{name}");
    feast
}

fn load_us() -> GenericCalendar<FeastRankOf> {
    GenericCalendar::from_toml_with_extensions(
        data_path("of.toml"),
        &[data_path("of-us-extensions.toml")],
    )
    .unwrap()
}

#[test]
fn canonical_cycles_partition_source_definitions() {
    assert_partition(&load::<FeastRank54>("54.toml"));
    assert_partition(&load::<FeastRank62>("ef.toml"));
    assert_partition(&load::<FeastRank62>("62-monastic.toml"));
    assert_partition(&load::<FeastRankOf>("of.toml"));
    assert_partition(&load::<FeastRankOf>("of-us-extensions.toml"));
    assert_partition(&load_us());
}

#[test]
fn christmas_observances_are_temporal_not_sanctoral() {
    fn check<Rank: FeastRankResolver + CalendarTypeProvider>(
        calendar: &GenericCalendar<Rank>,
        cases: &[(&str, &str)],
    ) {
        for (name, date) in cases {
            assert_feast(calendar, CalendarCycle::Temporal, 2026, name, date);
            assert!(
                !calendar
                    .cycle_feasts(CalendarCycle::Sanctoral, 2026)
                    .iter()
                    .any(|feast| feast.name == *name)
            );
        }
    }
    check(
        &load::<FeastRank54>("54.toml"),
        &[
            ("The Nativity of Our Lord Jesus Christ", "2026-12-25"),
            (
                "Vigil of the Nativity of Our Lord Jesus Christ",
                "2026-12-24",
            ),
            ("The Circumcision of Our Lord", "2026-01-01"),
            ("Vigil of the Epiphany", "2026-01-05"),
            ("Epiphany of Our Lord", "2026-01-06"),
            ("The Most Holy Name of Jesus", "2026-01-04"),
        ],
    );
    for file in ["ef.toml", "62-monastic.toml"] {
        check(
            &load::<FeastRank62>(file),
            &[
                ("The Nativity of our Lord Jesus Christ", "2026-12-25"),
                (
                    "Vigil of the Nativity of our Lord Jesus Christ",
                    "2026-12-24",
                ),
                (
                    "Octave Day of the Nativity of our Lord Jesus Christ",
                    "2026-01-01",
                ),
                ("The Epiphany of our Lord", "2026-01-06"),
                (
                    "Commemoration of the Baptism of our Lord Jesus Christ",
                    "2026-01-13",
                ),
            ],
        );
    }
    assert_feast(
        &load::<FeastRank62>("ef.toml"),
        CalendarCycle::Temporal,
        2026,
        "The Most Holy Name of Jesus",
        "2026-01-04",
    );
    for calendar in [load::<FeastRankOf>("of.toml"), load_us()] {
        check(
            &calendar,
            &[
                ("The Nativity of our Lord Jesus Christ", "2026-12-25"),
                ("Solemnity of Mary, the Holy Mother of God", "2026-01-01"),
            ],
        );
    }
    check(
        &load::<FeastRankOf>("of.toml"),
        &[("The Epiphany of the Lord", "2026-01-06")],
    );
    check(&load_us(), &[("The Epiphany of the Lord", "2026-01-04")]);
}

#[test]
fn traditional_fixed_lord_feasts_remain_sanctoral() {
    assert_feast(
        &load::<FeastRank54>("54.toml"),
        CalendarCycle::Sanctoral,
        2026,
        "Transfiguration of our Lord Jesus Christ",
        "2026-08-06",
    );
    assert_feast(
        &load::<FeastRank54>("54.toml"),
        CalendarCycle::Sanctoral,
        2026,
        "Exaltation of the Holy Cross",
        "2026-09-14",
    );
    for file in ["ef.toml", "62-monastic.toml"] {
        let calendar = load::<FeastRank62>(file);
        assert_feast(
            &calendar,
            CalendarCycle::Sanctoral,
            2026,
            "The Transfiguration of our Lord Jesus Christ",
            "2026-08-06",
        );
        assert_feast(
            &calendar,
            CalendarCycle::Sanctoral,
            2026,
            "The Exaltation of the Holy Cross",
            "2026-09-14",
        );
    }
    for calendar in [load::<FeastRankOf>("of.toml"), load_us()] {
        assert_feast(
            &calendar,
            CalendarCycle::Sanctoral,
            2026,
            "The Transfiguration of the Lord",
            "2026-08-06",
        );
        assert_feast(
            &calendar,
            CalendarCycle::Sanctoral,
            2026,
            "The Exaltation of the Holy Cross",
            "2026-09-14",
        );
        assert_feast(
            &calendar,
            CalendarCycle::Sanctoral,
            2026,
            "The Most Holy Name of Jesus",
            "2026-01-03",
        );
    }
}

#[test]
fn movable_sanctoral_feasts_do_not_become_temporal() {
    let calendar = load::<FeastRank54>("54.toml");
    assert_feast(
        &calendar,
        CalendarCycle::Sanctoral,
        2026,
        "Seven Sorrows of the Blessed Virgin Mary",
        "2026-03-27",
    );
    assert_feast(
        &calendar,
        CalendarCycle::Sanctoral,
        2026,
        "Solemnity of St. Joseph",
        "2026-04-22",
    );
    assert_feast(
        &load::<FeastRank62>("ef.toml"),
        CalendarCycle::Sanctoral,
        2026,
        "Commemoration of the Seven Sorrows of the Blessed Virgin Mary",
        "2026-03-27",
    );
    for calendar in [load::<FeastRankOf>("of.toml"), load_us()] {
        assert_feast(
            &calendar,
            CalendarCycle::Sanctoral,
            2026,
            "Blessed Virgin Mary, Mother of the Church",
            "2026-05-25",
        );
        assert_feast(
            &calendar,
            CalendarCycle::Sanctoral,
            2026,
            "The Immaculate Heart of the Blessed Virgin Mary",
            "2026-06-13",
        );
        assert_feast(
            &calendar,
            CalendarCycle::Sanctoral,
            2026,
            "The Most Holy Name of Mary",
            "2026-09-12",
        );
    }
}

#[test]
fn therese_keeps_calendar_specific_source_dates() {
    let name = "St. Thérèse of the Child Jesus";
    for file in ["ef.toml", "62-monastic.toml"] {
        assert_feast(
            &load::<FeastRank62>(file),
            CalendarCycle::Sanctoral,
            2026,
            name,
            "2026-10-03",
        );
    }
    assert_feast(
        &load_us(),
        CalendarCycle::Sanctoral,
        2026,
        name,
        "2026-10-01",
    );
}

#[test]
fn sunday_superseded_saint_is_retained_at_its_source_date() {
    let calendar = load::<FeastRankOf>("of.toml");
    let date = NaiveDate::from_ymd_opt(2026, 10, 4).unwrap();
    assert_eq!(date.weekday(), Weekday::Sun);
    let resolved = calendar
        .instantiate_for_lit_year(2026)
        .get_day(date)
        .unwrap();
    assert!(!resolved.day.desc.contains("St. Francis of Assisi"));
    assert_feast(
        &calendar,
        CalendarCycle::Sanctoral,
        2026,
        "St. Francis of Assisi",
        "2026-10-04",
    );
}

#[test]
fn leap_conditional_saints_and_fixed_anchored_vigils_remain_sanctoral() {
    fn check<Rank: FeastRankResolver + CalendarTypeProvider>(calendar: &GenericCalendar<Rank>) {
        let leap = assert_feast(
            calendar,
            CalendarCycle::Sanctoral,
            2024,
            "St. Matthias",
            "2024-02-25",
        );
        assert_eq!(leap.date_rule, "(2/25) in leap year else (2/24)");
        assert_feast(
            calendar,
            CalendarCycle::Sanctoral,
            2026,
            "St. Matthias",
            "2026-02-24",
        );
    }
    let old = load::<FeastRank54>("54.toml");
    check(&old);
    check(&load::<FeastRank62>("ef.toml"));
    check(&load::<FeastRank62>("62-monastic.toml"));
    assert_feast(
        &old,
        CalendarCycle::Sanctoral,
        2024,
        "Vigil of St. Matthias",
        "2024-02-24",
    );
    assert_feast(
        &old,
        CalendarCycle::Sanctoral,
        2026,
        "Vigil of St. Matthias",
        "2026-02-23",
    );
    assert_feast(
        &old,
        CalendarCycle::Sanctoral,
        2026,
        "Vigil of St. Matthew",
        "2026-09-20",
    );
}

#[test]
fn us_merge_retains_base_overrides_and_adds_local_feria_override() {
    let base = load::<FeastRankOf>("of.toml");
    let extension = load::<FeastRankOf>("of-us-extensions.toml");
    let merged = load_us();
    for (name, cycle) in base
        .cycle_overrides
        .iter()
        .chain(&extension.cycle_overrides)
    {
        assert_eq!(merged.cycle_overrides.get(name), Some(cycle));
    }
    let name = "Day of Prayer for the Legal Protection of Unborn Children";
    for calendar in [&extension, &merged] {
        assert_feast(calendar, CalendarCycle::Temporal, 2026, name, "2026-01-22");
        assert_feast(calendar, CalendarCycle::Temporal, 2023, name, "2023-01-23");
        assert!(
            !calendar
                .cycle_feasts(CalendarCycle::Sanctoral, 2026)
                .iter()
                .any(|feast| feast.name == name)
        );
    }
    let base_epiphany = assert_feast(
        &base,
        CalendarCycle::Temporal,
        2026,
        "The Epiphany of the Lord",
        "2026-01-06",
    );
    let us_epiphany = assert_feast(
        &merged,
        CalendarCycle::Temporal,
        2026,
        "The Epiphany of the Lord",
        "2026-01-04",
    );
    assert_ne!(base_epiphany.date_rule, us_epiphany.date_rule);
    assert_eq!(
        merged
            .feasts
            .iter()
            .filter(|feast| feast.name.as_ref() == "The Epiphany of the Lord")
            .count(),
        1
    );
    assert_feast(
        &merged,
        CalendarCycle::Sanctoral,
        2026,
        "St. Vincent",
        "2026-01-23",
    );
}

#[test]
fn computed_ferias_are_absent_but_explicit_source_ferias_are_retained() {
    let calendar = load::<FeastRankOf>("of.toml");
    let resolved = calendar.instantiate_for_lit_year(2026);
    let day = resolved
        .get_day(NaiveDate::from_ymd_opt(2026, 1, 12).unwrap())
        .unwrap();
    for cycle in [CalendarCycle::Sanctoral, CalendarCycle::Temporal] {
        let feasts = calendar.cycle_feasts(cycle, 2026);
        assert!(
            !feasts
                .iter()
                .any(|feast| feast.date.as_deref() == Some("2026-01-12"))
        );
        assert!(
            !feasts
                .iter()
                .any(|feast| feast.description == day.day.desc.as_ref())
        );
    }
    assert_feast(
        &calendar,
        CalendarCycle::Temporal,
        2026,
        "Ash Wednesday",
        "2026-02-18",
    );
}
