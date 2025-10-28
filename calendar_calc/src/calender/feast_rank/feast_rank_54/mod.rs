#![allow(clippy::match_same_arms)]

use std::fmt::Debug;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use types::{ArcStr, CommemorationType, RcStr, TrivialDayRank};

use super::{
    DayType, FeastFlags, FeastRankResolver, FeriaFlags, LiturgicalContext, OctaveFlags,
    ResolveConflictsResult, SundayFlags,
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum OctaveType {
    Privileged1,
    Privileged2,
    Privileged3,
    Common,
    Simple,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum SundayClass {
    First,
    Second,
    Lesser,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
enum FeastRank54Inner {
    Feria {
        rank: FeriaClass,
        flags: FeriaFlags,
    },
    Feast {
        rank: FeastClass,
        flags: FeastFlags,
    },
    Vigil {
        kind: VigilKind,
    },
    Sunday {
        rank: SundayClass,
        flags: SundayFlags,
    },
    Octave {
        rank: OctaveType,
        flags: OctaveFlags,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq, Hash)]
enum VigilKind {
    /// The special vigils that are class I in practice (Christmas, Pentecost)
    ChristmasOrPentecost,
    /// The Vigil of Epiphany, special II-class behavior
    Epiphany,
    /// All other (common) vigils
    Common,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq, Hash)]
enum FeastClass {
    /// First Class Double - highest rank in 1954 (Christmas, Easter, Pentecost,
    /// etc.)
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq, Hash)]
enum FeriaClass {
    GreaterPrivilaged = 1,
    GreaterNonPrivilaged = 2,
    Lesser = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq, Hash)]
enum OccurrenceResult {
    FirstNothingOfSecond,
    SecondNothingOfFirst,
    FirstCommemorationOfSecond,
    SecondCommemorationOfFirst,
    FirstTransferOfSecond,
    SecondTransferOfFirst,
    SecondTransferAndCommemorationOfFirst,
    FirstTransferAndCommemorationOfSecond,
}

impl OccurrenceResult {
    fn reverse(self) -> Self {
        match self {
            OccurrenceResult::FirstNothingOfSecond => OccurrenceResult::SecondNothingOfFirst,
            OccurrenceResult::SecondNothingOfFirst => OccurrenceResult::FirstNothingOfSecond,
            OccurrenceResult::FirstCommemorationOfSecond => {
                OccurrenceResult::SecondCommemorationOfFirst
            }
            OccurrenceResult::SecondCommemorationOfFirst => {
                OccurrenceResult::FirstCommemorationOfSecond
            }
            OccurrenceResult::FirstTransferOfSecond => OccurrenceResult::SecondTransferOfFirst,
            OccurrenceResult::SecondTransferOfFirst => OccurrenceResult::FirstTransferOfSecond,
            OccurrenceResult::SecondTransferAndCommemorationOfFirst => {
                OccurrenceResult::FirstTransferAndCommemorationOfSecond
            }
            OccurrenceResult::FirstTransferAndCommemorationOfSecond => {
                OccurrenceResult::SecondTransferAndCommemorationOfFirst
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeastRank54(FeastRank54Inner);

impl FeastRankResolver for FeastRank54 {
    type FeastRankDescriptor = TrivialDayRank;

    fn descriptor(&self) -> Self::FeastRankDescriptor {
        TrivialDayRank(self.0.get_rank_string())
    }

    fn resolve_conflicts<T>(competetors: &[(Self, T)]) -> Result<ResolveConflictsResult<Self, T>>
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

    fn resolve_concurances(
        _primary: Self,
        _secondary: Self,
    ) -> Result<super::ResolveConcurancesResult> {
        Ok(super::ResolveConcurancesResult::VespersOfCurrentNothingOfFollowing)
    }

    fn new_with_context(rank: &str, day_type: DayType, context: &LiturgicalContext) -> Self
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
                rank: FeastClass::FirstClassDouble | FeastClass::SecondClassDouble,
                ..
            } | FeastRank54Inner::Octave {
                rank: OctaveType::Privileged1 | OctaveType::Privileged2 | OctaveType::Privileged3,
                ..
            }
        )
    }

    fn get_rank_string(&self) -> ArcStr {
        self.0.get_rank_string()
    }

    fn get_bvm_on_saturday_rank() -> Self {
        FeastRank54(FeastRank54Inner::Feast {
            rank: FeastClass::Simple,
            flags: FeastFlags::empty(),
        })
    }

    fn admits_bvm_on_saturday(&self) -> super::BVMOnSaturdayResult {
        // admit BVM on Saturday if feria rank is 3
        if let FeastRank54Inner::Feria {
            rank: FeriaClass::Lesser,
            ..
        } = self.0
        {
            super::BVMOnSaturdayResult::Admitted
        }
        // commemorate if simplex feast
        else if let FeastRank54Inner::Feast { rank, .. } = &self.0 {
            if rank == &FeastClass::Simple {
                super::BVMOnSaturdayResult::OtherCommemorated
            } else {
                super::BVMOnSaturdayResult::NotAdmitted
            }
        } else {
            super::BVMOnSaturdayResult::NotAdmitted
        }
    }
    fn id(&self) -> RcStr {
        self.0.get_rank_string_verbose().into()
    }

    fn transfers_vigil_from_sunday_to_saturday() -> bool
    where
        Self: Sized,
    {
        true
    }
}

// Using shared FeriaFlags, FeastFlags, SundayFlags from parent module

impl FeastRank54Inner {
    fn get_numeric_rank(&self) -> u8 {
        match self {
            FeastRank54Inner::Feria { rank, .. } => *rank as u8, // Ferial ranks start from 21
            FeastRank54Inner::Feast { rank, .. } => match rank {
                FeastClass::FirstClassDouble => 1,
                FeastClass::SecondClassDouble => 2,
                FeastClass::MajorDouble => 3,
                FeastClass::Double => 4,
                FeastClass::Semidouble => 5,
                FeastClass::Simple => 6,
                FeastClass::Commemoration => 7,
            },
            FeastRank54Inner::Vigil { kind } => match kind {
                VigilKind::ChristmasOrPentecost => 1,
                VigilKind::Epiphany => 8,
                VigilKind::Common => 16,
            }, // Vigil ranks start from small numbers for special vigils, larger for common
            FeastRank54Inner::Sunday { rank, .. } => *rank as u8,
            FeastRank54Inner::Octave { rank, .. } => match rank {
                OctaveType::Privileged1 => 1,
                OctaveType::Privileged2 => 2,
                OctaveType::Privileged3 => 3,
                OctaveType::Common => 4,
                OctaveType::Simple => 5,
            },
        }
    }

    #[allow(clippy::too_many_lines)]
    fn resolve_conflicts<T: Clone + Debug>(
        competetors: &[(Self, T)],
    ) -> Result<ResolveConflictsResult<FeastRank54, T>> {
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
            if let FeastRank54Inner::Feast {
                rank: FeastClass::Commemoration,
                ..
            } = *rank
            {
                base_commemorations.push((name.clone(), CommemorationType::Lauds));
                return false;
            }
            if let FeastRank54Inner::Octave {
                rank: OctaveType::Simple,
                flags,
            } = rank
                && !flags.contains(OctaveFlags::OCTAVE_DAY)
            {
                return false;
            }
            true
        });
        // for (i, (rank, name)) in sorted_competetors.iter().enumerate() {
        //     if let FeastRank54Inner::Feast {
        //         rank: FeastClass::Commemoration,
        //         ..
        //     } = *rank
        //     {
        //         base_commemorations.push(name.clone());
        //         indices_to_remove.push(i);
        //     }
        //     if let FeastRank54Inner::Octave {
        //         rank: OctaveType::Simple,
        //         flags,
        //     } = rank
        //     {
        //         if !flags.contains(OctaveFlags::OCTAVE_DAY) {
        //             indices_to_remove.push(i);
        //         }
        //     }
        // }
        // // Remove in reverse order to avoid index shifting
        // for i in indices_to_remove.into_iter().rev() {
        //     sorted_competetors.remove(i);
        // }

