#![cfg(test)]

use std::collections::{HashMap, HashSet};

use itertools::Itertools as _;
use rayon::prelude::*;

use crate::calender::feast_rank::FeastRankResolver;

pub fn test_feast_rank_enumeration_conflicts<FR: Sized + FeastRankResolver + Send + PartialEq>(
    enumeration: Vec<FR>,
    n: usize,
) {
    println!("{} Variants", enumeration.len());
    let cs: Vec<_> = enumeration.into_iter().combinations(n).collect();
    println!("{} Combinations of {}", cs.len(), n);
    println!();
    cs.into_par_iter().enumerate().for_each(|(_, c)| {
        c.clone()
            .into_iter()
            .map(|f| (f.clone(), f.get_rank_string()))
            .permutations(n)
            .filter_map(|perm| {
                FR::resolve_conflicts(&perm)
                    .map(|res| {
                        (
                            perm.iter()
                                .map(|(_, name)| name.clone())
                                .collect::<Vec<_>>(),
                            res,
                        )
                    })
                    .ok()
            })
            .combinations(2)
            .for_each(|pair| {
                let (names1, mut res1) = pair[0].clone();
                let (names2, mut res2) = pair[1].clone();
                assert_eq!(
                    res1.winner, res2.winner,
                    "Mismatch in winners between {:?} and {:?}",
                    names1, names2,
                );
                res1.commemorations.sort();
                res2.commemorations.sort();
                assert_eq!(
                    res1.commemorations, res2.commemorations,
                    "Mismatch in commemorations between {:?} and {:?}, {:?} {:?}",
                    names1, names2, res1.transferred, res2.transferred
                );
                assert_eq!(
                    res1.winner_rank, res2.winner_rank,
                    "Mismatch in winning ranks between {names1:?} and {names2:?}"
                );
                assert_eq!(
                    res1.transferred, res2.transferred,
                    "Mismatch in transferred status between {names1:?} and {names2:?}"
                );
            });

        // make sure all permutations yield the same result
    });
}

fn build_occurance_graph<FR: FeastRankResolver>(
    enumeration: Vec<FR>,
) -> HashMap<String, HashSet<String>> {
    let mut adj_list = HashMap::new();
    for (feast1, feast2) in itertools::iproduct!(enumeration.iter(), enumeration.iter()) {
        let rank1 = format!("{}", feast1.id());
        let rank2 = format!("{}", feast2.id());

        // Use the generic resolve_conflicts implementation with a two-item
        // competitor list so this diagnostic can be reused across different
        // FeastRank implementations. Call it in both orders and ensure the
        // winners agree (resolve_conflicts should be order-independent).
        let res_ab = FR::resolve_conflicts(&[
            (FR::clone(feast1), rank1.clone()),
            (FR::clone(feast2), rank2.clone()),
        ]);

        let res_ba = FR::resolve_conflicts(&[
            (FR::clone(feast2), rank2.clone()),
            (FR::clone(feast1), rank1.clone()),
        ]);

        match (res_ab, res_ba) {
            (Ok(r_ab), Ok(r_ba)) => {
                // Both succeeded; ensure they picked the same winner.
                assert!(
                    !(r_ab.winner != r_ba.winner),
                    "Inconsistent winners for {} vs {}: '{}' vs '{}'",
                    rank1,
                    rank2,
                    r_ab.winner,
                    r_ba.winner
                );
                // Add directed edge from winner -> loser
                if r_ab.winner == rank1 {
                    adj_list
                        .entry(rank1.clone())
                        .or_insert_with(HashSet::new)
                        .insert(rank2.clone());
                } else if r_ab.winner == rank2 {
                    adj_list
                        .entry(rank2.clone())
                        .or_insert_with(HashSet::new)
                        .insert(rank1.clone());
                } else {
                    panic!(
                        "Unexpected winner '{}' for competitors '{}' and '{}'",
                        r_ab.winner, rank1, rank2
                    );
                }
            }
            (Ok(r_ab), Err(_)) => {
                // One order succeeded while the swapped order failed. Use
                // the successful result to derive the directed edge.
                if r_ab.winner == rank1 {
                    adj_list
                        .entry(rank1.clone())
                        .or_insert_with(HashSet::new)
                        .insert(rank2.clone());
                } else if r_ab.winner == rank2 {
                    adj_list
                        .entry(rank2.clone())
                        .or_insert_with(HashSet::new)
                        .insert(rank1.clone());
                }
            }
            (Err(_), Ok(r_ba)) => {
                // Swapped order succeeded; use that result.
                if r_ba.winner == rank1 {
                    adj_list
                        .entry(rank1.clone())
                        .or_insert_with(HashSet::new)
                        .insert(rank2.clone());
                } else if r_ba.winner == rank2 {
                    adj_list
                        .entry(rank2.clone())
                        .or_insert_with(HashSet::new)
                        .insert(rank1.clone());
                }
            }
            (Err(_e1), Err(_e2)) => {
                // Both sides failed to resolve; treat as no edge (same as
                // previous behaviour where both resolve_occurrence calls
                // returned Err).
            }
        }
    }
    adj_list
}

