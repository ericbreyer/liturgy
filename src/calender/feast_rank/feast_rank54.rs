use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use super::{DayType, FeastRank, LiturgicalContext, ResolveConflictsResult};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OctaveType {
    Privileged1,
    Privileged2,
    Privileged3,
    Common,
    Simple,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum FeastRank54Inner {
    Feria {
        rank: u8,
        flags: FeriaFlags,
    },
    Feast {
        rank: FeastClass,
        flags: FeastFlags,
    },
    Vigil {
        rank: u8,
    },
    Sunday {
        rank: u8,
    },
    Octave {
        rank: u8,
        is_octave_day: bool,
        octave_type: OctaveType,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
enum FeastClass {
    /// First Class Double - highest rank in 1954 (Christmas, Easter, Pentecost, etc.)
    FirstClassDouble = 1,
    /// Second Class Double - very high rank feasts  
    SecondClassDouble = 2,
    /// Major Double - important feasts
    MajorDouble = 3,
    /// Double - regular major feasts
    Double = 4,
    /// Semidouble - lesser feasts
    Semidouble = 5,
    /// Simple - commemorations and simple observances
    Simple = 6,
    /// Commemoration - lowest rank, made alongside other celebrations
    Commemoration = 7,
}

enum OccurrenceResult {
    FirstNothingOfSecond,
    SecondNothingOfFirst,
    FirstCommemorationOfSecond,
    FirstCommemorationOfSecondAtLauds,
    SecondCommemorationOfFirst,
    SecondCommemorationOfFirstAtLauds,
    FirstTransferOfSecond,
    SecondTransferOfFirst,
}

impl OccurrenceResult {
    fn reverse(self) -> Self {
        match self {
            OccurrenceResult::FirstNothingOfSecond => OccurrenceResult::SecondNothingOfFirst,
            OccurrenceResult::SecondNothingOfFirst => OccurrenceResult::FirstNothingOfSecond,
            OccurrenceResult::FirstCommemorationOfSecond => {
                OccurrenceResult::SecondCommemorationOfFirst
            }
            OccurrenceResult::FirstCommemorationOfSecondAtLauds => {
                OccurrenceResult::SecondCommemorationOfFirstAtLauds
            }
            OccurrenceResult::SecondCommemorationOfFirst => {
                OccurrenceResult::FirstCommemorationOfSecond
            }
            OccurrenceResult::SecondCommemorationOfFirstAtLauds => {
                OccurrenceResult::FirstCommemorationOfSecondAtLauds
            }
            OccurrenceResult::FirstTransferOfSecond => OccurrenceResult::SecondTransferOfFirst,
            OccurrenceResult::SecondTransferOfFirst => OccurrenceResult::FirstTransferOfSecond,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeastRank54(FeastRank54Inner);

impl FeastRank for FeastRank54 {
    fn resolve_conflicts<T>(competetors: &[(Self, T)]) -> ResolveConflictsResult<Self, T>
    where
        Self: Sized,
        T: Clone + Debug,
    {
        FeastRank54Inner::resolve_conflicts(
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
        FeastRank54(FeastRank54Inner::new_with_context(rank, day_type, context))
    }

    fn is_ferial_or_sunday_rank(&self) -> bool {
        matches!(
            self.0,
            FeastRank54Inner::Feria { .. } | FeastRank54Inner::Sunday { .. }
        )
    }

    fn is_high_festial(&self) -> bool {
        matches!(
            self.0,
            FeastRank54Inner::Feast {
                rank: FeastClass::FirstClassDouble,
                ..
            } | FeastRank54Inner::Feast {
                rank: FeastClass::SecondClassDouble,
                ..
            } | FeastRank54Inner::Feast {
                rank: FeastClass::Double,
                ..
            }
        )
    }

    fn get_rank_string(&self) -> String {
        self.0.get_rank_string()
    }

    fn get_bvm_on_saturday_rank() -> Option<Self>
    where
        Self: Sized,
    {
        Some(FeastRank54(FeastRank54Inner::Feria {
            rank: 3,
            flags: FeriaFlags::empty(),
        }))
    }

    fn admits_bvm_on_saturday(&self) -> super::BVMOnSaturdayResult
    {
        // admit BVM on Saturday if feria rank is 3
        if let FeastRank54Inner::Feria { rank: 4, .. } = self.0 {
            super::BVMOnSaturdayResult::Admitted
        }
        // commemorate if simplex feast
    else if let FeastRank54Inner::Feast { rank, .. } = &self.0 {
            if rank == &FeastClass::Simple {
                super::BVMOnSaturdayResult::Commemorated
            } else {
                super::BVMOnSaturdayResult::NotAdmitted
            }
        } else {
            super::BVMOnSaturdayResult::NotAdmitted
        }
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
     struct FeriaFlags: u8 {
        const OF_LENT = 0b00000001;
        const EMBER_DAY = 0b00000010;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
     struct FeastFlags: u8 {
        const OF_OUR_LORD = 0b00000001;
        const IMMACULATE_CONCEPTION = 0b00000010;
        const MOVABLE = 0b00000100;
        const ALL_SOULS = 0b00001000;
    }
}

impl FeastRank54Inner {
    fn get_numeric_rank(&self) -> u8 {
        match self {
            FeastRank54Inner::Feria { rank, .. } => *rank, // Ferial ranks start from 21
            FeastRank54Inner::Feast { rank, .. } => match rank {
                FeastClass::FirstClassDouble => 1,
                FeastClass::SecondClassDouble => 2,
                FeastClass::MajorDouble => 3,
                FeastClass::Double => 4,
                FeastClass::Semidouble => 5,
                FeastClass::Simple => 6,
                FeastClass::Commemoration => 7,
            },
            FeastRank54Inner::Vigil { rank } => *rank, // Vigil ranks start from 16
            FeastRank54Inner::Sunday { rank } => *rank, // Sunday ranks start from 11
            FeastRank54Inner::Octave { rank, .. } => *rank, // Octave ranks start from 6
        }
    }

    fn resolve_conflicts<T: Clone + Debug>(
        competetors: &[(Self, T)],
    ) -> ResolveConflictsResult<FeastRank54, T> {
        if competetors.is_empty() {
            panic!("No competetors provided for conflict resolution");
        }

        let mut sorted_competetors = competetors.to_vec();
        sorted_competetors.sort_by(|(rank_a, _), (rank_b, _)| {
            rank_a.get_numeric_rank().cmp(&rank_b.get_numeric_rank())
        });

        // any 4th class feast automatically is a commemoration
        let mut base_commemorations = Vec::new();
        let mut indices_to_remove = Vec::new();
        for (i, (rank, name)) in sorted_competetors.iter().enumerate() {
            if let FeastRank54Inner::Feast {
                rank: FeastClass::Commemoration,
                ..
            } = *rank
            {
                base_commemorations.push(name.clone());
                indices_to_remove.push(i);
            }
        }
        // Remove in reverse order to avoid index shifting
        for i in indices_to_remove.into_iter().rev() {
            sorted_competetors.remove(i);
        }

        // If all competitors were commemorations, pick the first one as winner
        if sorted_competetors.is_empty() {
            panic!("No competetors provided for conflict resolution");
        }
        let mut commemorations = Vec::new();
        let mut winner = sorted_competetors[0].1.clone();
        let mut winning_rank = &sorted_competetors[0].0;
        let mut transferred = None;
        for i in 1..sorted_competetors.len() {
            let (current_rank, current_name) = &sorted_competetors[i];
            match sorted_competetors[0]
                .0
                .resolve_occurrence(current_rank, true)
            {
                Ok(occurrence) => {
                    match occurrence {
                        OccurrenceResult::FirstNothingOfSecond => {
                            // Winner remains the same, nothing changes
                        }
                        OccurrenceResult::SecondNothingOfFirst => {
                            // Current becomes the new winner
                            winner = current_name.clone();
                            winning_rank = current_rank;
                        }
                        OccurrenceResult::FirstCommemorationOfSecond
                        | OccurrenceResult::FirstCommemorationOfSecondAtLauds => {
                            commemorations.push(current_name.clone());
                        }
                        OccurrenceResult::SecondCommemorationOfFirst
                        | OccurrenceResult::SecondCommemorationOfFirstAtLauds => {
                            commemorations.push(winner.clone());
                            winner = current_name.clone();
                            winning_rank = current_rank;
                        }
                        OccurrenceResult::FirstTransferOfSecond => {
                            transferred =
                                Some((FeastRank54(current_rank.clone()), current_name.clone()));
                        }
                        OccurrenceResult::SecondTransferOfFirst => {
                            transferred = Some((FeastRank54(winning_rank.clone()), winner.clone()));
                            winner = current_name.clone();
                            winning_rank = current_rank;
                        }
                    }
                }
                Err(e) => {
                    panic!(
                        "Error resolving occurrence between {:?} and {:?}: {}",
                        sorted_competetors[0].1, current_name, e
                    );
                }
            }
        }

        let winner_rank = winning_rank.get_numeric_rank();

        // add base commemorations to commemorations if winner is not a sunday or a 1st or 2nd class movable feast
        if let FeastRank54Inner::Feast { rank, flags } = winning_rank {
            if !((*rank as u8) < 3 && flags.contains(FeastFlags::MOVABLE)) {
                commemorations.extend(base_commemorations);
            }
        } else if let FeastRank54Inner::Sunday { .. } = winning_rank {
            // do nothing, sundays do not get commemorations
        } else if let FeastRank54Inner::Feria { rank: 1, .. } = winning_rank {
            // do nothing, 1st class ferias do not get commemorations
        } else if let FeastRank54Inner::Octave { rank: 1, .. } = winning_rank {
            // do nothing, 1st class octaves do not get commemorations
        } else {
            commemorations.extend(base_commemorations);
        }

        super::ResolveConflictsResult {
            winner,
            winner_rank: FeastRank54(winning_rank.clone()),
            transferred,
            commemorations,
        }
    }

    fn resolve_occurrence(&self, other: &Self, try_swapped: bool) -> Result<OccurrenceResult> {
        if let FeastRank54Inner::Feria {
            rank: rank1,
            flags: flags1,
        } = self
        {
            // both ferias
            if let FeastRank54Inner::Feria {
                rank: rank2,
                flags: flags2,
            } = other
            {
                if rank1 == rank2 {
                    let is_ember_day1 = flags1.contains(FeriaFlags::EMBER_DAY);
                    let is_ember_day2 = flags2.contains(FeriaFlags::EMBER_DAY);
                    if is_ember_day1 && !is_ember_day2 {
                        return Ok(OccurrenceResult::FirstNothingOfSecond);
                    } else if !is_ember_day1 && is_ember_day2 {
                        return Ok(OccurrenceResult::SecondNothingOfFirst);
                    } else {
                        bail!("Two ferias of the same rank cannot occur on the same day");
                    }
                }
                match rank1.cmp(rank2) {
                    std::cmp::Ordering::Less => return Ok(OccurrenceResult::FirstNothingOfSecond),
                    std::cmp::Ordering::Greater => {
                        return Ok(OccurrenceResult::SecondNothingOfFirst)
                    }
                    _ => {}
                }
            }
        }
        // self is feast
        if let FeastRank54Inner::Feast {
            rank: rank1,
            flags: flags1,
        } = self
        {
            // other is octave
            if let FeastRank54Inner::Octave {
                rank: rank2,
                is_octave_day,
                octave_type,
            } = other
            {
                match octave_type {
                    OctaveType::Privileged1 => {
                        if *is_octave_day {
                            return Ok(OccurrenceResult::SecondTransferOfFirst);
                        } else {
                            return Ok(OccurrenceResult::FirstNothingOfSecond);
                        }
                    }
                    OctaveType::Privileged2 => {
                        if *rank1 == FeastClass::FirstClassDouble {
                            if *is_octave_day {
                                return Ok(OccurrenceResult::SecondTransferOfFirst);
                            } else {
                                return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                            }
                        } else {
                            return Ok(OccurrenceResult::FirstNothingOfSecond);
                        }
                    }
                    OctaveType::Privileged3 | OctaveType::Common => {
                        if (*rank1 as u8) < 6 {
                            return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                        } else {
                            return Ok(OccurrenceResult::FirstNothingOfSecond);
                        }
                    }
                    OctaveType::Simple => {
                        if *is_octave_day {
                            return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                        } else {
                            return Ok(OccurrenceResult::FirstNothingOfSecond);
                        }
                    }
                }
            }
            // other is feast
            if let FeastRank54Inner::Feast {
                rank: rank2,
                flags: _flags2,
            } = other
            {
                match (rank1, rank2) {
                    (FeastClass::Simple, FeastClass::FirstClassDouble) => {
                        return Ok(OccurrenceResult::SecondNothingOfFirst);
                    }
                    (FeastClass::Simple, _) => {
                        return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                    }
                    _ => {}
                }
            }
            // other is vigil
            if let FeastRank54Inner::Vigil { rank: rank2 } = other {
                match (rank1, rank2) {
                    (FeastClass::FirstClassDouble, _) => {
                        return Ok(OccurrenceResult::FirstNothingOfSecond);
                    }
                    (FeastClass::SecondClassDouble, _) => {
                        return Ok(OccurrenceResult::FirstNothingOfSecond);
                    }
                    (FeastClass::MajorDouble, _) => {
                        return Ok(OccurrenceResult::FirstNothingOfSecond);
                    }
                    (FeastClass::Double, _) => {
                        return Ok(OccurrenceResult::FirstNothingOfSecond);
                    }
                    (FeastClass::Semidouble, _) => {
                        return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                    }
                    (FeastClass::Simple, _) => {
                        return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                    }
                    (FeastClass::Commemoration, _) => {
                        return Ok(OccurrenceResult::SecondNothingOfFirst);
                    }
                    _ => {}
                }
            }
            // other is sunday — follow 1954 rules:
            // - Sunday I: no feast may be celebrated; feasts are commemorated (except Easter/Pentecost which cannot be commemorated — not detectable here)
            // - Sunday II: only Doubles of the I Class may be celebrated; other feasts are commemorated
            // - Lesser Sundays: Doubles of I or II class, or a feast of Our Lord, may be celebrated; others are commemorated
            if let FeastRank54Inner::Sunday { rank: rank2 } = other {
                // Sunday I (greatest Sundays)
                if *rank2 == 1 {
                    return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                }

                // Sunday II (greater Sundays of II class)
                if *rank2 == 2 {
                    if *rank1 == FeastClass::FirstClassDouble {
                        return Ok(OccurrenceResult::SecondTransferOfFirst);
                    } else {
                        return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                    }
                }

                // Lesser Sundays: allow First or Second Class Doubles or feasts 'of our Lord' to take precedence
                if *rank1 == FeastClass::FirstClassDouble
                    || *rank1 == FeastClass::SecondClassDouble
                    || flags1.contains(FeastFlags::OF_OUR_LORD)
                {
                    return Ok(OccurrenceResult::SecondTransferOfFirst);
                } else {
                    return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                }
            }
            }

        // self is vigil
        if let FeastRank54Inner::Vigil { rank: rank1 } = self {
            if let FeastRank54Inner::Octave {
                rank: rank2,
                is_octave_day: _,
                octave_type: _,
            } = other
            {
                match (rank1, rank2) {
                    _ => {}
                }
            }
            if let FeastRank54Inner::Feast {
                rank: rank2,
                flags: _flags2,
            } = other
            {
                match (rank1, rank2) {
                    (1, FeastClass::FirstClassDouble) => {
                        return Ok(OccurrenceResult::SecondNothingOfFirst);
                    }
                    (1, _) => return Ok(OccurrenceResult::FirstNothingOfSecond),
                    (2, FeastClass::FirstClassDouble) => {
                        return Ok(OccurrenceResult::SecondNothingOfFirst)
                    }
                    (2, _) => return Ok(OccurrenceResult::FirstNothingOfSecond),
                    (3, FeastClass::FirstClassDouble) => {
                        return Ok(OccurrenceResult::SecondNothingOfFirst)
                    }
                    (3, _) => return Ok(OccurrenceResult::FirstNothingOfSecond),
                    (4, FeastClass::FirstClassDouble) => {
                        return Ok(OccurrenceResult::SecondNothingOfFirst)
                    }
                    (4, _) => return Ok(OccurrenceResult::FirstNothingOfSecond),
                    (5, _) => return Ok(OccurrenceResult::SecondCommemorationOfFirst),
                    (6, _) => return Ok(OccurrenceResult::SecondCommemorationOfFirst),
                    _ => {}
                }
            }
            if let FeastRank54Inner::Feria {
                rank: rank2,
                flags: _flags2,
            } = other
            {
                match (rank1, rank2) {
                    (_, 3) => return Ok(OccurrenceResult::FirstNothingOfSecond),
                    _ => {}
                }
            }
            if let FeastRank54Inner::Sunday { rank: rank2 } = other {
                match (rank1, rank2) {
                    _ => {}
                }
            }
        }

        // self is octave
        if let FeastRank54Inner::Octave {
            rank: rank1,
            is_octave_day: _,
            octave_type: _,
        } = self
        {
            if let FeastRank54Inner::Feria {
                rank: rank2,
                flags: _flags2,
            } = other
            {
                match (rank1, rank2) {
                    (1, 1) => return Ok(OccurrenceResult::FirstNothingOfSecond),
                    (1, _) => return Ok(OccurrenceResult::FirstNothingOfSecond),
                    (2, 1) => return Ok(OccurrenceResult::SecondNothingOfFirst),
                    (2, _) => return Ok(OccurrenceResult::FirstNothingOfSecond),
                    (3, _) => return Ok(OccurrenceResult::SecondCommemorationOfFirst),
                    _ => {
                        let r1 = self.get_numeric_rank();
                        let r2 = other.get_numeric_rank();
                        if r1 < r2 {
                            return Ok(OccurrenceResult::FirstNothingOfSecond);
                        } else if r1 > r2 {
                            return Ok(OccurrenceResult::SecondNothingOfFirst);
                        } else {
                            bail!("Two days of the same rank cannot occur on the same day");
                        }
                    }
                }
            }
            if let FeastRank54Inner::Sunday { rank: rank2 } = other {
                match (rank1, rank2) {
                    _ => {
                        // fall through to final numeric tie-breaker
                        {}
                    }
                }
            }
            if let FeastRank54Inner::Octave {
                rank: rank2,
                is_octave_day: _,
                octave_type: _,
            } = other
            {
                match (rank1, rank2) {
                    _ => {
                        // fall through to final numeric tie-breaker
                        {}
                    }
                }
            }
        }

        // try swapping the order
        if try_swapped {
            return other.resolve_occurrence(self, false).map(|r| r.reverse());
        }
        // No explicit rule matched; fall through to numeric-rank fallback below.
        // just pick higher rank or apply tie-breaker if equal
        let rank1 = self.get_numeric_rank();
        let rank2 = other.get_numeric_rank();
        match rank1.cmp(&rank2) {
            std::cmp::Ordering::Less => Ok(OccurrenceResult::FirstNothingOfSecond),
            std::cmp::Ordering::Greater => Ok(OccurrenceResult::SecondNothingOfFirst),
            std::cmp::Ordering::Equal => {
                // tie-breaker by variant precedence and subrank
                // precedence groups (lower is higher priority): Feast(0), Octave(1), Sunday(2), Vigil(3), Feria(4)
                let (ptype1, sub1) = match self {
                    FeastRank54Inner::Feast { rank, .. } => (0u8, *rank as u8),
                    FeastRank54Inner::Octave { rank, .. } => (1u8, *rank),
                    FeastRank54Inner::Sunday { rank } => (2u8, *rank),
                    FeastRank54Inner::Vigil { rank } => (3u8, *rank),
                    FeastRank54Inner::Feria { rank, .. } => (4u8, *rank),
                };
                let (ptype2, sub2) = match other {
                    FeastRank54Inner::Feast { rank, .. } => (0u8, *rank as u8),
                    FeastRank54Inner::Octave { rank, .. } => (1u8, *rank),
                    FeastRank54Inner::Sunday { rank } => (2u8, *rank),
                    FeastRank54Inner::Vigil { rank } => (3u8, *rank),
                    FeastRank54Inner::Feria { rank, .. } => (4u8, *rank),
                };
                if ptype1 < ptype2 {
                    Ok(OccurrenceResult::FirstNothingOfSecond)
                } else if ptype1 > ptype2 {
                    Ok(OccurrenceResult::SecondNothingOfFirst)
                } else {
                    // same variant category: lower subrank wins
                    if sub1 < sub2 {
                        Ok(OccurrenceResult::FirstNothingOfSecond)
                    } else if sub1 > sub2 {
                        Ok(OccurrenceResult::SecondNothingOfFirst)
                    } else {
                        // deterministic fallback: prefer self
                        Ok(OccurrenceResult::FirstNothingOfSecond)
                    }
                }
            }
        }
    }

    fn get_rank_string(&self) -> String {
        match self {
            FeastRank54Inner::Feria { rank, flags } => {
                let mut parts = match rank {
                    1 => vec!["Greater Privileged Feria".to_string()],
                    2 => vec!["Greater Non-Privileged Feria".to_string()],
                    3 => vec!["Ordinary Feria".to_string()],
                    _ => panic!("Unknown feria rank: {}", rank),
                };
                if flags.contains(FeriaFlags::OF_LENT) {
                    parts.push("of Lent".to_string());
                }
                if flags.contains(FeriaFlags::EMBER_DAY) {
                    parts.push("Ember Day".to_string());
                }
                parts.join(" ")
            }
            FeastRank54Inner::Feast { rank, flags } => {
                let base_name = match rank {
                    FeastClass::FirstClassDouble => "First Class Double",
                    FeastClass::SecondClassDouble => "Second Class Double",
                    FeastClass::MajorDouble => "Major Double",
                    FeastClass::Double => "Double",
                    FeastClass::Semidouble => "Semidouble",
                    FeastClass::Simple => "Simple",
                    FeastClass::Commemoration => "Commemoration",
                };
                let mut parts = vec![base_name.to_string()];
                if flags.contains(FeastFlags::OF_OUR_LORD) {
                    parts.push("of Our Lord".to_string());
                }
                if flags.contains(FeastFlags::IMMACULATE_CONCEPTION) {
                    parts.push("(Immaculate Conception)".to_string());
                }
                if flags.contains(FeastFlags::MOVABLE) {
                    parts.push("(Movable)".to_string());
                }
                if flags.contains(FeastFlags::ALL_SOULS) {
                    parts.push("(All Souls)".to_string());
                }
                parts.join(" ")
            }
            FeastRank54Inner::Vigil { rank } => match rank {
                1 => "Vigil of the First Class",
                2 => "Vigil of the Second Class",
                3 => "Vigil of the Third Class",
                _ => "Unknown Vigil",
            }
            .to_string(),
            FeastRank54Inner::Sunday { rank } => match rank {
                1 => "Greater Sunday of the First Class",
                2 => "Greater Sunday of the Second Class",
                3 => "Lesser Sunday",
                _ => "Unknown Sunday",
            }
            .to_string(),
            FeastRank54Inner::Octave {
                rank,
                is_octave_day,
                octave_type: _,
            } => match (rank, is_octave_day) {
                (1, true) => "Octave Day of the First Class",
                (1, false) => "In an Octave of the First Class",
                (2, true) => "Octave Day of the Second Class",
                (2, false) => "In an Octave of the Second Class",
                (3, true) => "Octave Day of the Third Class",
                (3, false) => "In an Octave of the Third Class",
                _ => "Unknown Octave",
            }
            .to_string(),
        }
    }

    fn new_with_context(rank: &str, day_type: &DayType, context: &LiturgicalContext) -> Self {
        // Create flags based on context
        let mut feast_flags = FeastFlags::empty();
        let mut feria_flags = FeriaFlags::empty();

        if context.of_our_lord {
            feast_flags |= FeastFlags::OF_OUR_LORD;
        }
        if context.is_movable {
            feast_flags |= FeastFlags::MOVABLE;
        }
        if context.of_lent {
            feria_flags |= FeriaFlags::OF_LENT;
        }

        // Parse feast name for special cases
        if let Some(feast_name) = &context.feast_name {
            if feast_name.contains("Immaculate Conception") {
                feast_flags |= FeastFlags::IMMACULATE_CONCEPTION;
            }
            if feast_name.contains("All Souls") {
                feast_flags |= FeastFlags::ALL_SOULS;
            }
        }

        // Parse rank string and day type to determine specific rank
        match day_type {
            DayType::Feria => {
                // Check for special feria types in 1954
                let rank = match rank {
                    "greater privileged" | "I" => 1, // Ash Wednesday and Monday, Tuesday, and Wednesday of Holy Week. No feast day could be celebrated on these days.
                    "greater non-privileged" | "II" => 2, // The ferias of Advent, Lent, and Passion Week, Rogation Monday, and the Ember Days. Any feast day except a Simple could occur on these days, with a commemoration of the feria.
                    "ordinary" | "III" => 3,              // Ordinary ferias
                    "IV" => 3,                            // Ordinary ferias
                    _ => panic!("Unknown feria rank: {}", rank),
                };

                // Special handling for Ember days
                if let Some(season) = &context.season_name {
                    if season.contains("Ember") {
                        feria_flags |= FeriaFlags::EMBER_DAY;
                    }
                }

                FeastRank54Inner::Feria {
                    rank,
                    flags: feria_flags,
                }
            }
            DayType::Feast => {
                // Map 1954 liturgical rank strings to feast types
                let feast_rank = match rank {
                    "totum_duplex" | "first_class_duplex" | "first class double" | "I" => {
                        FeastClass::FirstClassDouble
                    }
                    "second_class_duplex" | "second class double" | "II" => {
                        FeastClass::SecondClassDouble
                    }
                    "major_duplex" | "greater_duplex" | "major double" => FeastClass::MajorDouble,
                    "duplex" | "double" | "III" => FeastClass::Double,
                    "semiduplex" | "semidouble" | "IV" => FeastClass::Semidouble,
                    "simplex" | "simple" | "V" => FeastClass::Simple,
                    "commemoratio" | "commemoration" | "com" | "VI" => FeastClass::Commemoration,
                    _ => FeastClass::Simple,
                };
                FeastRank54Inner::Feast {
                    rank: feast_rank,
                    flags: feast_flags,
                }
            }
            DayType::Sunday => {
                let rank = match rank {
                    "I" => 1,   // Major sundays (Easter, Pentecost, etc.)
                    "II" => 2,  // Important sundays
                    "III" => 3, // Ordinary sundays
                    _ => 2,     // Default to second class
                };
                FeastRank54Inner::Sunday { rank }
            }
            DayType::Vigil => {
                let rank = match rank {
                    "I" => 1,   // Major vigils
                    "II" => 2,  // Important vigils
                    "III" => 3, // Lesser vigils
                    _ => 2,     // Default to second class
                };
                FeastRank54Inner::Vigil { rank }
            }
            DayType::Octave => {
                let rank = match rank {
                    "I" => 1,
                    "II" => 2,
                    "III" => 3,
                    _ => 2,
                };
                // Try to get octave_type from context.season_name or feast_name
                let octave_type = if let Some(season) = &context.season_name {
                    if season.contains("Easter Octave") || season.contains("Pentecost Octave") {
                        OctaveType::Privileged1
                    } else if season.contains("Epiphany Octave") {
                        OctaveType::Privileged2
                    } else if season.contains("Christmas Octave") {
                        OctaveType::Privileged3
                    } else if season.contains("Immaculate Conception")
                        || season.contains("Assumption")
                        || season.contains("St. John the Baptist")
                        || season.contains("Ss. Peter and Paul")
                        || season.contains("All Saints")
                    {
                        OctaveType::Common
                    } else if season.contains("St. Stephen")
                        || season.contains("St. John Apostle")
                        || season.contains("Holy Innocents")
                        || season.contains("Nativity of Mary")
                    {
                        OctaveType::Simple
                    } else {
                        OctaveType::Common
                    }
                } else {
                    OctaveType::Common
                };
                FeastRank54Inner::Octave {
                    rank,
                    is_octave_day: context.is_octave_day,
                    octave_type,
                }
            }
        }
    }
}

/// Check if a feast can be commemorated according to 1954 rules
fn can_commemorate_1954(winning_rank: &FeastRank54Inner) -> bool {
    match winning_rank {
        FeastRank54Inner::Feast {
            rank: FeastClass::FirstClassDouble,
            ..
        } => false, // First Class Double excludes commemorations
        FeastRank54Inner::Feast {
            rank: FeastClass::SecondClassDouble,
            ..
        } => false, // Second Class Double excludes commemorations
        FeastRank54Inner::Feast {
            rank: FeastClass::MajorDouble,
            ..
        } => false, // Major Double excludes commemorations
        FeastRank54Inner::Feast {
            rank: FeastClass::Double,
            ..
        } => false, // Double excludes commemorations
        FeastRank54Inner::Sunday { rank } if *rank <= 2 => false, // Important sundays exclude commemorations
        FeastRank54Inner::Octave { .. } => false,                 // Octaves exclude commemorations
        FeastRank54Inner::Feria { rank: 1, .. } => false, // Ash Wednesday excludes commemorations
        _ => true, // Semidouble, Simple, and other ranks allow commemorations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feast_rank_54_precedence() {
        let context = LiturgicalContext::new();

        let christmas =
            FeastRank54::new_with_context("I", &DayType::Feast, &context.clone().of_our_lord());
        let saint_feast = FeastRank54::new_with_context("III", &DayType::Feast, &context);

        let competetors = vec![
            (christmas, "Christmas".to_string()),
            (saint_feast, "St. John".to_string()),
        ];

        let result = FeastRank54::resolve_conflicts(&competetors);
        assert_eq!(result.winner, "Christmas");
    }
}
