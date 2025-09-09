use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DayType {
    Octave,
    Feria,
    Feast,
    Sunday,
    Vigil,
}