pub fn test_feast_rank_enumeration_occurance_graph<FR: FeastRankResolver>(enumeration: Vec<FR>) {
    let graph = build_occurance_graph::<FR>(enumeration);
    // Use Johnson's algorithm to enumerate all elementary cycles in the
    // directed graph. This finds every simple cycle exactly once and is
    // better suited for diagnostics than the previous single-DFS attempt.
    fn johnson_enumerate_cycles(graph: &HashMap<String, HashSet<String>>) -> Vec<Vec<String>> {
        // Map nodes to indices for efficient processing
        let mut nodes: Vec<String> = graph.keys().cloned().collect();
        nodes.sort();
        let idx_of: HashMap<String, usize> = nodes
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, n)| (n, i))
            .collect();

        // Adjacency list with integer indices
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];
        for (u, neighs) in graph {
            if let Some(&ui) = idx_of.get(u) {
                for v in neighs {
                    if let Some(&vi) = idx_of.get(v) {
                        adj[ui].push(vi);
                    }
                }
            }
        }

        let mut blocked = vec![false; nodes.len()];
        let mut set_b: Vec<HashSet<usize>> = vec![HashSet::new(); nodes.len()];
        let mut stack: Vec<usize> = Vec::new();
        let mut cycles: Vec<Vec<String>> = Vec::new();

        fn unblock(v: usize, blocked: &mut [bool], set_b: &mut [HashSet<usize>]) {
            if blocked[v] {
                blocked[v] = false;
                let to_unblock: Vec<usize> = set_b[v].drain().collect();
                for w in to_unblock {
                    unblock(w, blocked, set_b);
                }
            }
        }

        fn circuit(
            v: usize,
            s: usize,
            adj: &Vec<Vec<usize>>,
            blocked: &mut [bool],
            set_b: &mut [HashSet<usize>],
            stack: &mut Vec<usize>,
            cycles: &mut Vec<Vec<String>>,
            nodes: &Vec<String>,
        ) -> bool {
            let mut found_cycle = false;
            blocked[v] = true;
            stack.push(v);

            for &w in &adj[v] {
                if w == s {
                    let mut cycle: Vec<String> = stack.iter().map(|&i| nodes[i].clone()).collect();
                    // close cycle by repeating start
                    cycle.push(nodes[s].clone());
                    cycles.push(cycle);
                    found_cycle = true;
                } else if !blocked[w] && circuit(w, s, adj, blocked, set_b, stack, cycles, nodes) {
                    found_cycle = true;
                }
            }

            if found_cycle {
                unblock(v, blocked, set_b);
            } else {
                for &w in &adj[v] {
                    set_b[w].insert(v);
                }
            }

            stack.pop();
            found_cycle
        }

        let n = nodes.len();
        for s in 0..n {
            // subgraph restricted to vertices with index >= s
            let mut sub_adj: Vec<Vec<usize>> = vec![Vec::new(); n];
            for u in s..n {
                sub_adj[u] = adj[u].iter().copied().filter(|&v| v >= s).collect();
            }

            for i in s..n {
                blocked[i] = false;
                set_b[i].clear();
            }
            stack.clear();

            // run circuit from s on subgraph
            circuit(
                s,
                s,
                &sub_adj,
                &mut blocked,
                &mut set_b,
                &mut stack,
                &mut cycles,
                &nodes,
            );
        }

        // Normalize cycles: rotate each cycle so the lexicographically
        // smallest node is first (helps deduplicate rotated duplicates),
        // then deduplicate.
        println!("Normalizing and deduplicating cycles");
        let mut norm_cycles: Vec<Vec<String>> = cycles
            .into_iter()
            .map(|mut cyc| {
                // cyc is closed (last == first). Drop the trailing repeated node
                if cyc.len() > 1 && cyc.first() == cyc.last() {
                    cyc.pop();
                }
                // find smallest index lexicographically
                if cyc.is_empty() {
                    return cyc;
                }
                let mut min_idx = 0usize;
                for (i, s) in cyc.iter().enumerate() {
                    if s < &cyc[min_idx] {
                        min_idx = i;
                    }
                }
                // rotate
                let mut rotated = Vec::with_capacity(cyc.len() + 1);
                for i in 0..cyc.len() {
                    rotated.push(cyc[(min_idx + i) % cyc.len()].clone());
                }
                // close cycle again
                rotated.push(rotated[0].clone());
                rotated
            })
            .collect();
        println!("Cycles normalized");
        // deduplicate by string representation
        norm_cycles.sort();
        norm_cycles.reverse();
        norm_cycles.dedup();
        norm_cycles
    }

    let all_cycles = johnson_enumerate_cycles(&graph);

    assert!(
        all_cycles.is_empty(),
        "Cycles detected in occurrence graph:\n{}\n{} cycles",
        all_cycles
            .iter()
            .map(|cycle| cycle.join(" -> "))
            .collect::<Vec<_>>()
            .join("\n"),
        all_cycles.len()
    )
}

// Johnson's algorithm replaces the previous simple DFS cycle finder.
