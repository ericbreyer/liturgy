use std::fmt::Debug;

use chrono::NaiveDate;
use serde::{Serialize, ser::SerializeStruct as _};

use crate::{ArcStr, LiturgicalUnit};

pub trait DayRank: Clone + std::fmt::Debug + Serialize + Send + Sync {
    fn as_str(&self) -> &str;
}

#[derive(Clone, Debug, Serialize)]
pub struct TrivialDayRank(pub ArcStr);

impl DayRank for TrivialDayRank {
    fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(Clone, Debug)]
pub struct DayRank62 {
    pub office: DayRank62Office,
    pub description: ArcStr,
}

impl Serialize for DayRank62 {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Clone, Debug, Serialize)]
pub enum DayRank62Office {
    Sunday,
    Feastial,
    Semifestial,
    Ordinary,
    Ferial,
}

impl DayRank for DayRank62 {
    fn as_str(&self) -> &str {
        self.description.as_ref()
    }
}

impl DayRank62 {
    #[must_use]
    pub fn new(office: DayRank62Office, description: &str) -> Self {
        DayRank62 {
            office,
            description: ArcStr::from(description),
        }
    }

    #[must_use]
    pub fn has_first_vespers(&self) -> bool {
        matches!(
            self.office,
            DayRank62Office::Sunday | DayRank62Office::Feastial
        )
    }
}

#[derive(Clone, Default)]
pub enum ConcuringVespersAction {
    #[default]
    Use,
    Commemorate,
    UseCommemorateSelf,
}

impl Debug for ConcuringVespersAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConcuringVespersAction::Use => write!(f, "Vespers of Following"),
            ConcuringVespersAction::Commemorate => write!(f, "Commemorate Vespers of Following"),
            ConcuringVespersAction::UseCommemorateSelf => {
                write!(f, "Vespers of Following, Commemorate Own Vespers")
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommemorationType {
    Optional,
    Lauds,
    LaudsAndVespers,
    PeterAndPaulSpecial,
}

#[derive(Debug, Clone)]
pub struct DayDescription<R: DayRank> {
    pub date: NaiveDate,

    pub day_in_season: ArcStr,

    pub day: LiturgicalUnit<R>,
    pub commemorations: Vec<(LiturgicalUnit<R>, CommemorationType)>,
    pub concuring_vespers: Option<(LiturgicalUnit<R>, ConcuringVespersAction)>,
    pub underlying_octave: Option<String>,

    pub season: ArcStr,
}

impl<R> DayDescription<R>
where
    R: DayRank,
{
    pub fn add_concuring_vespers(
        self,
        vespers: LiturgicalUnit<R>,
        action: ConcuringVespersAction,
    ) -> Self {
        Self {
            concuring_vespers: Some((vespers, action)),
            ..self
        }
    }
}

impl<R> Serialize for DayDescription<R>
where
    R: DayRank,
{
    // Custom serialization to handle LiturgicalUnit serialization
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Keep external serialized shape stable (date, day_in_season, day_rank, day,
        // commemorations)
        let mut state = serializer.serialize_struct("DayDescription", 5)?;
        state.serialize_field("date", &self.date.to_string())?;
        state.serialize_field("day_in_season", self.day_in_season.as_ref())?;
        state.serialize_field("day", &self.day)?;
        state.serialize_field("commemorations", &self.commemorations)?;
        state.end()
    }
}
