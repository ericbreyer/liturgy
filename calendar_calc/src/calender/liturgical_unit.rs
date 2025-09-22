use std::fmt::Debug;

use chrono::NaiveDate;
use serde::{ser::SerializeStruct, Serialize};

use crate::{calender::feast_rank::FeastRank, types::ArcStr};

#[derive(Clone)]
pub struct LiturgicalUnit {
    pub desc: ArcStr,
    pub rank: ArcStr,
    pub date: NaiveDate,
    pub color: ArcStr,
}

impl Debug for LiturgicalUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LiturgicalUnit")
            .field("desc", &self.desc)
            .field("rank", &self.rank)
            .finish()
    }
}

impl Serialize for LiturgicalUnit {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("LiturgicalUnit", 3)?;
        state.serialize_field("desc", &self.desc)?;
        state.serialize_field("rank", &self.rank)?;
        state.serialize_field("date", &self.date.to_string())?;
        state.serialize_field("color", &self.color)?;
        state.end()
    }
}

impl LiturgicalUnit {
    pub fn transfered(&self) -> Self {
        Self {
            desc: format!("{} (transferred)", self.desc).into(),
            rank: self.rank.clone(),
            date: self.date,
            color: self.color.clone(),
        }
    }

    pub fn bvm_on_saturday<R: FeastRank>(&mut self) {
        self.desc = "BVM on Saturday".into();
        self.rank = R::get_bvm_on_saturday_rank().unwrap().get_rank_string();
    }

    pub fn bvm_on_saturday_commemoration<R: FeastRank>(date: NaiveDate) -> Self {
        Self {
            desc: "BVM on Saturday".into(),
            rank: R::get_bvm_on_saturday_rank().unwrap().get_rank_string(),
            date,
            color: "white".into(),
        }
    }
}
