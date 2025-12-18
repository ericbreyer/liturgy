use std::fmt::Debug;
mod office_component;
pub mod ordo_repo;
mod vespers;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use types::{DayDescription, DayRank, DayRank62, TrivialDayRank};

pub use crate::{ordo_repo::OrdoRepo, vespers::Vespers};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    fn vespers_location_62(&self, day: &DayDescription<DayRank62>) -> Result<(Vespers, Vec<String>)>;
}

pub trait VespersOrdo {
    fn vespers_ordo (
        &self,
        repo: &OrdoRepo,
    ) -> Result<Vespers>;
    fn vespers_ordo_sources (
        &self,
        repo: &OrdoRepo,
    ) -> Result<Vec<String>>;
}

impl VespersOrdo for DayDescription<DayRank62>
{
    fn vespers_ordo (
        &self,
        repo: &OrdoRepo,
    ) -> Result<Vespers> {
        Ok(repo.vespers_location_62(self)?.0)
    }

    fn vespers_ordo_sources (
        &self,
        repo: &OrdoRepo,
    ) -> Result<Vec<String>> {
        Ok(repo.vespers_location_62(self)?.1)
    }
}

impl VespersOrdo for DayDescription<TrivialDayRank>
{
    fn vespers_ordo (
        &self,
        _repo: &OrdoRepo,
    ) -> Result<Vespers> {
        Err(anyhow::anyhow!("VespersOrdo not implemented for DayRank other than DayRank62"))
    }

    fn vespers_ordo_sources (
        &self,
        _repo: &OrdoRepo,
    ) -> Result<Vec<String>> {
        Err(anyhow::anyhow!("VespersOrdo not implemented for DayRank other than DayRank62"))
    }
}