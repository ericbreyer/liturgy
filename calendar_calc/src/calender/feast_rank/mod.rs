use std::fmt::Debug;

use crate::calender::DayType;
mod feast_rank54;
mod feast_rank62;
mod feast_rank_of;
pub use feast_rank54::FeastRank54;
pub use feast_rank62::FeastRank62;
pub use feast_rank_of::FeastRankOf;

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
    is_octave_day: bool,
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
        self
    }

    /// Mark as movable (depends on Easter)
    pub fn movable(mut self) -> Self {
        self.is_movable = true;
        self
    }

    pub fn octave_day(mut self, is_octave_day: bool) -> Self {
        self.is_octave_day = is_octave_day;
        self
    }

    /// Mark as feast of Our Lord
    pub fn of_our_lord(mut self) -> Self {
        self.of_our_lord = true;
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

pub struct ResolveConflictsResult<R: FeastRank, T: Clone> {
    pub winner: T,
    pub winner_rank: R,
    pub transferred: Option<(R, T)>,
    pub commemorations: Vec<T>,
}

pub enum BVMOnSaturdayResult {
    /// The rank does not admit BVM on Saturday
    NotAdmitted,
    /// The rank admits BVM on Saturday, and this is the rank to use
    Admitted,
    /// The rank admits BVM on Saturday, but this is a feast of the Lord that takes precedence
    Commemorated,
}
pub trait FeastRank: Clone + Debug {
    fn resolve_conflicts<T>(competetors: &[(Self, T)]) -> ResolveConflictsResult<Self, T>
    where
        // Self: Sized,
        T: Clone + Debug;
    fn new_with_context(rank: &str, day_type: &DayType, context: &LiturgicalContext) -> Self
    where
        Self: Sized;
    fn is_ferial_or_sunday_rank(&self) -> bool;
    fn is_high_festial(&self) -> bool;
    fn get_rank_string(&self) -> String;
    fn get_bvm_on_saturday_rank() -> Option<Self>
    where
        Self: Sized;
    fn admits_bvm_on_saturday(&self) -> BVMOnSaturdayResult;
}
