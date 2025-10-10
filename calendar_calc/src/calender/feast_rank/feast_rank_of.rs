use std::fmt::Debug;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use types::{ArcStr, RcStr, TrivialDayRank};

use super::{
    DayType, FeastFlags, FeastRankResolver, FeriaFlags, LiturgicalContext, ResolveConflictsResult,
    SundayFlags,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeastRankOf(FeastRankOfInner);

impl FeastRankResolver for FeastRankOf {
    type FeastRankDescriptor = TrivialDayRank;
    fn resolve_conflicts<T>(competetors: &[(Self, T)]) -> Result<ResolveConflictsResult<Self, T>>
    where
        Self: Sized,
        T: Clone + Debug,
    {
        FeastRankOfInner::resolve_conflicts(
            competetors
                .iter()
                .map(|(f, n)| (f.0.clone(), n.clone()))
                .collect::<Vec<_>>()
                .as_slice(),
        )
    }

    fn new_with_context(rank: &str, day_type: &DayType, context: &LiturgicalContext) -> Self
    where
        Self: Sized,
    {
        FeastRankOf(FeastRankOfInner::new_with_context(rank, day_type, context))
    }

    fn is_ferial_or_sunday_rank(&self) -> bool {
        matches!(
            self.0,
            FeastRankOfInner::Feria { .. } | FeastRankOfInner::Sunday { .. }
        )
    }

    fn is_high_festial(&self) -> bool {
        matches!(self.0, FeastRankOfInner::Feast { rank: 1..=2, .. })
    }

    fn get_rank_string(&self) -> ArcStr {
        self.0.get_rank_string()
    }

    fn get_bvm_on_saturday_rank() -> Option<Self>
    where
        Self: Sized,
    {
        Some(FeastRankOf(FeastRankOfInner::Feast {
            rank: 4,
            flags: FeastFlags::empty(),
        }))
    }

    fn admits_bvm_on_saturday(&self) -> super::BVMOnSaturdayResult {
        // admits a commemoration if this is a feria of rank 4
        if let FeastRankOfInner::Feria { rank: 4, .. } = self.0 {
            super::BVMOnSaturdayResult::Commemorated
        } else {
            super::BVMOnSaturdayResult::NotAdmitted
        }
    }

    fn id(&self) -> RcStr {
        self.0.get_rank_string_verbose().into()
    }

    fn descriptor(&self) -> Self::FeastRankDescriptor {
        TrivialDayRank(self.0.get_rank_string())
    }
}

// Using shared FeastFlags, FerialFlags and SundayFlags from parent module

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum FeastRankOfInner {
    /// Feast - any liturgical feast (Solemnity=1, Feast=2, Memorial=3, Optional=4)
    Feast { rank: u8, flags: FeastFlags },
    /// Sunday - liturgical rank varies by season
    Sunday { rank: u8, flags: SundayFlags }, // 1=highest (like Easter), 2=major season, 3=ordinary time
    /// Feria - weekday with rank based on season
    Feria { rank: u8, flags: FeriaFlags }, // 1=Ash Wed, Good Friday, 2=Lent/Advent, 3=Ordinary Time
}

#[derive(Debug, Clone, PartialEq)]
enum OccurrenceResult {
    FirstWins,
    SecondWins,
    SecondWinsFirstTransferred,
    FirstWinsSecondTransferred,
    FirstWinsSecondCommemoration,
    SecondWinsFirstCommemoration,
    CommemorateBoth,
}

impl OccurrenceResult {
    /// Swap the perspective of this result (first becomes second, second becomes first)
    fn swap(self) -> Self {
        match self {
            OccurrenceResult::FirstWins => OccurrenceResult::SecondWins,
            OccurrenceResult::SecondWins => OccurrenceResult::FirstWins,
            OccurrenceResult::SecondWinsFirstTransferred => {
                OccurrenceResult::FirstWinsSecondTransferred
            }
            OccurrenceResult::FirstWinsSecondTransferred => {
                OccurrenceResult::SecondWinsFirstTransferred
            }
            OccurrenceResult::FirstWinsSecondCommemoration => {
                OccurrenceResult::SecondWinsFirstCommemoration
            }
            OccurrenceResult::SecondWinsFirstCommemoration => {
                OccurrenceResult::FirstWinsSecondCommemoration
            }
            OccurrenceResult::CommemorateBoth => OccurrenceResult::CommemorateBoth,
        }
    }
}

impl FeastRankOfInner {
    fn resolve_conflicts<T: Clone + Debug>(
        competetors: &[(Self, T)],
    ) -> Result<ResolveConflictsResult<FeastRankOf, T>> {
        let mut debug_trace = Vec::new();
        if competetors.is_empty() {
            bail!("No competitors provided for conflict resolution");
        }

        let mut sorted_competetors = competetors.to_vec();
        // sorted_competetors.sort_by(|(rank_a, _), (rank_b, _)| {
        //     rank_a.get_numeric_rank().cmp(&rank_b.get_numeric_rank())
        // });

        // any 4th class feast automatically is a commemoration
        let mut base_commemorations = Vec::new();
        sorted_competetors.retain(|(rank, name)| {
            if let FeastRankOfInner::Feast { rank: 4, .. } = *rank {
                base_commemorations.push(name.clone());
                return false;
            }
            true
        });

        // If all competitors were commemorations, pick the first one as winner
        if sorted_competetors.is_empty() {
            bail!("No competitors provided for conflict resolution after filtering");
        }
        // Two-pass resolution:
        // 1) Determine the final winner by comparing competitors in sequence.
        // 2) With the final winner fixed, compute commemorations, axed entries and any transfer.
        let mut commemorations = Vec::new();
        let mut winner = Some(sorted_competetors[0].1.clone());
        let mut winning_rank = &sorted_competetors[0].0;
        let mut transferred: Option<(FeastRankOf, T)> = None;

        // First pass: pick the winner (provisionally update winner when an outcome would
        // make the current competitor take precedence).
        for (current_rank, current_name) in sorted_competetors.iter() {
            // let (current_rank, current_name) = &sorted_competetors[i];
            if std::ptr::eq(current_rank, winning_rank) {
                continue;
            }
            debug_trace.push(format!(
                "Resolving between {:?} ({:?}) and {:?} ({:?})",
                winning_rank, winner, current_rank, current_name
            ));

            if winner.is_none() {
                winner = Some(current_name.clone());
                winning_rank = current_rank;
                continue;
            }

            let occurrence = winning_rank
                .resolve_occurrence(current_rank, false)
                .context(format!(
                    "Error resolving occurrence between {:?} and {:?}",
                    sorted_competetors[0].1, current_name
                ))?;
            debug_trace.push(format!("    -> Occurrence result: {:?}", occurrence));

            // Only outcomes that would switch the provisional winner are considered
            // in this pass; we don't record commemorations/transfers here.
            match occurrence {
                OccurrenceResult::SecondWins
                | OccurrenceResult::SecondWinsFirstCommemoration
                | OccurrenceResult::SecondWinsFirstTransferred => {
                    winner = Some(current_name.clone());
                    winning_rank = current_rank;
                }
                OccurrenceResult::CommemorateBoth => {
                    commemorations.push(current_name.clone());
                    commemorations.push(winner.clone().unwrap());
                    winner = None;
                }
                _ => {
                    // winner remains the same
                }
            }
        }
        let (winner, winning_rank) = if let Some(winner) = winner {
            (winner, winning_rank)
        } else {
            // find the feria in the list, if any
            competetors
                .iter()
                .find(|(r, _)| matches!(r, FeastRankOfInner::Feria { rank: _, .. }))
                .map(|(r, n)| (n.clone(), r))
                .context(
                    "Conflict resolution resulted in no winner; no feria present to break tie",
                )?
        };
        if let FeastRankOfInner::Feria { rank, flags } = winning_rank {
            if flags.contains(FeriaFlags::LENT) || *rank <= 2 {
                // Lent feria takes precedence over memorials
                commemorations.clear();
            }
        }
        // Second pass: determine commemorations, transfers relative to
        // the final winner. We skip the winner entry itself.
        for (rank, name) in sorted_competetors.iter() {
            // Skip the entry that corresponds to the final winner. Use pointer equality
            // to avoid requiring PartialEq on the generic contestant payload `T`.
            if std::ptr::eq(rank, winning_rank) {
                continue;
            }

            debug_trace.push(format!(
                "Comparing final winner {:?} ({:?}) to {:?} ({:?})",
                winning_rank, winner, rank, name
            ));

            let occurrence = winning_rank
                .resolve_occurrence(rank, false)
                .context(format!(
                    "Error resolving occurrence between {:?} and {:?}",
                    winner, name
                ))?;

            debug_trace.push(format!("    -> Occurrence result: {:?}", occurrence));
            match occurrence {
                OccurrenceResult::FirstWinsSecondCommemoration => {
                    commemorations.push(name.clone());
                }
                OccurrenceResult::FirstWinsSecondTransferred => {
                    if transferred.is_some() {
                        bail!("Multiple transfers detected in conflict resolution");
                    }
                    transferred = Some((FeastRankOf(rank.clone()), name.clone()));
                }
                OccurrenceResult::SecondWinsFirstCommemoration => {
                    commemorations.push(winner.clone());
                }
                OccurrenceResult::SecondWinsFirstTransferred => {
                    if transferred.is_some() {
                        bail!("Multiple transfers detected in conflict resolution");
                    }
                    transferred = Some((FeastRankOf(winning_rank.clone()), winner.clone()));
                }
                OccurrenceResult::CommemorateBoth => {
                    bail!("CommemorateBoth should not occur in second pass");
                }
                _ => {
                    // Nothing to do for other outcomes
                }
            }
        }

        let _winner_rank = winning_rank.get_numeric_rank();

        let commemorations = if matches!(winning_rank, FeastRankOfInner::Feria { rank: 2..=4, .. })
        {
            commemorations
                .into_iter()
                .chain(base_commemorations)
                .collect::<Vec<_>>()
        } else {
            // Sundays do not take commemorations
            Vec::new()
        };

        Ok(super::ResolveConflictsResult {
            winner,
            winner_rank: FeastRankOf(winning_rank.clone()),
            transferred,
            commemorations,
            debug_trace,
        })
    }

    /// Convert from legacy rank string and day type with context
    fn new_with_context(rank: &str, day_type: &DayType, context: &LiturgicalContext) -> Self {
        let numeric_rank = Self::parse_rank_string(rank);

        match day_type {
            DayType::Feria => {
                let mut flags = FeriaFlags::empty();

                // Set flags based on liturgical context
                if context.of_lent {
                    flags |= FeriaFlags::LENT;
                }

                FeastRankOfInner::Feria {
                    rank: numeric_rank,
                    flags,
                }
            }
            DayType::Feast => {
                let mut flags = FeastFlags::empty();
                if context.of_our_lord {
                    flags |= FeastFlags::OF_THE_LORD;
                }
                if context.is_movable {
                    flags |= FeastFlags::MOVABLE;
                }

                // Map numeric ranks to Ordinary Form feast types
                match numeric_rank {
                    1 => FeastRankOfInner::Feast { rank: 1, flags }, // Solemnity
                    2 => FeastRankOfInner::Feast { rank: 2, flags }, // Feast
                    3 => FeastRankOfInner::Feast { rank: 3, flags }, // Memorial
                    4 => FeastRankOfInner::Feast { rank: 4, flags }, // Optional Memorial
                    _ => panic!(
                        "Invalid numeric rank {} for Feast in Ordinary Form",
                        numeric_rank
                    ),
                }
            }
            DayType::Sunday => FeastRankOfInner::Sunday {
                rank: numeric_rank,
                flags: SundayFlags::empty(),
            }, // rank 1=major (Easter/Christmas), 2=seasonal, 3=ordinary time
            DayType::Vigil => {
                // Vigils in Ordinary Form are treated as optional memorials unless of major feast
                let flags = if context.of_our_lord {
                    FeastFlags::OF_THE_LORD
                } else {
                    FeastFlags::empty()
                };
                if numeric_rank <= 2 {
                    FeastRankOfInner::Feast { rank: 3, flags } // Memorial
                } else {
                    FeastRankOfInner::Feast { rank: 4, flags } // Optional Memorial
                }
            }
            DayType::Octave => {
                // Octaves in Ordinary Form are very limited - only Christmas and Easter octaves remain
                // The octave days themselves should be lower rank than the main feast
                // they are treated as ferias or sundays, depending on context

                match context.secondary_day_type {
                    Some(DayType::Feria) => FeastRankOfInner::Feria {
                        rank: numeric_rank,
                        flags: FeriaFlags::empty(),
                    }, // Ordinary Time Feria
                    Some(DayType::Sunday) => FeastRankOfInner::Sunday {
                        rank: 2,
                        flags: SundayFlags::WAS_OCTAVE,
                    }, // Ordinary Time Sunday
                    _ => panic!("Octave day must have secondary day type of Feria or Sunday"),
                }
            }
        }
    }

    /// Parse a rank string into a numeric rank for Ordinary Form
    fn parse_rank_string(rank: &str) -> u8 {
        let rank_upper = rank.to_uppercase();

        match rank_upper.as_str() {
            "SOLEMNITY" | "I" => 1,                            // Solemnity (highest)
            "FEAST" | "II" => 2,                               // Feast
            "MEMORIAL" | "III" => 3,                           // Memorial (obligatory)
            "OPTIONAL" | "IV" | "COMM." | "COMMEMORATIO" => 4, // Optional Memorial
            _ => 3, // Default to Memorial for unknown ranks
        }
    }

    /// Get the numeric rank for comparison (lower is higher precedence)
    fn get_numeric_rank(&self) -> u8 {
        match self {
            FeastRankOfInner::Feast { rank, .. } => *rank, // 1=Solemnity, 2=Feast, 3=Memorial, 4=Optional
            FeastRankOfInner::Sunday { rank, .. } => {
                // Sunday ranks: 1=major (Easter/Christmas), 2=seasonal, 3=ordinary time
                match rank {
                    1 => 1, // Major Sunday (like Easter) takes precedence over most feasts
                    2 => 2, // Seasonal Sunday
                    _ => 3, // Ordinary Time Sunday
                }
            }
            FeastRankOfInner::Feria { rank, .. } => {
                // Feria ranks: 1=Ash Wed/Good Friday, 2=Lent/Advent, 3=Ordinary Time
                *rank
            }
        }
    }

    fn get_rank_string_verbose(&self) -> ArcStr {
        self._get_rank_string(true)
    }

    fn get_rank_string(&self) -> ArcStr {
        self._get_rank_string(false)
    }

    /// Get the rank as a string for display
    fn _get_rank_string(&self, v: bool) -> ArcStr {
        match self {
            FeastRankOfInner::Feast { rank, flags } => {
                let base_str = match rank {
                    1 => "Solemnity".into(),
                    2 => "Feast".into(),
                    3 => "Memorial".into(),
                    4 => "Optional Memorial".into(),
                    _ => "Feast___".into(),
                };
                if !v {
                    return base_str;
                }

                let mut rank_str = base_str;
                if flags.contains(FeastFlags::OF_THE_LORD) {
                    rank_str = format!("{} of the Lord", rank_str).into();
                }
                if flags.contains(FeastFlags::MOVABLE) {
                    rank_str = format!("{} (Movable)", rank_str).into();
                }
                rank_str
            }
            FeastRankOfInner::Sunday { rank, flags } => {
                let base_str = match rank {
                    1 => "Major Sunday".into(),
                    2 => "Sunday".into(),
                    _ => "Sunday___".into(),
                };
                if !v {
                    return base_str;
                }

                let mut rank_str = base_str;
                if flags.contains(SundayFlags::WAS_OCTAVE) {
                    rank_str = format!("{} (Octave)", rank_str).into();
                }
                rank_str
            }
            FeastRankOfInner::Feria { rank, flags } => {
                if !v {
                    return "Feria".into();
                }
                let rank_str = match rank {
                    1 => "I",
                    2 => "II",
                    3 => "III",
                    _ => "III",
                };
                if flags.contains(FeriaFlags::LENT) {
                    format!("Lenten feria {}", rank_str).into()
                } else {
                    format!("Feria {}", rank_str).into()
                }
            }
        }
    }
    /// Check if this feast is of Our Lord
    fn is_of_our_lord(&self) -> bool {
        match self {
            FeastRankOfInner::Feast { flags, .. } => flags.contains(FeastFlags::OF_THE_LORD),
            _ => false,
        }
    }

    /// Resolve occurrence between two feast ranks
    fn resolve_occurrence(&self, other: &Self, try_swapped: bool) -> Result<OccurrenceResult> {
        let self_rank = self.get_numeric_rank();
        let other_rank = other.get_numeric_rank();

        // Basic rule: lower numeric rank wins (higher precedence)
        match self_rank.cmp(&other_rank) {
            std::cmp::Ordering::Less => return Ok(OccurrenceResult::FirstWins),
            std::cmp::Ordering::Greater => return Ok(OccurrenceResult::SecondWins),
            std::cmp::Ordering::Equal => {
                // Same rank - need special rules
            }
        }

        // Same rank - need special rules
        let result = match (self, other) {
            // Sundays vs Feasts: Special rules based on feast type and Sunday type
            (
                feast,
                FeastRankOfInner::Sunday {
                    rank: sunday_rank,
                    flags,
                },
            ) => {
                match (feast, sunday_rank) {
                    // Solemnities always beat Sundays except major Sundays
                    (FeastRankOfInner::Feast { rank: 1, .. }, 1) => {
                        // Solemnity vs Major Sunday - Major Sunday wins
                        OccurrenceResult::SecondWinsFirstTransferred
                    }
                    (FeastRankOfInner::Feast { rank: 2, .. }, 2) => {
                        if feast.is_of_our_lord() {
                            OccurrenceResult::FirstWins // Of the Lord beats minor Sunday
                        } else {
                            OccurrenceResult::SecondWins // Major Sunday wins
                        }
                    }
                    (FeastRankOfInner::Feast { rank: 1, .. }, _) => {
                        OccurrenceResult::FirstWins // Solemnity wins
                    }
                    (FeastRankOfInner::Sunday { flags: flags1, .. }, _)
                        if flags.contains(SundayFlags::WAS_OCTAVE)
                            && !flags1.contains(SundayFlags::WAS_OCTAVE) =>
                    {
                        OccurrenceResult::SecondWins
                    }
                    // Other feasts generally give way to major Sundays
                    // (_, 1 | 2) => OccurrenceResult::SecondWins,
                    // Otherwise feast wins
                    _ => return self.handle_swap_or_error(other, try_swapped),
                }
            }
            // Feast vs Feast of same rank: Handle conflicts by liturgical precedence
            (
                FeastRankOfInner::Feast { rank: 1, flags: f1 },
                FeastRankOfInner::Feast { rank: 1, flags: f2 },
            ) => {
                // Rank I (Solemnity) conflicts: "Of the Lord" takes precedence
                if f1.contains(FeastFlags::OF_THE_LORD) && !f2.contains(FeastFlags::OF_THE_LORD) {
                    OccurrenceResult::FirstWinsSecondTransferred
                } else if !f1.contains(FeastFlags::OF_THE_LORD)
                    && f2.contains(FeastFlags::OF_THE_LORD)
                    || f1.contains(FeastFlags::MOVABLE) && !f2.contains(FeastFlags::MOVABLE)
                {
                    OccurrenceResult::SecondWinsFirstTransferred
                } else if !f1.contains(FeastFlags::MOVABLE) && f2.contains(FeastFlags::MOVABLE) {
                    OccurrenceResult::FirstWinsSecondTransferred
                } else {
                    // No clear precedence rule - this should be rare
                    return self.handle_swap_or_error(other, try_swapped);
                }
            }
            (
                FeastRankOfInner::Feast { rank: 2, flags: f1 },
                FeastRankOfInner::Feast { rank: 2, flags: f2 },
            ) => {
                // Rank I (Solemnity) conflicts: "Of the Lord" takes precedence
                if f1.contains(FeastFlags::OF_THE_LORD) && !f2.contains(FeastFlags::OF_THE_LORD) {
                    OccurrenceResult::FirstWins
                } else if !f1.contains(FeastFlags::OF_THE_LORD)
                    && f2.contains(FeastFlags::OF_THE_LORD)
                {
                    OccurrenceResult::SecondWins
                } else if f1.contains(FeastFlags::MOVABLE) && !f2.contains(FeastFlags::MOVABLE) {
                    OccurrenceResult::FirstWins
                } else if !f1.contains(FeastFlags::MOVABLE) && f2.contains(FeastFlags::MOVABLE) {
                    OccurrenceResult::SecondWins
                } else {
                    // No clear precedence rule - this should be rare
                    return self.handle_swap_or_error(other, try_swapped);
                }
            }
            (
                FeastRankOfInner::Feast {
                    rank: 3,
                    flags: _f1,
                },
                FeastRankOfInner::Feast {
                    rank: 3,
                    flags: _f2,
                },
            ) => OccurrenceResult::CommemorateBoth,
            (
                FeastRankOfInner::Feria { flags, .. },
                FeastRankOfInner::Feast {
                    rank: feast_rank, ..
                },
            ) => {
                if flags.contains(FeriaFlags::LENT) && *feast_rank >= 3 {
                    OccurrenceResult::FirstWinsSecondCommemoration
                } else {
                    OccurrenceResult::SecondWins
                }
            }
            _ => return self.handle_swap_or_error(other, try_swapped),
        };

        Ok(result)
    }

    fn handle_swap_or_error(&self, other: &Self, try_swapped: bool) -> Result<OccurrenceResult> {
        if try_swapped {
            bail!(
                "Could not resolve occurrence between {:?} and {:?}",
                self,
                other
            );
        }

        // Try swapping the order
        Ok(other.resolve_occurrence(self, true)?.swap())
    }
}

#[cfg(test)]
mod test {
    use test_case::test_matrix;

    use super::*;
    use crate::calender::feast_rank::{
        test::{
            test_feast_rank_enumeration_conflicts, test_feast_rank_enumeration_occurance_graph,
        },
        OctaveFlags,
    };

    #[test]
    fn test_basic_feast_ranking() {
        let context = LiturgicalContext {
            season_name: Some("Ordinary Time".to_string()),
            feast_name: None,
            is_movable: false,
            of_our_lord: false,
            is_easter_or_pentecost: false,
            of_lent: false,
            secondary_day_type: None,
            octave_flags: OctaveFlags::empty(),
        };

        let solemnity = FeastRankOfInner::new_with_context("I", &DayType::Feast, &context);
        let feast = FeastRankOfInner::new_with_context("II", &DayType::Feast, &context);
        let memorial = FeastRankOfInner::new_with_context("III", &DayType::Feast, &context);
        let optional = FeastRankOfInner::new_with_context("IV", &DayType::Feast, &context);

        assert!(matches!(solemnity, FeastRankOfInner::Feast { rank: 1, .. }));
        assert!(matches!(feast, FeastRankOfInner::Feast { rank: 2, .. }));
        assert!(matches!(memorial, FeastRankOfInner::Feast { rank: 3, .. }));
        assert!(matches!(optional, FeastRankOfInner::Feast { rank: 4, .. }));

        assert_eq!(solemnity.get_numeric_rank(), 1);
        assert_eq!(feast.get_numeric_rank(), 2);
        assert_eq!(memorial.get_numeric_rank(), 3);
        assert_eq!(optional.get_numeric_rank(), 4);
    }

    #[test]
    fn test_weekday_ranking() {
        let lent_context = LiturgicalContext {
            season_name: Some("Lent".to_string()),
            feast_name: None,
            is_movable: false,
            of_our_lord: false,
            is_easter_or_pentecost: false,
            of_lent: true,
            secondary_day_type: None,
            octave_flags: OctaveFlags::empty(),
        };

        let advent_context = LiturgicalContext {
            season_name: Some("Advent".to_string()),
            feast_name: None,
            is_movable: false,
            of_our_lord: false,
            is_easter_or_pentecost: false,
            of_lent: false,
            secondary_day_type: None,
            octave_flags: OctaveFlags::empty(),
        };

        let ordinary_context = LiturgicalContext {
            season_name: Some("Ordinary Time".to_string()),
            feast_name: None,
            is_movable: false,
            of_our_lord: false,
            is_easter_or_pentecost: false,
            of_lent: false,
            secondary_day_type: None,
            octave_flags: OctaveFlags::empty(),
        };

        let lent_weekday =
            FeastRankOfInner::new_with_context("III", &DayType::Feria, &lent_context);
        let advent_weekday =
            FeastRankOfInner::new_with_context("III", &DayType::Feria, &advent_context);
        let ordinary_weekday =
            FeastRankOfInner::new_with_context("IV", &DayType::Feria, &ordinary_context);

        assert!(matches!(
            lent_weekday,
            FeastRankOfInner::Feria { rank: 3, .. }
        ));
        assert!(matches!(
            advent_weekday,
            FeastRankOfInner::Feria { rank: 3, .. }
        ));
        assert!(matches!(
            ordinary_weekday,
            FeastRankOfInner::Feria { rank: 4, .. }
        ));

        assert_eq!(lent_weekday.get_numeric_rank(), 3); // rank 2 -> numeric 4
        assert_eq!(advent_weekday.get_numeric_rank(), 3); // rank 2 -> numeric 4
        assert_eq!(ordinary_weekday.get_numeric_rank(), 4); // rank 3 -> numeric 5
    }

    #[test]
    fn test_conflict_resolution() {
        let context = LiturgicalContext {
            season_name: Some("Ordinary Time".to_string()),
            feast_name: None,
            is_movable: false,
            of_our_lord: false,
            is_easter_or_pentecost: false,
            of_lent: false,
            secondary_day_type: None,
            octave_flags: OctaveFlags::empty(),
        };

        let solemnity = FeastRankOfInner::new_with_context("I", &DayType::Feast, &context);
        let memorial = FeastRankOfInner::new_with_context("III", &DayType::Feast, &context);

        let result = solemnity.resolve_occurrence(&memorial, false).unwrap();
        assert_eq!(result, OccurrenceResult::FirstWins);

        let result2 = memorial.resolve_occurrence(&solemnity, false).unwrap();
        assert_eq!(result2, OccurrenceResult::SecondWins);
    }

    #[test]
    fn test_feast_of_our_lord_against_sunday() {
        let context = LiturgicalContext {
            season_name: Some("Ordinary Time".to_string()),
            feast_name: None,
            is_movable: false,
            of_our_lord: true,
            is_easter_or_pentecost: false,
            of_lent: false,
            secondary_day_type: None,
            octave_flags: OctaveFlags::empty(),
        };

        let feast = FeastRankOfInner::new_with_context("II", &DayType::Feast, &context);
        let sunday = FeastRankOfInner::new_with_context("II", &DayType::Sunday, &context);

        let result = feast.resolve_occurrence(&sunday, false).unwrap();
        assert_eq!(result, OccurrenceResult::FirstWins);

        let result2 = sunday.resolve_occurrence(&feast, false).unwrap();
        assert_eq!(result2, OccurrenceResult::SecondWins);
    }

    #[test]
    fn test_lenten_feria_vs_memorial() {
        let lent_context = LiturgicalContext {
            season_name: Some("Lent".to_string()),
            feast_name: None,
            is_movable: false,
            of_our_lord: false,
            is_easter_or_pentecost: false,
            of_lent: true,
            secondary_day_type: None,
            octave_flags: OctaveFlags::empty(),
        };

        let ordinary_context = LiturgicalContext {
            season_name: Some("Ordinary Time".to_string()),
            feast_name: None,
            is_movable: false,
            of_our_lord: false,
            is_easter_or_pentecost: false,
            of_lent: false,
            secondary_day_type: None,
            octave_flags: OctaveFlags::empty(),
        };

        let lenten_feria =
            FeastRankOfInner::new_with_context("III", &DayType::Feria, &lent_context);
        let memorial =
            FeastRankOfInner::new_with_context("III", &DayType::Feast, &ordinary_context);

        // Lenten feria should win over memorial and memorial should be commemorated
        let result = lenten_feria.resolve_occurrence(&memorial, false).unwrap();
        assert_eq!(result, OccurrenceResult::FirstWinsSecondCommemoration);

        let result2 = memorial.resolve_occurrence(&lenten_feria, false).unwrap();
        assert_eq!(result2, OccurrenceResult::SecondWinsFirstCommemoration);
    }

    #[test]
    fn test_lenten_feria_vs_solemnity() {
        let lent_context = LiturgicalContext {
            season_name: Some("Lent".to_string()),
            feast_name: None,
            is_movable: false,
            of_our_lord: false,
            is_easter_or_pentecost: false,
            of_lent: true,
            secondary_day_type: None,
            octave_flags: OctaveFlags::empty(),
        };

        let ordinary_context = LiturgicalContext {
            season_name: Some("Ordinary Time".to_string()),
            feast_name: None,
            is_movable: false,
            of_our_lord: false,
            is_easter_or_pentecost: false,
            of_lent: false,
            secondary_day_type: None,
            octave_flags: OctaveFlags::empty(),
        };

        let lenten_feria =
            FeastRankOfInner::new_with_context("III", &DayType::Feria, &lent_context);
        let solemnity = FeastRankOfInner::new_with_context("I", &DayType::Feast, &ordinary_context);

        // Solemnity should still win over Lenten feria
        let result = lenten_feria.resolve_occurrence(&solemnity, false).unwrap();
        assert_eq!(result, OccurrenceResult::SecondWins);

        let result2 = solemnity.resolve_occurrence(&lenten_feria, false).unwrap();
        assert_eq!(result2, OccurrenceResult::FirstWins);
    }

    // test memorial against first class octave feria
    #[test]
    fn test_memorial_vs_octave_feria() {
        let octave_context = LiturgicalContext {
            season_name: Some("Christmas".to_string()),
            feast_name: None,
            is_movable: false,
            of_our_lord: false,
            is_easter_or_pentecost: false,
            of_lent: false,
            secondary_day_type: Some(DayType::Feria),
            octave_flags: OctaveFlags::empty(),
        };
        let ordinary_context = LiturgicalContext {
            season_name: Some("Ordinary Time".to_string()),
            feast_name: None,
            is_movable: false,
            of_our_lord: false,
            is_easter_or_pentecost: false,
            of_lent: false,
            secondary_day_type: None,
            octave_flags: OctaveFlags::empty(),
        };
        let octave_feria =
            FeastRankOfInner::new_with_context("I", &DayType::Octave, &octave_context);
        println!("Octave feria: {:?}", octave_feria);
        let memorial =
            FeastRankOfInner::new_with_context("III", &DayType::Feast, &ordinary_context);
        // Memorial should win over octave feria
        let result = memorial.resolve_occurrence(&octave_feria, false).unwrap();
        assert_eq!(result, OccurrenceResult::SecondWins);
        let result2 = octave_feria.resolve_occurrence(&memorial, false).unwrap();
        assert_eq!(result2, OccurrenceResult::FirstWins);
    }

    #[test]
    fn test_solemnity_conflict_of_the_lord_wins() {
        // Sacred Heart vs John the Baptist: "Of the Lord" should win
        let sacred_heart = FeastRankOfInner::Feast {
            rank: 1,
            flags: FeastFlags::OF_THE_LORD | FeastFlags::MOVABLE,
        };

        let john_baptist = FeastRankOfInner::Feast {
            rank: 1,
            flags: FeastFlags::empty(),
        };

        let result = sacred_heart
            .resolve_occurrence(&john_baptist, false)
            .unwrap();
        assert_eq!(result, OccurrenceResult::FirstWinsSecondTransferred);

        // Test the reverse order
        let result = john_baptist
            .resolve_occurrence(&sacred_heart, false)
            .unwrap();
        assert_eq!(result, OccurrenceResult::SecondWinsFirstTransferred);
    }

    #[test]
    fn test_solemnity_conflict_movable_transfers() {
        // When both are or neither are "of the Lord", movable feasts should transfer
        let movable_solemnity = FeastRankOfInner::Feast {
            rank: 1,
            flags: FeastFlags::MOVABLE,
        };

        let fixed_solemnity = FeastRankOfInner::Feast {
            rank: 1,
            flags: FeastFlags::empty(),
        };

        let result = movable_solemnity
            .resolve_occurrence(&fixed_solemnity, false)
            .unwrap();
        assert_eq!(result, OccurrenceResult::SecondWinsFirstTransferred);

        // Test the reverse order
        let result = fixed_solemnity
            .resolve_occurrence(&movable_solemnity, false)
            .unwrap();
        assert_eq!(result, OccurrenceResult::FirstWinsSecondTransferred);
    }

    #[test]
    fn test_solemnity_conflict_both_of_the_lord() {
        // When both are "of the Lord", movable should still transfer
        let movable_of_lord = FeastRankOfInner::Feast {
            rank: 1,
            flags: FeastFlags::OF_THE_LORD | FeastFlags::MOVABLE,
        };

        let fixed_of_lord = FeastRankOfInner::Feast {
            rank: 1,
            flags: FeastFlags::OF_THE_LORD,
        };

        let result = movable_of_lord
            .resolve_occurrence(&fixed_of_lord, false)
            .unwrap();
        assert_eq!(result, OccurrenceResult::SecondWinsFirstTransferred);
    }

    impl FeastRankOf {
        fn enumerate() -> Vec<Self> {
            let mut ranks = Vec::new();

            for rank in 1..=4 {
                for of_the_lord in [false, true] {
                    for movable in [false, true] {
                        let mut flags = FeastFlags::empty();
                        if of_the_lord {
                            flags |= FeastFlags::OF_THE_LORD;
                        }
                        if movable {
                            flags |= FeastFlags::MOVABLE;
                        }
                        ranks.push(FeastRankOf(FeastRankOfInner::Feast { rank, flags }));
                    }
                }
            }

            for rank in 1..=3 {
                for was_octave in [false, true] {
                    let mut flags = SundayFlags::empty();
                    if was_octave {
                        flags |= SundayFlags::WAS_OCTAVE;
                    }
                    ranks.push(FeastRankOf(FeastRankOfInner::Sunday { rank, flags }));
                }
            }

            for rank in 1..=3 {
                for lent in [false, true] {
                    let mut flags = FeriaFlags::empty();
                    if lent {
                        flags |= FeriaFlags::LENT;
                    }
                    ranks.push(FeastRankOf(FeastRankOfInner::Feria { rank, flags }));
                }
            }

            ranks
        }
    }

    #[test]
    fn test_feast_rank_of_enumeration_occurance_graph() {
        test_feast_rank_enumeration_occurance_graph(FeastRankOf::enumerate());
    }

    #[test_matrix(2..=4)]
    fn test_feast_rank_of_enumeration_conflicts(n: usize) {
        test_feast_rank_enumeration_conflicts(FeastRankOf::enumerate(), n);
    }
}
