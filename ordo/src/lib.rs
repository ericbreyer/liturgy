use std::fmt::Debug;
mod office_component;
pub mod ordo_repo;
mod vespers;

use anyhow::Result;
use types::{DayDescription, DayRank62};

use crate::{ordo_repo::OrdoRepo, vespers::Vespers};

#[derive(Clone, PartialEq)]
pub enum Location {
    /// Common, optionally carrying a name supplied by a rule (empty string if
    /// unnamed)
    Common(String),
    Proper,
    Ordinary(String),
    Octave(String),
    Psalter,
    Sunday(Option<String>),
}

impl Debug for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Location::Common(s) if s.is_empty() => write!(f, "Common"),
            Location::Common(s) => write!(f, "Common of {s}"),
            Location::Proper => write!(f, "Proper"),
            Location::Psalter => write!(f, "Psalter"),
            Location::Ordinary(s) => write!(f, "Psalter (Ordinary of {s})"),
            Location::Sunday(Some(s)) => write!(f, "of Sunday ({s})"),
            Location::Sunday(None) => write!(f, "of Sunday"),
            Location::Octave(s) => write!(f, "of {s}"),
        }
    }
}

/// Trait describing how to resolve Ordo locations for a given day and vespers
/// component. Implement this to provide day-by-day overrides (for example when
/// a day has specific propers).
trait OrdoRules {
    fn vespers_location(&self, day: &DayDescription<DayRank62>) -> Result<(Vespers, Vec<String>)>;
}

/// Build a vespers representation for a day and return a debug string.
/// This is a small public helper used by integration tests to snapshot
/// full-year ordos.
#[must_use]
pub fn build_vespers_snapshot(
    day: &DayDescription<DayRank62>,
    repo: &OrdoRepo,
) -> Result<(String, Vec<String>)> {
    let v = repo.vespers_location(day)?;

    Ok((format!("{}\n{}", day.date, v.0), v.1))
}
