use chrono::NaiveDate;
use serde::{Serialize, ser::SerializeStruct};

use crate::calender::feast_rank::FeastRank;

#[derive(Debug, Clone,)]
pub struct LiturgicalUnit {
    pub desc: String,
    pub rank: String,
    pub date: NaiveDate,
    pub color: String,
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

impl LiturgicalUnit
{
    pub fn transfered(&self) -> Self {
        Self {
            desc: format!("{} (transferred)", self.desc),
            rank: self.rank.clone(),
            date: self.date,
            color: self.color.clone(),
        }
    }

    pub fn bvm_on_saturday(&mut self) {
        self.desc = "BVM on Saturday".to_string();
    }

    pub fn bvm_on_saturday_commemoration<R: FeastRank>(date: NaiveDate) -> Self {
        Self {
            desc: "BVM on Saturday".to_string(),
            rank: R::get_bvm_on_saturday_rank().unwrap().get_rank_string(),
            date,
            color: "white".to_string(),
        }
    }
}
