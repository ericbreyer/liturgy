use std::fmt::Debug;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use types::{ArcStr, CommemorationType, DayRank, RcStr};

use crate::calender::DayType;
mod feast_rank_54;
mod feast_rank_62;
mod feast_rank_of;
mod test;
pub use feast_rank_54::FeastRank54;
pub use feast_rank_62::FeastRank62;
pub use feast_rank_of::FeastRankOf;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
    /// Flags describing octave-related properties for a liturgical day
    pub struct OctaveFlags: u8 {
        const OCTAVE_DAY = 0b01;
        const FIRST_3_DAYS = 0b10;
    }
}

impl Default for OctaveFlags {
    fn default() -> Self {
        OctaveFlags::empty()
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
    /// Flags used for Feria (weekday) special cases across feast rank implementations
    pub struct FeriaFlags: u8 {
        const OF_LENT = 0b01;
        const LENT = 0b01;

        const EMBER_DAY = 0b010;
        const HOLY_TRIDUUM = 0b100;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
    /// Flags describing properties of a feast
    pub struct FeastFlags: u8 {
        // Both names exist historically in submodules; keep both for backwards
        // compatibility (they map to the same mask).
        const OF_OUR_LORD = 0b0001;
        const OF_THE_LORD = 0b0001;

        const IMMACULATE_CONCEPTION = 0b0010;
        const MOVABLE = 0b0100;
        const ALL_SOULS = 0b1000;
        const AQUIRED_FIRST_VESPERS = 0b1_0000;
        const OF_PETER_AND_PAUL = 0b10_0000;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
    /// Flags describing Sunday-specific properties
    pub struct SundayFlags: u8 {
        const WAS_OCTAVE = 0b01;
        const EASTER_OR_PENTECOST = 0b10;
    }
}

/// Context information for creating `FeastRank62` from legacy data
#[derive(Debug, Clone, Default)]
#[allow(clippy::struct_excessive_bools)]
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
    #[must_use]
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
        if self.feast_name.as_deref() == Some("Easter Sunday")
            || self.feast_name.as_deref() == Some("Pentecost Sunday") || self.feast_name.as_deref() == Some("Holy Trinity and the Octave of Pentecost")
        {
            self.is_easter_or_pentecost = true;
        }
        self
    }

    /// Mark as movable (depends on Easter)
    #[must_use]
    pub fn movable(mut self) -> Self {
        self.is_movable = true;
        self
    }

    #[must_use]
    pub fn octave_day(mut self, is_octave_day: bool) -> Self {
        if is_octave_day {
            self.octave_flags.insert(OctaveFlags::OCTAVE_DAY);
        } else {
            self.octave_flags.remove(OctaveFlags::OCTAVE_DAY);
        }
        self
    }

    /// Mark that this day is within the first three days of an octave
    #[must_use]
    pub fn first_3_days(mut self, is_first_3_days: bool) -> Self {
        if is_first_3_days {
            self.octave_flags.insert(OctaveFlags::FIRST_3_DAYS);
        } else {
            self.octave_flags.remove(OctaveFlags::FIRST_3_DAYS);
        }
        self
    }

    /// Mark as feast of Our Lord
    #[must_use]
    pub fn of_our_lord(mut self) -> Self {
        self.of_our_lord = true;
        self
    }

    /// Mark that this context represents Easter or Pentecost Sunday behavior
    #[must_use]
    pub fn easter_or_pentecost(mut self, v: bool) -> Self {
        self.is_easter_or_pentecost = v;
        self
    }

    /// Mark as feast of Lent
    #[must_use]
    pub fn of_lent(mut self, v: bool) -> Self {
        self.of_lent = v;
        self
    }

    #[must_use]
    pub fn also_ferial(mut self) -> Self {
        self.secondary_day_type = Some(DayType::Feria);
        self
    }

    #[must_use]
    pub fn also_sunday(mut self) -> Self {
        self.secondary_day_type = Some(DayType::Sunday);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveConflictsResult<R: FeastRankResolver, T: Clone> {
    pub winner: T,
    pub winner_rank: R,
    pub transferred: Option<(R, T)>,
    pub commemorations: Vec<(T, CommemorationType)>,
}

impl<R, T> ResolveConflictsResult<R, T>
where
    R: FeastRankResolver,
    T: Clone,
{
    pub fn add_commemoration_lauds(&mut self, unit: T) {
        self.commemorations.push((unit, CommemorationType::Lauds));
    }
    pub fn add_commemoration_special(&mut self, unit: T, special: CommemorationType) {
        self.commemorations.push((unit, special));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveConcurancesResult {
    VespersOfCurrentNothingOfFollowing,
    VespersOfCurrentComemorationOfNextDay,
    VespersOfFollowingCommemorationOfCurrent,
    VespersOfFollowingNothingOfCurrent,
}

pub enum BVMOnSaturdayResult {
    /// The rank does not admit BVM on Saturday
    NotAdmitted,
    /// The rank admits BVM on Saturday, and this is the rank to use
    Admitted,
    /// The rank admits BVM on Saturday, but this is a feast of the Lord that
    /// takes precedence
    Commemorated,
    /// The rank admits BVM on Saturday, and the current feast is commemorated
    OtherCommemorated,
}
pub trait FeastRankResolver: Clone + Debug {
    type FeastRankDescriptor: DayRank;

    fn resolve_conflicts<T>(competetors: &[(Self, T)]) -> Result<ResolveConflictsResult<Self, T>>
    where
        T: Clone + Debug;
    fn resolve_concurances(primary: Self, secondary: Self) -> Result<ResolveConcurancesResult>;
    fn new_with_context(rank: &str, day_type: DayType, context: &LiturgicalContext) -> Self;

    fn is_ferial_or_sunday_rank(&self) -> bool;
    fn is_high_festial(&self) -> bool;
    fn get_rank_string(&self) -> ArcStr;

    fn get_bvm_on_saturday_rank() -> Self;
    fn admits_bvm_on_saturday(&self) -> BVMOnSaturdayResult;
    #[must_use]
    fn get_peter_and_paul_commemoration_rank() -> Self {
        Self::get_bvm_on_saturday_rank()
    }

    fn id(&self) -> RcStr;
    fn descriptor(&self) -> Self::FeastRankDescriptor;
    #[must_use]
    fn transfers_vigil_from_sunday_to_saturday() -> bool {
        false
    }
}
