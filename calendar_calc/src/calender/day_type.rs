use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, Copy)]
#[serde(rename_all = "lowercase")]
pub enum DayType {
    Octave,
    Feria,
    #[default]
    Feast,
    Sunday,
    Vigil,
}

impl DayType {
    pub fn is_vigil(self) -> bool {
        matches!(self, DayType::Vigil)
    }
}
