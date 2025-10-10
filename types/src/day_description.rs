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

#[derive(Debug, Clone)]
pub struct DayDescription<R: DayRank> {
    pub date: NaiveDate,
    pub day_in_season: ArcStr,
    pub day_rank: R,
    pub day: LiturgicalUnit<R>,
    pub commemorations: Vec<LiturgicalUnit<R>>,
    pub debug_trace: ArcStr,
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
        let mut state = serializer.serialize_struct("DayDescription", 5)?;
        state.serialize_field("date", &self.date.to_string())?;
        state.serialize_field("day_in_season", self.day_in_season.as_ref())?;
        state.serialize_field("day_rank", &self.day_rank)?;
        state.serialize_field("day", &self.day)?;
        state.serialize_field("commemorations", &self.commemorations)?;
        state.end()
    }
}
