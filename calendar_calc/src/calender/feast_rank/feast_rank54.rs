use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use core::panic;
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
        major: bool,
    },
    Sunday {
        rank: u8,
    },
    Octave {
        rank: OctaveType,
        is_octave_day: bool,
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
            } | FeastRank54Inner::Octave { rank: OctaveType::Privileged1, .. }
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

    fn admits_bvm_on_saturday(&self) -> super::BVMOnSaturdayResult {
        // admit BVM on Saturday if feria rank is 3
        if let FeastRank54Inner::Feria { rank: 3, .. } = self.0 {
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
            FeastRank54Inner::Vigil { major } => {
                if *major {
                    1
                } else {
                    2
                }
            } // Vigil ranks start from 16
            FeastRank54Inner::Sunday { rank } => *rank, // Sunday ranks start from 11
            FeastRank54Inner::Octave { rank, .. } => match rank {
                OctaveType::Privileged1 => 1,
                OctaveType::Privileged2 => 2,
                OctaveType::Privileged3 => 3,
                OctaveType::Common => 4,
                OctaveType::Simple => 5,
            },
        }
    }

    fn resolve_conflicts<T: Clone + Debug>(
        competetors: &[(Self, T)],
    ) -> ResolveConflictsResult<FeastRank54, T> {
        if competetors.is_empty() {
            panic!("No competetors provided for conflict resolution");
        }

        let mut sorted_competetors = competetors.to_vec();
        // sorted_competetors.sort_by(|(rank_a, _), (rank_b, _)| {
        //     rank_a.get_numeric_rank().cmp(&rank_b.get_numeric_rank())
        // });

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

        let _winner_rank = winning_rank.get_numeric_rank();

        
            commemorations.extend(base_commemorations);
        
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
            // other is a feria
            if let FeastRank54Inner::Feria {
                rank: rank2,
                flags: _flags2,
            } = other
            {
                match (rank1, rank2) {
                    (r, 1) if (*r as u8) <= (FeastClass::Double as u8) => {
                        return Ok(OccurrenceResult::SecondCommemorationOfFirst)
                    }
                    (_, 1) => return Ok(OccurrenceResult::SecondNothingOfFirst),
                    (FeastClass::Simple, 2) => return Ok(OccurrenceResult::SecondNothingOfFirst),
                    (_, 2) => return Ok(OccurrenceResult::FirstCommemorationOfSecond),
                    (_, 3) => return Ok(OccurrenceResult::FirstNothingOfSecond),
                    _ => {}
                }
            }

            // other is octave
            if let FeastRank54Inner::Octave {
                rank: rank2,
                is_octave_day,
            } = other
            {
                match *rank2 {
                    OctaveType::Privileged1 => {
                        match (rank1, is_octave_day) {
                            (FeastClass::FirstClassDouble, _) => {
                                return Ok(OccurrenceResult::FirstNothingOfSecond)
                            }
                            (FeastClass::SecondClassDouble, _) => {
                                return Ok(OccurrenceResult::SecondTransferOfFirst)
                            }
                            (_, true) => return Ok(OccurrenceResult::SecondCommemorationOfFirst),
                            (_, false) => return Ok(OccurrenceResult::SecondNothingOfFirst),
                        }

                    }
                    OctaveType::Privileged2 => {
                        if *rank1 == FeastClass::FirstClassDouble {
                            if *is_octave_day {
                                return Ok(OccurrenceResult::SecondTransferOfFirst);
                            } else {
                                return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                            }
                        } else if rank1 == &FeastClass::MajorDouble
                            && flags1.contains(FeastFlags::OF_OUR_LORD)
                        {
                            return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                        }
                         else {
                            return Ok(OccurrenceResult::FirstNothingOfSecond);
                        }
                    }
                    // Doubles and above win over common/privileged3 octaves, octave commemorated
                    OctaveType::Privileged3 | OctaveType::Common => match rank1 {
                        FeastClass::Semidouble
                        | FeastClass::Double
                        | FeastClass::MajorDouble
                        | FeastClass::SecondClassDouble
                        | FeastClass::FirstClassDouble if !is_octave_day => {
                            return Ok(OccurrenceResult::FirstCommemorationOfSecond)
                        }
                        FeastClass::SecondClassDouble if *is_octave_day => {
                            return Ok(OccurrenceResult::FirstNothingOfSecond)
                        }
                        FeastClass::Simple => return Ok(OccurrenceResult::SecondCommemorationOfFirst),
                        _ => return Ok(OccurrenceResult::FirstNothingOfSecond),
                    },
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
                    (FeastClass::SecondClassDouble, FeastClass::Double)=> {
                        return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                    }
                    (FeastClass::Simple, FeastClass::FirstClassDouble) => {
                        return Ok(OccurrenceResult::SecondNothingOfFirst);
                    }
                    (FeastClass::Simple, _) => {
                        return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                    }
                    (FeastClass::Semidouble, _) => {
                        return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                    }
                    (
                        FeastClass::FirstClassDouble,
                        FeastClass::FirstClassDouble | FeastClass::SecondClassDouble,
                    ) => {
                        return Ok(OccurrenceResult::SecondTransferOfFirst);
                    }
                    (FeastClass::FirstClassDouble, FeastClass::MajorDouble | FeastClass::Double) => {
                        return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                    }
                    (FeastClass::FirstClassDouble, _) => {
                        return Ok(OccurrenceResult::FirstNothingOfSecond);
                    }
                    (FeastClass::MajorDouble, FeastClass::Double) => {
                        return Ok(OccurrenceResult::FirstCommemorationOfSecond)
                    }
                    (FeastClass::MajorDouble, FeastClass::MajorDouble)
                        if _flags2.contains(FeastFlags::OF_OUR_LORD) =>
                    {
                        return Ok(OccurrenceResult::SecondCommemorationOfFirst)
                    }
                    (FeastClass::Double, FeastClass::Double)
                        if _flags2.contains(FeastFlags::OF_OUR_LORD) =>
                    {
                        return Ok(OccurrenceResult::SecondCommemorationOfFirst)
                    }
                    _ => {}
                }
            }
            // other is vigil
            if let FeastRank54Inner::Vigil { major: rank2 } = other {
                match (rank1, rank2) {
                    (FeastClass::FirstClassDouble, _) => {
                        return Ok(OccurrenceResult::FirstNothingOfSecond);
                    }
                    (FeastClass::SecondClassDouble, true) => {
                        return Ok(OccurrenceResult::FirstCommemorationOfSecond);
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
            // other is sunday — follow 1954 rules (explicit arms per user spec):
            // - Sunday I: no feast may be celebrated; feasts are commemorated (except Easter/Pentecost which cannot be commemorated — not detectable here)
            // - Sunday II: only Doubles of the I Class may be celebrated; other feasts are commemorated
            // - Lesser Sundays: Doubles of I or II class, or a feast of Our Lord, may be celebrated; others are commemorated
            if let FeastRank54Inner::Sunday { rank: rank2 } = other {
                match rank2 {
                    // Greater Sunday of the I class: Sunday wins; feast becomes a commemoration of the Sunday
                    1 => {
                        return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                    }
                    // Greater Sunday of the II class: only First Class Doubles may be celebrated
                    2 => match rank1 {
                        FeastClass::FirstClassDouble => {
                            // Feast (first) may be celebrated on Sunday II
                            return Ok(OccurrenceResult::FirstNothingOfSecond);
                        }
                        FeastClass::SecondClassDouble | FeastClass::MajorDouble if flags1.contains(FeastFlags::OF_OUR_LORD) => {
                            // Feast (first) may be celebrated on Sunday II if it is a feast of Our Lord
                            return Ok(OccurrenceResult::FirstNothingOfSecond);
                        }
                        _ => {
                            // Feast is commemorated
                            return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                        }
                    },
                    // Lesser Sundays: Doubles of I or II class, or a feast of Our Lord, take precedence
                    3 => match rank1 {
                        FeastClass::FirstClassDouble | FeastClass::SecondClassDouble => {
                            return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                        }
                        _ => {
                            if flags1.contains(FeastFlags::OF_OUR_LORD) {
                                return Ok(OccurrenceResult::FirstNothingOfSecond);
                            } else {
                                return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                            }
                        }
                    },
                    // Unknown/other sunday rank: fall through to numeric fallback
                    _ => {}
                }
            }
        }

        // self is vigil
        if let FeastRank54Inner::Vigil { major: rank1 } = self {
            if let FeastRank54Inner::Octave {
                rank: rank2,
                is_octave_day: _,
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
                    (true, FeastClass::FirstClassDouble) => {
                        return Ok(OccurrenceResult::SecondNothingOfFirst);
                    }
                    (true, _) => return Ok(OccurrenceResult::FirstNothingOfSecond),
                    (false, FeastClass::FirstClassDouble) => {
                        return Ok(OccurrenceResult::SecondNothingOfFirst)
                    }
                    (false, _) => return Ok(OccurrenceResult::FirstNothingOfSecond),
                }
            }
            if let FeastRank54Inner::Feria {
                rank: rank2,
                flags: _flags2,
            } = other
            {
                match (rank1, rank2) {
                    (true, 2) => return Ok(OccurrenceResult::FirstNothingOfSecond),
                    (_, 2) => return Ok(OccurrenceResult::SecondCommemorationOfFirst),
                    (_, 3) => return Ok(OccurrenceResult::FirstNothingOfSecond),
                    _ => {}
                }
            }
            if let FeastRank54Inner::Sunday { rank: rank2 } = other {
                return Ok(OccurrenceResult::SecondCommemorationOfFirst);
            }
        }

        // self is octave
        if let FeastRank54Inner::Octave {
            rank: rank1,
            is_octave_day: is_octave_day1,
        } = self
        {
            // ferias: octaves generally outrank ferial days; simple octave days are weaker
            if let FeastRank54Inner::Feria {
                rank: rank2,
                flags: _flags2,
            } = other
            {
                match (rank1, rank2) {
                    // simple octaves yield to ferias (rare), keep previous conservative behavior
                    (OctaveType::Simple, _) if *is_octave_day1 => return Ok(OccurrenceResult::FirstNothingOfSecond),
                    (OctaveType::Simple, _) => return Ok(OccurrenceResult::SecondNothingOfFirst),
                    (OctaveType::Common, 2) => {
                        return Ok(OccurrenceResult::FirstCommemorationOfSecond)
                    }
                    _ => return Ok(OccurrenceResult::FirstNothingOfSecond),
                }
            }

            // Sundays that fall within an octave follow octave rules; treat them similarly to feasts here
            if let FeastRank54Inner::Sunday { rank: rank2 } = other {
                match (is_octave_day1, rank1, rank2) {
                    (_, _, 1) => {
                        return Ok(OccurrenceResult::SecondNothingOfFirst);
                    }
                    (true, OctaveType::Common, 2) => {
                        return Ok(OccurrenceResult::SecondCommemorationOfFirst)
                    }
                    (false, OctaveType::Privileged1, _) => {
                        return Ok(OccurrenceResult::FirstNothingOfSecond)
                    }
                    (false, OctaveType::Privileged2, _) => {
                        // privileged2 octaves overpower Sundays in practice (only First Class Double may displace on octave day)
                        return Ok(OccurrenceResult::FirstNothingOfSecond);
                    }
                    (_, OctaveType::Privileged3, _) => {
                        return Ok(OccurrenceResult::SecondNothingOfFirst);
                    }
                    (
                        false,
                        OctaveType::Privileged3 | OctaveType::Common | OctaveType::Simple,
                        _,
                    ) => {
                        // these octaves yield to Sundays (Sundays are liturgically higher than a simple octave)
                        return Ok(OccurrenceResult::SecondNothingOfFirst);
                    }
                    _ => {}
                }
            }

            // octave vs octave: fall through to numeric tie-breaker
            if let FeastRank54Inner::Octave {
                rank: _rank2,
                is_octave_day: _,
            } = other
            {
                // fall through to final numeric tie-breaker
            }
        }

        // self is sunday
        if let FeastRank54Inner::Sunday { rank: rank1 } = self {
            // other is feria
            if let FeastRank54Inner::Feria {
                rank: rank2,
                flags: _flags2,
            } = other
            {
                match (rank1, rank2) {
                    (1, _) => return Ok(OccurrenceResult::FirstNothingOfSecond),
                    (2, 1) => return Ok(OccurrenceResult::FirstNothingOfSecond),
                    (2, _) => return Ok(OccurrenceResult::SecondCommemorationOfFirst),
                    (3, 1) => return Ok(OccurrenceResult::FirstNothingOfSecond),
                    (3, 2) => return Ok(OccurrenceResult::FirstNothingOfSecond),
                    (3, _) => return Ok(OccurrenceResult::SecondCommemorationOfFirst),
                    _ => {}
                }
            }
        }

        // try swapping the order
        if try_swapped {
            return other.resolve_occurrence(self, false).map(|r| r.reverse());
        }
        // No explicit rule matched; fall through to numeric-rank fallback below.
        // just pick higher rank or apply tie-breaker if equal
        bail!(
            "No explicit occurrence rule matched between {:?} and {:?}",
            self,
            other
        );
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
                base_name.to_string()
            }
            FeastRank54Inner::Vigil { major } => match *major {
                true => "Major Vigil",
                false => "Minor Vigil",
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
            } => match (rank, is_octave_day) {
                (OctaveType::Privileged1, true) => "Octave Day of a Privileged 1st Class Octave",
                (OctaveType::Privileged1, false) => "Day within a Privileged 1st Class Octave",
                (OctaveType::Privileged2, true) => "Octave Day of a Privileged 2nd Class Octave",
                (OctaveType::Privileged2, false) => "Day within a Privileged 2nd Class Octave",
                (OctaveType::Privileged3, true) => "Octave Day of a Privileged 3rd Class Octave",
                (OctaveType::Privileged3, false) => "Day within a Privileged 3rd Class Octave",
                (OctaveType::Common, true) => "Major Double",
                (OctaveType::Common, false) => "Day within a Common Octave",
                (OctaveType::Simple, true) => "Simple",
                (OctaveType::Simple, false) => "Day within a Simple Octave",
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
                    _ => panic!("Unknown feast rank: {}", rank),
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
                    _ => panic!("Unknown sunday rank: {}", rank),
                };
                FeastRank54Inner::Sunday { rank }
            }
            DayType::Vigil => {
                let rank = match rank {
                    "major" | "I" => true,
                    "minor" | "II" => false,
                    _ => panic!("Unknown vigil rank: {}", rank),
                };
                FeastRank54Inner::Vigil { major: rank}
            }
            DayType::Octave => {
                let rank = match rank {
                    "privileged1" | "I" => OctaveType::Privileged1,
                    "privileged2" | "II" => OctaveType::Privileged2,
                    "privileged3" | "III" => OctaveType::Privileged3,
                    "common" | "IV" => OctaveType::Common,
                    "simple" | "V" => OctaveType::Simple,
                    _ => panic!("Unknown octave rank: {}", rank),
                };
                FeastRank54Inner::Octave {
                    rank,
                    is_octave_day: context.is_octave_day,
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
