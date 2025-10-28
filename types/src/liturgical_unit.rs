use std::fmt::Debug;

use chrono::NaiveDate;
use serde::{Serialize, ser::SerializeStruct};

use crate::{ArcStr, DayKind, DayRank};

#[derive(Clone)]
pub struct LiturgicalUnit<R: DayRank> {
    pub desc: ArcStr,
    pub rank: R,
    pub date: NaiveDate,
    pub color: ArcStr,
    pub day_kind: DayKind,
    pub titles: Vec<ArcStr>,
}

impl<R> Debug for LiturgicalUnit<R>
where
    R: DayRank,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LiturgicalUnit")
            .field("desc", &self.desc)
            .field("rank", &self.rank)
            .field("day_kind", &self.day_kind)
            .field("date", &self.date)
            .field("color", &self.color)
            .finish()
    }
}

impl<R> Serialize for LiturgicalUnit<R>
where
    R: DayRank,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // desc, rank, date, color, day_kind
        let mut state = serializer.serialize_struct("LiturgicalUnit", 5)?;
        state.serialize_field("desc", &self.desc)?;
        state.serialize_field("rank", &self.rank)?;
        state.serialize_field("date", &self.date.to_string())?;
        state.serialize_field("color", &self.color)?;
        state.serialize_field("day_kind", &format!("{:?}", &self.day_kind))?;
        state.end()
    }
}

impl<R> LiturgicalUnit<R>
where
    R: DayRank,
{
    pub fn transfered(&self) -> Self {
        Self {
            desc: format!("{} (transferred)", self.desc).into(),
            rank: self.rank.clone(),
            date: self.date,
            color: self.color.clone(),
            day_kind: self.day_kind.clone(),
            titles: self.titles.clone(),
        }
    }
}