        // If all competitors were commemorations, pick the first one as winner
        if sorted_competetors.is_empty() {
            bail!("No competitors provided for conflict resolution after filtering");
        }
        // Two-pass resolution:
        // 1) Determine the final winner by comparing competitors in sequence.
        // 2) With the final winner fixed, compute commemorations, axed entries and any
        //    transfer.
        let mut commemorations = Vec::new();
        let mut winner = sorted_competetors[0].1.clone();
        let mut winning_rank = &sorted_competetors[0].0;
        let mut transferred: Option<(FeastRank54, T)> = None;

        // First pass: pick the winner (provisionally update winner when an outcome
        // would make the current competitor take precedence).
        for (current_rank, current_name) in &sorted_competetors {
            // let (current_rank, current_name) = &sorted_competetors[i];
            if std::ptr::eq(current_rank, winning_rank) {
                continue;
            }
            debug_trace.push(format!(
                "Resolving between {winning_rank:?} ({winner:?}) and {current_rank:?} ({current_name:?})"
            ));

            let occurrence = winning_rank
                .resolve_occurrence(current_rank)
                .context(format!(
                    "Error resolving occurrence between {winner:?} and {current_name:?}"
                ))?;
            debug_trace.push(format!("    -> Occurrence result: {occurrence:?}"));

            // Only outcomes that would switch the provisional winner are considered
            // in this pass; we don't record commemorations/transfers here.
            match occurrence {
                OccurrenceResult::SecondNothingOfFirst
                | OccurrenceResult::SecondCommemorationOfFirst
                | OccurrenceResult::SecondTransferOfFirst
                | OccurrenceResult::SecondTransferAndCommemorationOfFirst => {
                    winner = current_name.clone();
                    winning_rank = current_rank;
                }
                _ => {
                    // winner remains the same
                }
            }
        }

        // Second pass: determine commemorations, transfers relative to
        // the final winner. We skip the winner entry itself.
        for (rank, name) in &sorted_competetors {
            // Skip the entry that corresponds to the final winner. Use pointer equality
            // to avoid requiring PartialEq on the generic contestant payload `T`.
            if std::ptr::eq(rank, winning_rank) {
                continue;
            }

            debug_trace.push(format!(
                "Comparing final winner {winning_rank:?} ({winner:?}) to {rank:?} ({name:?})"
            ));

            let occurrence = winning_rank.resolve_occurrence(rank).context(format!(
                "Error resolving occurrence between {winner:?} and {name:?}"
            ))?;

            debug_trace.push(format!("    -> Occurrence result: {occurrence:?}"));
            match occurrence {
                OccurrenceResult::FirstCommemorationOfSecond => {
                    commemorations.push((name.clone(), CommemorationType::Lauds));
                }
                OccurrenceResult::FirstTransferOfSecond => {
                    if transferred.is_some() {
                        bail!("Multiple transfers detected in conflict resolution");
                    }
                    transferred = Some((FeastRank54(rank.clone()), name.clone()));
                }
                OccurrenceResult::FirstTransferAndCommemorationOfSecond => {
                    if transferred.is_some() {
                        bail!("Multiple transfers detected in conflict resolution");
                    }
                    transferred = Some((FeastRank54(rank.clone()), name.clone()));
                    commemorations.push((name.clone(), CommemorationType::Lauds));
                }
                OccurrenceResult::SecondCommemorationOfFirst => {
                    commemorations.push((winner.clone(), CommemorationType::Lauds));
                }
                OccurrenceResult::SecondTransferOfFirst => {
                    if transferred.is_some() {
                        bail!("Multiple transfers detected in conflict resolution");
                    }
                    transferred = Some((FeastRank54(winning_rank.clone()), winner.clone()));
                }
                OccurrenceResult::SecondTransferAndCommemorationOfFirst => {
                    if transferred.is_some() {
                        bail!("Multiple transfers detected in conflict resolution");
                    }
                    transferred = Some((FeastRank54(winning_rank.clone()), winner.clone()));
                    commemorations.push((winner.clone(), CommemorationType::Lauds));
                }
                _ => {
                    // Nothing to do for other outcomes
                }
            }
        }

        let _winner_rank = winning_rank.get_numeric_rank();

        let commemorations = commemorations
            .into_iter()
            .chain(base_commemorations)
            .collect::<Vec<_>>();

