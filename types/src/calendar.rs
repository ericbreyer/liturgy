use serde::{Deserialize, Serialize};

/// Shared calendar type used across crates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalendarType {
    /// 1954 Roman Calendar
    Calendar1954,
    /// 1962 Roman Calendar (Extraordinary Form)
    Calendar1962,
    /// Ordinary Form (Post-Vatican II)
    OrdinaryForm,
}
