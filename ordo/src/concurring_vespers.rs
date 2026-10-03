//! Concurring vespers resolution logic.
//!
//! This module handles the complex logic of determining which day's office
//! to use when vespers from different days can occur at the same time
//! (e.g., first vespers of one feast vs second vespers of another).

use types::{CommemorationType, DayDescription, DayRank62, LiturgicalUnit};

/// Represents a resolved concurring vespers scenario.
///
/// This encapsulates the decision about which day's office to use and which
/// days to commemorate, making the logic explicit and testable.
#[derive(Debug, Clone)]
pub struct ConcurringVespersResolution<'a> {
    /// Which day's office to use for the main vespers
    pub day_used: &'a LiturgicalUnit<DayRank62>,
    /// Optional day to commemorate (if different from day_used)
    pub day_commemorated: Option<&'a LiturgicalUnit<DayRank62>>,
    /// Whether this is first vespers (vs second or ordinary)
    pub is_first_vespers: bool,
}

impl<'a> ConcurringVespersResolution<'a> {
    /// Resolve concurring vespers logic for a calendar day.
    ///
    /// # Arguments
    /// - `day_description`: The full calendar day with concurring vespers info
    ///
    /// # Returns
    /// A resolved structure indicating which day to use and whether it's first vespers
    pub fn resolve(day_description: &'a DayDescription<DayRank62>) -> Self {
        // Step 1: Determine which day to use and which to commemorate
        let (day_used, day_commemorated, use_concurring) =
            if let Some((cv, ca)) = &day_description.concuring_vespers {
                use types::ConcuringVespersAction;
                match ca {
                    ConcuringVespersAction::Use => {
                        // Use the concurring day's office, don't commemorate anything
                        (cv, None, true)
                    }
                    ConcuringVespersAction::Commemorate => {
                        // Use the primary day's office, commemorate the concurring day
                        (&day_description.day, Some(cv), false)
                    }
                    ConcuringVespersAction::UseCommemorateSelf => {
                        // Use the concurring day's office, commemorate the primary day
                        (cv, Some(&day_description.day), true)
                    }
                }
            } else {
                // No concurring vespers; use the primary day, no secondary commemoration
                (&day_description.day, None, false)
            };

        // Step 2: Determine if this is first vespers
        // First vespers occurs when we're using a different day than the primary day
        let is_first_vespers = use_concurring;

        Self {
            day_used,
            day_commemorated,
            is_first_vespers,
        }
    }

    /// Generate the description string for this vespers (e.g., "First Vespers of Saint X")
    pub fn description(&self) -> String {
        if self.is_first_vespers {
            format!("First Vespers of {}", self.day_used.desc.as_ref())
        } else if self.day_used.rank.has_first_vespers() {
            format!("Second Vespers of {}", self.day_used.desc.as_ref())
        } else {
            format!("Vespers of {}", self.day_used.desc.as_ref())
        }
    }

    /// Predicate: should we include the primary day's committal commemorations?
    ///
    /// We include them if:
    /// - This is first vespers (we're not on the primary day itself), OR
    /// - There's no secondary concurring day to commemorate (the primary day is all we have)
    pub fn should_include_day_commemorations(&self) -> bool {
        self.is_first_vespers || self.day_commemorated.is_none()
    }

    /// Get the commemoration type to use for any commemorated days.
    ///
    /// Currently always LaudsAndVespers; could be extended for other types.
    pub fn commemoration_type(&self) -> CommemorationType {
        CommemorationType::LaudsAndVespers
    }
}