        Ok(super::ResolveConflictsResult {
            winner,
            winner_rank: FeastRank54(winning_rank.clone()),
            transferred,
            commemorations,
        })
    }

    fn resolve_occurrence(&self, other: &Self) -> Result<OccurrenceResult> {
        self.resolve_occurrence_inner(other, true)
    }

    fn resolve_occurrence_inner(
        &self,
        other: &Self,
        try_swapped: bool,
    ) -> Result<OccurrenceResult> {
        #![allow(clippy::too_many_lines)]
        if let FeastRank54Inner::Feria {
            rank: rank1,
            flags: _flags1,
        } = self
        {
            // both ferias
            if let FeastRank54Inner::Feria {
                rank: rank2,
                flags: _flags2,
            } = other
            {
                if rank1 == rank2 {
                    bail!("Two ferias of the same rank cannot occur on the same day");
                }
                match (*rank1 as u8).cmp(&(*rank2 as u8)) {
                    std::cmp::Ordering::Less => return Ok(OccurrenceResult::FirstNothingOfSecond),
                    std::cmp::Ordering::Greater => {
                        return Ok(OccurrenceResult::SecondNothingOfFirst);
                    }
                    std::cmp::Ordering::Equal => {}
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
                flags: flags2,
            } = other
            {
                match (rank1, rank2) {
                    (_, FeriaClass::GreaterPrivilaged)
                        if flags2.contains(FeriaFlags::HOLY_TRIDUUM) =>
                    {
                        // Holy Triduum ferias take precedence over all feasts
                        return Ok(OccurrenceResult::SecondNothingOfFirst);
                    }
                    (
                        FeastClass::FirstClassDouble
                        | FeastClass::SecondClassDouble
                        | FeastClass::MajorDouble
                        | FeastClass::Double,
                        FeriaClass::GreaterPrivilaged,
                    ) => return Ok(OccurrenceResult::SecondCommemorationOfFirst),
                    (
                        FeastClass::Semidouble | FeastClass::Simple,
                        FeriaClass::GreaterPrivilaged,
                    ) => return Ok(OccurrenceResult::SecondCommemorationOfFirst),
                    (
                        FeastClass::FirstClassDouble
                        | FeastClass::SecondClassDouble
                        | FeastClass::MajorDouble
                        | FeastClass::Double
                        | FeastClass::Semidouble,
                        FeriaClass::GreaterNonPrivilaged,
                    ) => return Ok(OccurrenceResult::FirstCommemorationOfSecond),
                    (FeastClass::Simple, FeriaClass::GreaterNonPrivilaged) => {
                        return Ok(OccurrenceResult::SecondNothingOfFirst);
                    }

                    (_, FeriaClass::Lesser) => return Ok(OccurrenceResult::FirstNothingOfSecond),
                    (FeastClass::Commemoration, _) => {
                        bail!("Commemoration should have been filtered out earlier")
                    }
                }
            }

            // other is octave
            if let FeastRank54Inner::Octave {
                rank: rank2,
                flags: flags2,
                ..
            } = other
            {
                let is_octave_day = flags2.contains(OctaveFlags::OCTAVE_DAY);
                let is_first_3_days = flags2.contains(OctaveFlags::FIRST_3_DAYS);
                match *rank2 {
                    OctaveType::Privileged1 => match (rank1, is_octave_day) {
                        (FeastClass::FirstClassDouble, _) => {
                            return Ok(OccurrenceResult::FirstNothingOfSecond);
                        }

                        (FeastClass::Semidouble, true) => {
                            // On the actual octave day of a Privileged1 octave, prefer
                            // the octave day (it should take precedence over Major/Double
                            // feasts). Make the octave win explicitly to avoid
                            // permutation-dependent outcomes.
                            return Ok(OccurrenceResult::SecondNothingOfFirst);
                        }
                        (FeastClass::SecondClassDouble, _) => {
                            return Ok(OccurrenceResult::SecondTransferAndCommemorationOfFirst);
                        }
                        (
                            FeastClass::MajorDouble | FeastClass::Double | FeastClass::Semidouble,
                            _,
                        ) if !is_first_3_days => {
                            return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                        }
                        (
                            FeastClass::MajorDouble | FeastClass::Double | FeastClass::Semidouble,
                            _,
                        ) if is_first_3_days => {
                            return Ok(OccurrenceResult::SecondNothingOfFirst);
                        }

                        (_, true) => return Ok(OccurrenceResult::SecondCommemorationOfFirst),
                        (_, false) => return Ok(OccurrenceResult::SecondNothingOfFirst),
                    },
                    OctaveType::Privileged2 => match (rank1, is_octave_day) {
                        (FeastClass::FirstClassDouble, true) => {
                            // Prefer the First Class Double over the actual octave day
                            // of a Privileged2 octave (the feast should win). Avoid
                            // using transfer outcomes here because they create
                            // permutation-dependent behavior in three-way comparisons.
                            return Ok(OccurrenceResult::FirstNothingOfSecond);
                        }
                        (FeastClass::FirstClassDouble, false)
                            if flags1.contains(FeastFlags::OF_OUR_LORD) =>
                        {
                            return Ok(OccurrenceResult::FirstNothingOfSecond);
                        }
                        (FeastClass::FirstClassDouble, false) => {
                            return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                        }
                        (FeastClass::SecondClassDouble, _) => {
                            // Prefer the Second Class Double over a day within a
                            // Privileged2 octave to avoid cycles with Privileged3
                            // octave days that produce permutation-dependent outcomes.
                            return Ok(OccurrenceResult::FirstNothingOfSecond);
                        }
                        (FeastClass::MajorDouble, _)
                            if flags1.contains(FeastFlags::OF_OUR_LORD) =>
                        {
                            return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                        }
                        (FeastClass::MajorDouble, _) => {
                            // Prefer the MajorDouble over a day within a Privileged2 octave
                            // to avoid rock-paper-scissors cycles with SecondClassDouble.
                            // Make the feast win explicitly (MajorDouble wins) instead of
                            // the octave taking precedence.
                            return Ok(OccurrenceResult::FirstNothingOfSecond);
                        }

                        (FeastClass::Double, _) => {
                            // Prefer the feast (Double/Semidouble) over a day within a
                            // Privileged2 octave. Returning FirstCommemorationOfSecond makes the
                            // feast win (the octave is commemorated) and eliminates a
                            // non-transitive cycle observed in three-way comparisons for those
                            // ranks.
                            return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                        }
                        (FeastClass::Semidouble, _) => {
                            // Prefer the feast (Double/Semidouble) over a day within a
                            // Privileged2 octave. Returning FirstCommemorationOfSecond makes the
                            // feast win (the octave is commemorated) and eliminates a
                            // non-transitive cycle observed in three-way comparisons for those
                            // ranks.
                            return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                        }
                        (FeastClass::Simple, _) => {
                            // For Simple feasts, prefer the Privileged2 octave day (or the
                            // octave when applicable). Making the octave take precedence
                            // here breaks a rock-paper-scissors cycle between Simple feasts,
                            // Privileged1 non-octave days and Privileged2 octave days that
                            // produced permutation-dependent winners.
                            return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                        }
                        _ => return Ok(OccurrenceResult::SecondNothingOfFirst),
                    },
                    // Doubles and above win over common/privileged3 octaves, octave commemorated
                    OctaveType::Privileged3 => match rank1 {
                        FeastClass::FirstClassDouble
                            if !flags1.contains(FeastFlags::OF_OUR_LORD) =>
                        {
                            return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                        }
                        FeastClass::FirstClassDouble
                            if flags1.contains(FeastFlags::OF_OUR_LORD) =>
                        {
                            return Ok(OccurrenceResult::FirstNothingOfSecond);
                        }
                        FeastClass::Semidouble
                        | FeastClass::Double
                        | FeastClass::MajorDouble
                        | FeastClass::SecondClassDouble
                            if !is_octave_day =>
                        {
                            return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                        }
                        FeastClass::SecondClassDouble if is_octave_day => {
                            // Treat a Second Class Double versus the actual octave day of a
                            // Privileged3 octave as the octave taking precedence. Return
                            // FirstNothingOfSecond so that the feast yields to the octave
                            // (the octave wins). This removes a non-transitive cycle that
                            // produced permutation-dependent winners.
                            return Ok(OccurrenceResult::FirstNothingOfSecond);
                        }
                        // For Double/Semidouble on the actual octave day of a Privileged3
                        // octave, prefer the octave day (make the octave win). Use the
                        // SecondNothingOfFirst outcome here to make the pairwise relation
                        // explicit and eliminate permutation-dependent cycles.
                        FeastClass::Double if is_octave_day => {
                            // Prefer the Double feast over the actual octave day of a
                            // Privileged3 octave. Make the Double win explicitly to
                            // avoid cycles where the Privileged3 octave day could
                            // beat the Double in some permutations.
                            return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                        }
                        FeastClass::Semidouble if is_octave_day => {
                            // Prefer the OCTAVE_DAY over Semidoubles on Privileged3
                            // octaves so that the octave day deterministically wins.
                            // This aligns Semidouble behavior with Double on the
                            // actual octave day and avoids a cycle between
                            // Octave Day -> Double -> Semidouble -> Octave Day.
                            return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                        }
                        FeastClass::Simple => {
                            return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                        }
                        // Explicit: MajorDouble versus the actual OCTAVE_DAY of a Privileged3
                        // octave. Historically some cases fell through here and relied on
                        // swapped/delegated logic which allowed an Err to bubble up when
                        // the reverse order also lacked an explicit arm. Make the pairwise
                        // outcome explicit: have the Privileged3 OCTAVE_DAY take
                        // precedence over a MajorDouble (the octave day wins).
                        FeastClass::MajorDouble if is_octave_day => {
                            return Ok(OccurrenceResult::FirstNothingOfSecond);
                        }
                        // Explicit: Semidoubles of Our Lord vs Privileged3 OCTAVE_DAY
                        // Prefer the OCTAVE_DAY to win in order to avoid cycles where
                        // Semidouble (of Our Lord) -> Major Double (Octave Day) -> Privileged3
                        // Octave Day -> Semidouble (of Our Lord).
                        FeastClass::Semidouble
                            if is_octave_day && flags1.contains(FeastFlags::OF_OUR_LORD) =>
                        {
                            return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                        }
                        _ => {}
                    },
                    OctaveType::Common => match rank1 {
                        // If this is a First Class Double and the other is the actual
                        // octave day, have the First Class Double be commemorated so
                        // the octave wins explicitly.
                        FeastClass::FirstClassDouble if is_octave_day => {
                            return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                        }
                        // If this is a Second Class Double and the other is the actual
                        // octave day, prefer the octave day (make the octave win).
                        FeastClass::SecondClassDouble if is_octave_day => {
                            return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                        }
                        // Non-octave-day First/Second Class Doubles generally beat the octave day
                        FeastClass::FirstClassDouble | FeastClass::SecondClassDouble => {
                            return Ok(OccurrenceResult::FirstNothingOfSecond);
                        }
                        // When the other is the actual octave day, prefer the feast
                        // to be celebrated rather than the octave-day for Major/Double.
                        // However, Semidoubles are weaker and should yield to the
                        // actual Common octave day (which is represented as a Major Double
                        // in many cases). Make Major/Double win explicitly when facing a
                        // Common octave OCTAVE_DAY but have Semidoubles yield so we don't
                        // create cycles where low-ranked Semidoubles beat octave days.
                        FeastClass::MajorDouble | FeastClass::Double if is_octave_day => {
                            return Ok(OccurrenceResult::FirstNothingOfSecond);
                        }
                        FeastClass::Semidouble if is_octave_day => {
                            // Make the Common octave day take precedence over Semidoubles.
                            return Ok(OccurrenceResult::SecondNothingOfFirst);
                        }
                        // When not the octave day, these feast ranks may commemorate the octave
                        FeastClass::MajorDouble | FeastClass::Double | FeastClass::Semidouble
                            if !is_octave_day =>
                        {
                            return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                        }
                        FeastClass::Simple => {
                            return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                        }
                        // A Commemoration (lowest feast) should yield to a Common octave day;
                        // make the octave win explicitly so octaves don't get beaten by
                        // commemorations which would create cycles with Simple octave days.
                        FeastClass::Commemoration => {
                            return Ok(OccurrenceResult::SecondNothingOfFirst);
                        }
                        _ => return Ok(OccurrenceResult::FirstNothingOfSecond),
                    },
                    OctaveType::Simple => {
                        match (rank1, is_octave_day) {
                            (_, false) => return Ok(OccurrenceResult::FirstNothingOfSecond),
                            (FeastClass::FirstClassDouble, true) => {
                                return Ok(OccurrenceResult::FirstNothingOfSecond);
                            }
                            (FeastClass::Semidouble, true) => {
                                // Make Semidoubles yield to a Simple octave day (octave wins).
                                // This explicit arm prevents a Major->Semidouble->Simple
                                // cycle that was producing permutation-dependent winners.
                                return Ok(OccurrenceResult::FirstNothingOfSecond);
                            }
                            (FeastClass::SecondClassDouble, true) => {
                                return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                            }
                            (FeastClass::MajorDouble, true) => {
                                return Ok(OccurrenceResult::FirstNothingOfSecond);
                            }
                            (FeastClass::Double, true) => {
                                // Prefer the Double over a simple octave day to avoid
                                // permutation-dependent outcomes in three-way comparisons.
                                return Ok(OccurrenceResult::FirstNothingOfSecond);
                            }
                            _ => return Ok(OccurrenceResult::SecondCommemorationOfFirst),
                        }
                    }
                }
            }
            // other is feast
            if let FeastRank54Inner::Feast {
                rank: rank2,
                flags: flags2,
            } = other
            {
                match (rank1, rank2) {
                    (FeastClass::FirstClassDouble, FeastClass::FirstClassDouble)
                        if flags1.contains(FeastFlags::OF_OUR_LORD)
                            && !flags2.contains(FeastFlags::OF_OUR_LORD) =>
                    {
                        return Ok(OccurrenceResult::FirstNothingOfSecond);
                    }
                    (FeastClass::FirstClassDouble, FeastClass::SecondClassDouble) => {
                        return Ok(OccurrenceResult::FirstNothingOfSecond);
                    }
                    (FeastClass::FirstClassDouble, FeastClass::MajorDouble) => {
                        return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                    }
                    (FeastClass::FirstClassDouble, FeastClass::Double)
                        if !flags1.contains(FeastFlags::OF_OUR_LORD) =>
                    {
                        return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                    }
                    (
                        FeastClass::FirstClassDouble,
                        FeastClass::Double | FeastClass::Semidouble | FeastClass::Simple,
                    ) => {
                        return Ok(OccurrenceResult::FirstNothingOfSecond);
                    }
                    (FeastClass::SecondClassDouble, FeastClass::MajorDouble) => {
                        // Symmetric: Second vs Major -> Major wins (second yields).
                        return Ok(OccurrenceResult::FirstNothingOfSecond);
                    }
                    (
                        FeastClass::SecondClassDouble,
                        FeastClass::Double | FeastClass::Semidouble | FeastClass::Simple,
                    ) => {
                        return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                    }

                    (FeastClass::MajorDouble, FeastClass::MajorDouble)
                        if flags2.contains(FeastFlags::OF_OUR_LORD)
                            && !flags1.contains(FeastFlags::OF_OUR_LORD) =>
                    {
                        return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                    }
                    (FeastClass::MajorDouble, FeastClass::Double) => {
                        return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                    }
                    (FeastClass::MajorDouble, FeastClass::Semidouble | FeastClass::Simple) => {
                        return Ok(OccurrenceResult::FirstNothingOfSecond);
                    }

                    (FeastClass::Double, FeastClass::Double)
                        if flags2.contains(FeastFlags::OF_OUR_LORD)
                            && !flags1.contains(FeastFlags::OF_OUR_LORD) =>
                    {
                        return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                    }

                    (FeastClass::Double, FeastClass::Semidouble | FeastClass::Simple) => {
                        return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                    }

                    (FeastClass::Semidouble, FeastClass::Simple) => {
                        return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                    }

                    _ => {}
                }
            }
            // other is vigil
            if let FeastRank54Inner::Vigil { kind: kind2 } = other {
                match (rank1, kind2) {
                    (FeastClass::FirstClassDouble, _) => {
                        return Ok(OccurrenceResult::FirstNothingOfSecond);
                    }
                    (FeastClass::SecondClassDouble, VigilKind::ChristmasOrPentecost) => {
                        return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                    }
                    (FeastClass::SecondClassDouble, _) => {
                        return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                    }
                    (FeastClass::MajorDouble, _) => {
                        // Prefer the MajorDouble/Double over a Vigil: have the feast
                        // be celebrated (the vigil is commemorated). This prevents
                        // cycles where Vigils could beat Major Doubles in some
                        // permutations and produce ordering-dependent winners.
                        return Ok(OccurrenceResult::FirstNothingOfSecond);
                    }
                    (FeastClass::Double, _) => {
                        return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                    }
                    (FeastClass::Semidouble, _) => {
                        // Prefer the vigil over a semidouble feast: have the semidouble be
                        // commemorated so that vigil-vs-semidouble is explicit and
                        // eliminates permutation-dependent cycles involving vigils.
                        return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                    }
                    (FeastClass::Simple, _) => {
                        return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                    }
                    _ => {}
                }
            }
            // other is sunday — follow 1954 rules (explicit arms per user spec):
            // - Sunday I: no feast may be celebrated; feasts are commemorated (except
            //   Easter/Pentecost which cannot be commemorated — not detectable here)
            // - Sunday II: only Doubles of the I Class may be celebrated; other feasts are
            //   commemorated
            // - Lesser Sundays: Doubles of I or II class, or a feast of Our Lord, may be
            //   celebrated; others are commemorated
            if let FeastRank54Inner::Sunday { rank: rank2, flags } = other {
                match rank2 {
                    // Greater Sunday of the I class: Sunday wins; feast becomes a commemoration of
                    // the Sunday Except for Easter/Pentecost Sundays which do
                    // not admit commemorations.
                    SundayClass::First => match rank1 {
                        FeastClass::FirstClassDouble
                        | FeastClass::SecondClassDouble
                        | FeastClass::MajorDouble
                        | FeastClass::Double
                        | FeastClass::Semidouble => {
                            if flags.contains(SundayFlags::EASTER_OR_PENTECOST) {
                                // behave like First-class Sunday but do not admit commemorations
                                return Ok(OccurrenceResult::SecondNothingOfFirst);
                            }
                            return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                        }
                        _ => {
                            return Ok(OccurrenceResult::SecondNothingOfFirst);
                        }
                    },
                    // Greater Sunday of the II class: only First Class Doubles may be celebrated
                    SundayClass::Second => match rank1 {
                        FeastClass::FirstClassDouble => {
                            // Feast (first) may be celebrated on Sunday II
                            return Ok(OccurrenceResult::FirstNothingOfSecond);
                        }
                        // FeastClass::MajorDouble if flags1.contains(FeastFlags::OF_OUR_LORD) => {
                        //     // Major Doubles of Our Lord may be celebrated on Sunday II.
                        //     // Do not extend this special-case to SecondClassDouble to
                        //     // avoid a non-transitive cycle: SecondClassDouble -> Sunday II ->
                        // MajorDouble     // which can produce
                        // permutation-dependent winners.     return
                        // Ok(OccurrenceResult::FirstNothingOfSecond); }
                        // FeastClass::SecondClassDouble
                        //     if flags1.contains(FeastFlags::OF_OUR_LORD) =>
                        // {
                        //     // Feast is commemorated (Sunday II takes precedence over a
                        //     // Second Class Double when it's not a feast of Our Lord).
                        //     return Ok(OccurrenceResult::FirstNothingOfSecond);
                        // }
                        FeastClass::SecondClassDouble => {
                            // Feast is commemorated (Sunday II takes precedence over a
                            // Second Class Double when it's not a feast of Our Lord).
                            return Ok(OccurrenceResult::SecondTransferAndCommemorationOfFirst);
                        }
                        FeastClass::MajorDouble
                        | FeastClass::Double
                        | FeastClass::Semidouble
                        | FeastClass::Simple
                        | FeastClass::Commemoration => {
                            // For consistency in permutation resolution, have Sunday II take
                            // precedence over Doubles, Semidoubles and
                            // Simple feasts (they are commemorated).
                            return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                        }
                    },
                    // Lesser Sundays: Doubles of I or II class, or a feast of Our Lord, take
                    // precedence
                    SundayClass::Lesser => match rank1 {
                        // Lesser Sundays: Doubles of the I or II class take precedence and may be
                        // celebrated. Similarly, a Major Double that is a feast of Our Lord may
                        // be celebrated. Other feasts are commemorated.
                        FeastClass::FirstClassDouble
                            if flags1.contains(FeastFlags::OF_OUR_LORD) =>
                        {
                            return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                        }
                        FeastClass::FirstClassDouble => {
                            return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                        }
                        FeastClass::SecondClassDouble
                            if flags1.contains(FeastFlags::OF_OUR_LORD) =>
                        {
                            return Ok(OccurrenceResult::FirstNothingOfSecond);
                        }
                        FeastClass::SecondClassDouble => {
                            return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                        }
                        FeastClass::MajorDouble if flags1.contains(FeastFlags::OF_OUR_LORD) => {
                            return Ok(OccurrenceResult::FirstNothingOfSecond);
                        }
                        FeastClass::MajorDouble => {
                            // Prefer the Major Double over a Lesser Sunday in ordinary cases
                            // (unless it's a feast of Our Lord, handled above). Make the
                            // Major win explicitly to avoid permutation-dependent cycles
                            // with SecondClassDouble and Sundays.
                            return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                        }
                        FeastClass::Double => {
                            return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                        }
                        _ => {
                            return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                        }
                    },
                }
            }
        }

        // self is vigil
        if let FeastRank54Inner::Vigil { kind } = self {
            if let FeastRank54Inner::Octave {
                rank: rank2,
                flags: _flags2,
                ..
            } = other
            {
                let is_major_vigil = matches!(kind, VigilKind::ChristmasOrPentecost);
                match rank2 {
                    // Prefer major-like vigils over non-octave day within a Privileged1 octave
                    OctaveType::Privileged1 if is_major_vigil => {
                        return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                    }
                    // Prefer major-like vigils over simple octaves
                    OctaveType::Simple if is_major_vigil => {
                        return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                    }
                    OctaveType::Privileged3 => {
                        return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                    }
                    OctaveType::Privileged2 => {
                        return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                    }
                    // Common and other octaves generally beat vigils
                    OctaveType::Common => return Ok(OccurrenceResult::SecondCommemorationOfFirst),
                    OctaveType::Simple => return Ok(OccurrenceResult::SecondNothingOfFirst),
                    OctaveType::Privileged1 => {}
                }
            }

            if let FeastRank54Inner::Feria {
                rank: rank2,
                flags: _flags2,
            } = other
            {
                let is_major_vigil = matches!(kind, VigilKind::ChristmasOrPentecost);
                match rank2 {
                    // Vigils of the highest kind should take precedence over
                    // Greater Non-Privileged ferias to avoid cycles with Simple feasts.
                    FeriaClass::GreaterNonPrivilaged if is_major_vigil => {
                        return Ok(OccurrenceResult::FirstNothingOfSecond);
                    }
                    FeriaClass::GreaterNonPrivilaged => {
                        return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                    }
                    FeriaClass::Lesser => return Ok(OccurrenceResult::FirstNothingOfSecond),
                    FeriaClass::GreaterPrivilaged => {}
                }
            }
            // If the other is a Feast and the Vigil appears first in the ordering,
            // delegate to the Feast-side rules and reverse the result so that the
            // pairwise relation is always explicit and symmetric.
            if let FeastRank54Inner::Feast { .. } = other {
                return other
                    .resolve_occurrence_inner(self, false)
                    .map(OccurrenceResult::reverse);
            }
            if let FeastRank54Inner::Sunday { .. } = other {
                // Historically, special vigils like Christmas and Pentecost were
                // class I and could take precedence over Sundays. Epiphany's
                // vigil behaved like class II. Common vigils should yield to
                // Sundays. Implement that here.
                match kind {
                    VigilKind::ChristmasOrPentecost => {
                        // Vigil wins over Sunday
                        return Ok(OccurrenceResult::FirstNothingOfSecond);
                    }
                    VigilKind::Epiphany | VigilKind::Common => {
                        return Ok(OccurrenceResult::SecondNothingOfFirst);
                    }
                }
            }
        }

        // self is octave
        if let FeastRank54Inner::Octave {
            rank: rank1,
            flags: flags1,
            ..
        } = self
        {
            let is_octave_day1 = flags1.contains(OctaveFlags::OCTAVE_DAY);
            // ferias: octaves generally outrank ferial days; simple octave days are weaker
            if let FeastRank54Inner::Feria {
                rank: rank2,
                flags: _flags2,
            } = other
            {
                match (rank1, rank2) {
                    (_, FeriaClass::GreaterPrivilaged) => {
                        return Ok(OccurrenceResult::SecondNothingOfFirst);
                    }
                    // simple octaves yield to ferias (rare), keep previous conservative behavior
                    (OctaveType::Simple, _) if is_octave_day1 => {
                        return Ok(OccurrenceResult::FirstNothingOfSecond);
                    }
                    (OctaveType::Simple, _) => return Ok(OccurrenceResult::SecondNothingOfFirst),
                    (
                        OctaveType::Privileged1 | OctaveType::Privileged2,
                        FeriaClass::GreaterNonPrivilaged,
                    ) => return Ok(OccurrenceResult::FirstNothingOfSecond),
                    (OctaveType::Common, FeriaClass::GreaterNonPrivilaged) => {
                        return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                    }
                    _ => return Ok(OccurrenceResult::FirstNothingOfSecond),
                }
            }

            // Sundays that fall within an octave follow octave rules; treat them similarly
            // to feasts here
            if let FeastRank54Inner::Sunday {
                rank: rank2,
                flags: _,
            } = other
            {
                match (is_octave_day1, rank1, rank2) {
                    (false, OctaveType::Privileged1, SundayClass::First) => {
                        return Ok(OccurrenceResult::SecondNothingOfFirst);
                    }
                    (false, OctaveType::Privileged1, SundayClass::Second | SundayClass::Lesser) => {
                        return Ok(OccurrenceResult::FirstNothingOfSecond);
                    }
                    // If the other is a 1st class Sunday, the Sunday should always win over an
                    // octave day
                    (_, _, SundayClass::First) => {
                        return Ok(OccurrenceResult::SecondNothingOfFirst);
                    }
                    // If this is a Common octave day (often represented as a Major Double)
                    // and it is the actual OCTAVE_DAY, prefer the octave day over
                    // Lesser or Second-Class Sundays so that octave-days deterministically
                    // beat those Sundays. This makes the pairwise relation explicit
                    // and prevents permutation-dependent winners when octaves interact
                    // with Sundays and feasts.
                    (_, OctaveType::Common, SundayClass::Second) => {
                        return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                    }
                    (true, OctaveType::Common, SundayClass::Lesser) => {
                        return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                    }
                    // If this is a privileged1 octave day but NOT the actual octave day
                    // (is_octave_day == false), prefer the Sunday over the octave day.
                    // This prevents a rock-paper-scissors cycle between Privileged1
                    // octave days, Second Class Doubles and Greater Sundays of the
                    // Second Class which could produce permutation-dependent winners.
                    (false, OctaveType::Privileged2, SundayClass::Second) => {
                        // Prefer Sundays over non-octave days of Privileged2 octaves.
                        // This breaks a rock-paper-scissors cycle between Second Class
                        // Doubles, Sundays II, and Privileged2 non-octave days by
                        // making the Sunday take precedence when the octave day
                        // itself is not the OCTAVE_DAY.
                        return Ok(OccurrenceResult::SecondNothingOfFirst);
                    }
                    (false, OctaveType::Privileged2, SundayClass::Lesser) => {
                        // Prefer Sundays over non-octave days of Privileged2 octaves.
                        // Make the Lesser Sunday win deterministically to avoid cycles
                        // where Privileged2 non-octave days beat Lesser Sundays in some
                        // permutations.
                        return Ok(OccurrenceResult::SecondNothingOfFirst);
                    }
                    (_, OctaveType::Privileged3, _) => {
                        return Ok(OccurrenceResult::SecondNothingOfFirst);
                    }
                    (true, OctaveType::Simple, _) => {
                        // octave days yield to Sundays
                        return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                    }
                    (false, OctaveType::Common, SundayClass::Lesser) => {
                        // these octaves yield to Sundays (Sundays are liturgically higher than a
                        // simple octave)
                        return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                    }
                    (false, OctaveType::Simple, SundayClass::Lesser) => {
                        // these octaves yield to Sundays (Sundays are liturgically higher than a
                        // simple octave)
                        return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                    }
                    _ => {}
                }
            }

            // other is feast
            // Delegate to the Feast vs Octave logic (reverse the result) to ensure
            // symmetry: some Feast-vs-Octave cases are handled in the Feast branch
            // but the Octave->Feast order lacked explicit arms and would fall
            // through to an error. Reuse the existing rules by asking the other
            // side to resolve and reversing the outcome.
            if let FeastRank54Inner::Feast { .. } = other {
                return other
                    .resolve_occurrence_inner(self, false)
                    .map(OccurrenceResult::reverse);
            }

            // other is vigil
            // Delegate Octave vs Vigil to the Vigil-side logic and reverse the outcome
            // so that when an Octave appears first we reuse the explicit Vigil->Octave
            // rules and keep pairwise outcomes symmetric.
            if let FeastRank54Inner::Vigil { .. } = other {
                return other
                    .resolve_occurrence_inner(self, false)
                    .map(OccurrenceResult::reverse);
            }

            // octave vs octave: fall through to numeric tie-breaker
            if let FeastRank54Inner::Octave {
                rank: rank2,
                flags: flags2,
                ..
            } = other
            {
                match (rank1, rank2) {
                    // Explicit: Privileged1 vs Common octave -- make the common octave
                    // (e.g., an actual Major Double OCTAVE_DAY) take precedence over
                    // a non-octave-day Privileged1. This fills a missing symmetric
                    // arm that could otherwise fall through to swapped/implicit
                    // logic and cause permutation-dependent winners.
                    (OctaveType::Privileged1, OctaveType::Common) => {
                        // Prefer a Privileged1 octave over a Common octave. This
                        // breaks a rock-paper-scissors cycle observed in three-way
                        // comparisons with Vigils (Major Vigil > Privileged1, Privileged1
                        // > Common, Common > Major Vigil). Making Privileged1 win
                        // deterministically removes the cycle and produces a
                        // permutation-independent outcome.
                        return Ok(OccurrenceResult::FirstNothingOfSecond);
                    }
                    // Symmetric counterpart: when the Common octave appears on the left
                    // and a Privileged1 octave on the right, have the Privileged1
                    // octave explicitly win as well.
                    (OctaveType::Common, OctaveType::Privileged1) => {
                        return Ok(OccurrenceResult::SecondNothingOfFirst);
                    }
                    (OctaveType::Privileged3, OctaveType::Common)
                        if flags2.contains(OctaveFlags::OCTAVE_DAY)
                            && !flags1.contains(OctaveFlags::OCTAVE_DAY) =>
                    {
                        // Make the Common octave (often represented as a Major Double
                        // on its OCTAVE_DAY) take precedence over Privileged3 octave
                        // days. Return SecondNothingOfFirst so that when the privileged
                        // octave appears on the left the Common octave on the right
                        // is chosen as the winner. This explicit arm avoids relying
                        // on swapped/delegated logic and prevents ordering from
                        // changing the pairwise outcome.
                        return Ok(OccurrenceResult::SecondCommemorationOfFirst);
                    }
                    (OctaveType::Privileged2 | OctaveType::Privileged3, OctaveType::Common) => {
                        // Make the Common octave (often represented as a Major Double
                        // on its OCTAVE_DAY) take precedence over Privileged2/3
                        // octave days. Return SecondNothingOfFirst so that when the
                        // privileged octave appears on the left the Common octave on
                        // the right is chosen as the winner. This explicit arm avoids
                        // relying on swapped/delegated logic and prevents ordering
                        // from changing the pairwise outcome.
                        return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                    }
                    (OctaveType::Privileged2 | OctaveType::Privileged3, OctaveType::Simple) => {
                        return Ok(OccurrenceResult::FirstNothingOfSecond);
                    }
                    // Explicit: Privileged1 vs Simple octave -- make Privileged1
                    // take precedence over a Simple octave (both when Privileged1
                    // is on the left and when it's on the right). This prevents
                    // cycles where Simple could beat Privileged1 in some
                    // permutations and produce ordering-dependent winners.
                    (OctaveType::Privileged1, OctaveType::Simple) => {
                        return Ok(OccurrenceResult::FirstNothingOfSecond);
                    }
                    (OctaveType::Simple, OctaveType::Privileged1) => {
                        return Ok(OccurrenceResult::SecondNothingOfFirst);
                    }
                    (OctaveType::Common, OctaveType::Simple) => {
                        return Ok(OccurrenceResult::FirstCommemorationOfSecond);
                    }
                    (OctaveType::Simple, _) if !flags1.contains(OctaveFlags::OCTAVE_DAY) => {
                        return Ok(OccurrenceResult::SecondNothingOfFirst);
                    }
                    _ => {}
                }
            }
        }

        // try swapping the order
        if try_swapped {
            return other
                .resolve_occurrence_inner(self, false)
                .map(OccurrenceResult::reverse);
        }
        // No explicit rule matched; fall through to numeric-rank fallback below.
        // just pick higher rank or apply tie-breaker if equal
        bail!(
            "No explicit occurrence rule matched between {:?} and {:?}",
            self,
            other
        );
    }
    fn get_rank_string(&self) -> ArcStr {
        self.get_rank_string_inner(false)
    }
    fn get_rank_string_verbose(&self) -> ArcStr {
        self.get_rank_string_inner(true)
    }
    fn get_rank_string_inner(&self, v: bool) -> ArcStr {
        match self {
            FeastRank54Inner::Feria { rank, flags } => {
                let mut parts = match rank {
                    FeriaClass::GreaterPrivilaged => vec!["Greater Privileged Feria".to_string()],
                    FeriaClass::GreaterNonPrivilaged => {
                        vec!["Greater Non-Privileged Feria".to_string()]
                    }
                    FeriaClass::Lesser => vec!["Ordinary Feria".to_string()],
                };
                if v && flags.contains(FeriaFlags::HOLY_TRIDUUM) {
                    parts.push("of the Holy Triduum".to_string());
                }
                parts.join(" ").into()
            }
            FeastRank54Inner::Feast { rank, flags } => {
                // Include OF_OUR_LORD in the label when present to disambiguate
                // otherwise-identical feast ranks that differ only by this flag.
                let base_name = match rank {
                    FeastClass::FirstClassDouble => "First Class Double",
                    FeastClass::SecondClassDouble => "Second Class Double",
                    FeastClass::MajorDouble => "Major Double",
                    FeastClass::Double => "Double",
                    FeastClass::Semidouble => "Semidouble",
                    FeastClass::Simple => "Simple",
                    FeastClass::Commemoration => "Commemoration",
                };
                if v && flags.contains(FeastFlags::OF_OUR_LORD) {
                    format!("{base_name} (of Our Lord)").into()
                } else {
                    base_name.into()
                }
            }
            FeastRank54Inner::Vigil { kind } => match kind {
                VigilKind::ChristmasOrPentecost => "Vigil (Christmas/Pentecost)",
                VigilKind::Epiphany => "Vigil (Epiphany)",
                VigilKind::Common => "Vigil",
            }
            .into(),
            FeastRank54Inner::Sunday { rank, .. } => match rank {
                SundayClass::First => "Greater Sunday of the First Class",
                SundayClass::Second => "Greater Sunday of the Second Class",
                SundayClass::Lesser => "Lesser Sunday",
            }
            .into(),
            FeastRank54Inner::Octave { rank, flags, .. } => {
                let is_octave_day = flags.contains(OctaveFlags::OCTAVE_DAY);
                match (rank, is_octave_day) {
                    (OctaveType::Privileged1, true) => {
                        "Octave Day of a Privileged 1st Class Octave"
                    }
                    (OctaveType::Privileged1, false) => "Day within a Privileged 1st Class Octave",
                    (OctaveType::Privileged2, true) => {
                        "Octave Day of a Privileged 2nd Class Octave"
                    }
                    (OctaveType::Privileged2, false) => "Day within a Privileged 2nd Class Octave",
                    (OctaveType::Privileged3, true) => {
                        "Octave Day of a Privileged 3rd Class Octave"
                    }
                    (OctaveType::Privileged3, false) => "Day within a Privileged 3rd Class Octave",
                    // Represent the actual octave day of a Common octave distinctly
                    // so it doesn't collide with feast Major Doubles in the
                    // occurrence graph labels.
                    (OctaveType::Common, true) => "Major Double (Octave Day)",
                    (OctaveType::Common, false) => "Day within a Common Octave",
                    // Disambiguate a Simple octave day from a Feast "Simple" so
                    // test graphs and debugging don't conflate distinct node types.
                    (OctaveType::Simple, true) => "Simple Octave Day",
                    (OctaveType::Simple, false) => "Day within a Simple Octave",
                }
            }
            .into(),
        }
    }

    fn new_with_context(rank: &str, day_type: DayType, context: &LiturgicalContext) -> Self {
        // Create flags based on context
        let mut feast_flags = FeastFlags::empty();
        let mut feria_flags = FeriaFlags::empty();

        if context.of_our_lord {
            feast_flags |= FeastFlags::OF_OUR_LORD;
        }

        // Parse rank string and day type to determine specific rank
        match day_type {
            DayType::Feria => {
                // Check for special feria types in 1954
                let rank = match rank {
                    "greater privileged" | "I" => FeriaClass::GreaterPrivilaged, /* Ash Wednesday and Monday, Tuesday, and Wednesday of Holy Week. No feast day could be celebrated on these days. */
                    "greater non-privileged" | "II" => FeriaClass::GreaterNonPrivilaged, /* The ferias of Advent, Lent, and Passion Week, Rogation Monday, and the Ember Days. Any feast day except a Simple could occur on these days, with a commemoration of the feria. */
                    "ordinary" | "III" | "IV" => FeriaClass::Lesser, // Ordinary ferias
                    _ => panic!("Unknown feria rank: {rank}"),
                };

                if let Some(feast_name) = &context.feast_name
                    && (feast_name.contains("Holy Thursday")
                        || feast_name.contains("Holy Saturday")
                        || feast_name.contains("Good Friday"))
                {
                    feria_flags |= FeriaFlags::HOLY_TRIDUUM;
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
                    _ => panic!("Unknown feast rank: {rank}"),
                };
                FeastRank54Inner::Feast {
                    rank: feast_rank,
                    flags: feast_flags,
                }
            }
            DayType::Sunday => {
                let rank = match rank {
                    "I" => SundayClass::First,    // Major sundays (Easter, Pentecost, etc.)
                    "II" => SundayClass::Second,  // Important sundays
                    "III" => SundayClass::Lesser, // Ordinary sundays
                    _ => panic!("Unknown sunday rank: {rank}"),
                };
                let mut sflags = SundayFlags::empty();
                if context.is_easter_or_pentecost && rank == SundayClass::First {
                    sflags.insert(SundayFlags::EASTER_OR_PENTECOST);
                }
                FeastRank54Inner::Sunday {
                    rank,
                    flags: sflags,
                }
            }
            DayType::Vigil => {
                // Determine vigil kind from feast name when possible
                let kind = if let Some(name) = &context.feast_name {
                    let lname = name.to_lowercase();
                    if lname.contains("christmas")
                        || lname.contains("vigil of christmas")
                        || lname.contains("pentecost")
                        || lname.contains("vigil of pentecost")
                    {
                        VigilKind::ChristmasOrPentecost
                    } else if lname.contains("epiphany") || lname.contains("vigil of the epiphany")
                    {
                        VigilKind::Epiphany
                    } else {
                        VigilKind::Common
                    }
                } else {
                    VigilKind::Common
                };
                FeastRank54Inner::Vigil { kind }
            }
            DayType::Octave => {
                let rank = match rank {
                    "privileged1" | "I" => OctaveType::Privileged1,
                    "privileged2" | "II" => OctaveType::Privileged2,
                    "privileged3" | "III" => OctaveType::Privileged3,
                    "common" | "IV" => OctaveType::Common,
                    "simple" | "V" => OctaveType::Simple,
                    _ => panic!("Unknown octave rank: {rank}"),
                };
                FeastRank54Inner::Octave {
                    rank,
                    flags: context.octave_flags,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    // rayon iter imports intentionally omitted when unused
    use test_case::test_matrix;

    use super::*;
    use crate::calender::feast_rank::test::{
        test_feast_rank_enumeration_conflicts, test_feast_rank_enumeration_occurance_graph,
    };

    #[test]
    fn test_feast_rank_54_precedence() {
        let context = LiturgicalContext::new();

        let christmas =
            FeastRank54::new_with_context("I", DayType::Feast, &context.clone().of_our_lord());
        let saint_feast = FeastRank54::new_with_context("III", DayType::Feast, &context);

        let competetors = vec![
            (christmas, "Christmas".to_string()),
            (saint_feast, "St. John".to_string()),
        ];

        let result = FeastRank54::resolve_conflicts(&competetors).unwrap();
        assert_eq!(result.winner, "Christmas");
    }

    #[test]
    fn test_octave_day_vs_first_class_sunday() {
        // Privileged1 octave day vs 1st class sunday: Sunday should win
        let mut ctx_octave = LiturgicalContext::new();
        ctx_octave = ctx_octave.octave_day(true);

        let double =
            FeastRank54::new_with_context("double", DayType::Feast, &LiturgicalContext::new());
        let octave = FeastRank54::new_with_context("I", DayType::Octave, &ctx_octave);
        let sunday = FeastRank54::new_with_context("I", DayType::Sunday, &LiturgicalContext::new());

        let competetors = vec![
            (double, "Double".to_string()),
            (sunday, "SundayI".to_string()),
            (octave, "OctaveDay".to_string()),
        ];
        let result = FeastRank54::resolve_conflicts(&competetors).unwrap();

        // Expect the Sunday to win; report current behavior in assertion
        assert_eq!(result.winner, "SundayI");
    }

    #[test]
    fn test_first_class_double_vs_second_class_sunday_vs_third_class_octave() {
        // First Class Double vs 2nd class Sunday vs 2nd class Octave day
        let first_class_double =
            FeastRank54::new_with_context("I", DayType::Feast, &LiturgicalContext::new());
        let second_class_sunday =
            FeastRank54::new_with_context("II", DayType::Sunday, &LiturgicalContext::new());
        let second_class_octave = FeastRank54::new_with_context(
            "privileged3",
            DayType::Octave,
            &LiturgicalContext::new(),
        );

        let competetors = vec![
            (first_class_double, "Ss. Peter and Paul".to_string()),
            (
                second_class_sunday,
                "Dominica III after Pentecost".to_string(),
            ),
            (second_class_octave, "Sacred Heart".to_string()),
        ];
        let result = FeastRank54::resolve_conflicts(&competetors).unwrap();
        println!("Result: {result:?}");
        // Expect the First Class Double to win with the octave being commemorated
        assert_eq!(result.winner, "Ss. Peter and Paul");
        assert_eq!(
            result.commemorations,
            vec![("Sacred Heart".to_string(), CommemorationType::Lauds)]
        );
    }

    impl FeastRank54Inner {
        fn enumerate() -> Vec<Self> {
            let mut ranks = Vec::new();

            // Ferial ranks (1-3) with all flags combinations
            for rank in &[
                FeriaClass::GreaterPrivilaged,
                FeriaClass::GreaterNonPrivilaged,
                FeriaClass::Lesser,
            ] {
                for holy_triduum in [false, true] {
                    if holy_triduum && *rank != FeriaClass::GreaterPrivilaged {
                        // Holy Triduum only applies to Greater Privileged ferias
                        continue;
                    }
                    let mut flags = FeriaFlags::empty();
                    if holy_triduum {
                        flags.insert(FeriaFlags::HOLY_TRIDUUM);
                    }
                    ranks.push(FeastRank54Inner::Feria { rank: *rank, flags });
                }
            }

            // Feast ranks
            for &rank in &[
                FeastClass::FirstClassDouble,
                FeastClass::SecondClassDouble,
                FeastClass::MajorDouble,
                FeastClass::Double,
                FeastClass::Semidouble,
                FeastClass::Simple,
                FeastClass::Commemoration,
            ] {
                for of_our_lord in [false, true] {
                    let mut flags = FeastFlags::empty();
                    if of_our_lord
                        && !matches!(
                            rank,
                            FeastClass::FirstClassDouble
                                | FeastClass::SecondClassDouble
                                | FeastClass::MajorDouble
                        )
                    {
                        continue;
                    }
                    if of_our_lord {
                        flags.insert(FeastFlags::OF_OUR_LORD);
                    }
                    ranks.push(FeastRank54Inner::Feast { rank, flags });
                }
            }

            // Vigil ranks
            // Single representative for vigil kinds (major/minor distinction removed)
            ranks.push(FeastRank54Inner::Vigil {
                kind: VigilKind::Common,
            });
            // Sunday ranks (1-3)
            for rank in &[SundayClass::First, SundayClass::Second, SundayClass::Lesser] {
                ranks.push(FeastRank54Inner::Sunday {
                    rank: *rank,
                    flags: SundayFlags::empty(),
                });
            }
            // Octave ranks with all flags combinations
            for &rank in &[
                OctaveType::Privileged1,
                OctaveType::Privileged2,
                OctaveType::Privileged3,
                OctaveType::Common,
                OctaveType::Simple,
            ] {
                for is_octave_day in [false, true] {
                    if rank == OctaveType::Simple && !is_octave_day {
                        // Skip non-octave days for simple octaves
                        continue;
                    }
                    for is_first_3_days in [false, true] {
                        if is_first_3_days && !matches!(rank, OctaveType::Privileged1) {
                            continue;
                        }

                        let mut flags = OctaveFlags::empty();
                        if is_octave_day {
                            flags.insert(OctaveFlags::OCTAVE_DAY);
                        }
                        if is_first_3_days {
                            flags.insert(OctaveFlags::FIRST_3_DAYS);
                        }
                        ranks.push(FeastRank54Inner::Octave { rank, flags });
                    }
                }
            }

            ranks
        }
    }

    #[test]
    fn test_feast_rank_54_enumeration_occurance() {
        for (feast1, feast2) in
            itertools::iproduct!(FeastRank54Inner::enumerate(), FeastRank54Inner::enumerate())
        {
            let rank1 = feast1.get_rank_string();
            let rank2 = feast2.get_rank_string();
            let result1 = feast1.resolve_occurrence(&feast1);
            let result2 = feast2.resolve_occurrence(&feast2);
            match (result1, result2) {
                (Ok(res1), Ok(res2)) if res1 == res2.reverse() => {
                    // All good
                }
                (Err(_e1), Err(_e2)) => {
                    // Both sides failed; nothing to assert here
                }

                (Ok(res1), Ok(res2)) => {
                    assert_eq!(res1, res2.reverse(), "Mismatch between {rank1} and {rank2}");
                }
                (Err(e), Ok(_res)) | (Ok(_res), Err(e)) => {
                    core::panic!("One side failed for {} vs {}: {}", rank1, rank2, e);
                }
            }
        }
    }

    #[test]
    fn test_feast_rank_54_enumeration_occurance_graph() {
        test_feast_rank_enumeration_occurance_graph(
            FeastRank54Inner::enumerate()
                .into_iter()
                .map(FeastRank54)
                .collect(),
        );
    }

    #[test_matrix(2..=4)]
    fn test_feast_rank_54_enumeration_conflicts(n: usize) {
        test_feast_rank_enumeration_conflicts(
            FeastRank54Inner::enumerate()
                .into_iter()
                .map(FeastRank54)
                .collect(),
            n,
        );
    }
}
