use std::fmt::Debug;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use types::{ArcStr, RcStr};
use crate::{calender::DayType, };
mod feast_rank54;
mod feast_rank62;
mod feast_rank_of;
mod test;
pub use feast_rank54::FeastRank54;
pub use feast_rank62::FeastRank62;
pub use feast_rank_of::FeastRankOf;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
    /// Flags describing octave-related properties for a liturgical day
    pub struct OctaveFlags: u8 {
        const OCTAVE_DAY = 0b00000001;
        const FIRST_3_DAYS = 0b00000010;
    }
}

impl Default for OctaveFlags {
    fn default() -> Self {
        OctaveFlags::empty()
    }
}

/// Context information for creating FeastRank62 from legacy data
#[derive(Debug, Clone, Default)]
pub struct LiturgicalContext {
    /// The season name (e.g., "Lent", "Advent", "Ordinary Time")
    season_name: Option<String>,
    /// The feast name (used to detect special cases like Immaculate Conception)
    feast_name: Option<String>,
    /// Whether this feast is movable (depends on Easter)
    is_movable: bool,
    /// Whether this feast is of Our Lord
    of_our_lord: bool,
    of_lent: bool,
    secondary_day_type: Option<DayType>,
    octave_flags: OctaveFlags,
    /// Mark that this Sunday is the special Easter or Pentecost Sunday which
    /// should behave like a First-class Sunday but not admit commemorations.
    is_easter_or_pentecost: bool,
}

impl LiturgicalContext {
    /// Create a new context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the season name
    pub fn season<S: Into<String>>(mut self, name: S) -> Self {
        self.season_name = Some(name.into());
        self
    }

    /// Set the feast name
    pub fn feast<S: Into<String>>(mut self, name: S) -> Self {
        self.feast_name = Some(name.into());
        if self.feast_name.as_deref() == Some("Easter Sunday") || self.feast_name.as_deref() == Some("Pentecost Sunday") {
            self.is_easter_or_pentecost = true;
        }
        self
    }

    /// Mark as movable (depends on Easter)
    pub fn movable(mut self) -> Self {
        self.is_movable = true;
        self
    }

    pub fn octave_day(mut self, is_octave_day: bool) -> Self {
        if is_octave_day {
            self.octave_flags.insert(OctaveFlags::OCTAVE_DAY);
        } else {
            self.octave_flags.remove(OctaveFlags::OCTAVE_DAY);
        }
        self
    }

    /// Mark that this day is within the first three days of an octave
    pub fn first_3_days(mut self, is_first_3_days: bool) -> Self {
        if is_first_3_days {
            self.octave_flags.insert(OctaveFlags::FIRST_3_DAYS);
        } else {
            self.octave_flags.remove(OctaveFlags::FIRST_3_DAYS);
        }
        self
    }

    /// Mark as feast of Our Lord
    pub fn of_our_lord(mut self) -> Self {
        self.of_our_lord = true;
        self
    }

    /// Mark that this context represents Easter or Pentecost Sunday behavior
    pub fn easter_or_pentecost(mut self, v: bool) -> Self {
        self.is_easter_or_pentecost = v;
        self
    }

    /// Mark as feast of Lent
    pub fn of_lent(mut self, v: bool) -> Self {
        self.of_lent = v;
        self
    }

    pub fn also_ferial(mut self) -> Self {
        self.secondary_day_type = Some(DayType::Feria);
        self
    }

    pub fn also_sunday(mut self) -> Self {
        self.secondary_day_type = Some(DayType::Sunday);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveConflictsResult<R: FeastRank, T: Clone> {
    pub winner: T,
    pub winner_rank: R,
    pub transferred: Option<(R, T)>,
    pub commemorations: Vec<T>,
    pub debug_trace: Vec<String>,
}

pub enum BVMOnSaturdayResult {
    /// The rank does not admit BVM on Saturday
    NotAdmitted,
    /// The rank admits BVM on Saturday, and this is the rank to use
    Admitted,
    /// The rank admits BVM on Saturday, but this is a feast of the Lord that takes precedence
    Commemorated,
    /// The rank admits BVM on Saturday, and the current feast is commemorated
    OtherCommemorated,
}
pub trait FeastRank: Clone + Debug {
    fn resolve_conflicts<T>(competetors: &[(Self, T)]) -> Result<ResolveConflictsResult<Self, T>>
    where
        // Self: Sized,
        T: Clone + Debug;
    fn new_with_context(rank: &str, day_type: &DayType, context: &LiturgicalContext) -> Self
    where
        Self: Sized;
    fn is_ferial_or_sunday_rank(&self) -> bool;
    fn is_high_festial(&self) -> bool;
    fn get_rank_string(&self) -> ArcStr;
    fn get_bvm_on_saturday_rank() -> Option<Self>
    where
        Self: Sized;
    fn admits_bvm_on_saturday(&self) -> BVMOnSaturdayResult;
    fn id(&self) -> RcStr;
    /// Whether vigils that fall on Sunday should be transferred to the previous Saturday.
    /// Default is false; the 1954 implementation opts in.
    fn transfers_vigil_from_sunday_to_saturday() -> bool
    where
        Self: Sized,
    {
        false
    }
}
