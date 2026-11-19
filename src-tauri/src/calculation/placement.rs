use super::{Problem, check_cancelled};
use std::sync::atomic::AtomicBool;

pub(super) struct Placement {
    pub sites: Vec<usize>,
    pub assignment: Vec<usize>,
    pub cost: f64,
}

pub(super) fn placement(
    problem: &Problem,
    weights: &[f64],
    sites: Vec<usize>,
    assignment: Vec<usize>,
) -> Placement {
    let cost = assignment
        .iter()
        .enumerate()
        .map(|(block, &site)| weights[block] * problem.distance(site, block))
        .sum();
    Placement {
        sites,
        assignment,
        cost,
    }
}

pub(super) fn improve(
    problem: &Problem,
    weights: &[f64],
    start: usize,
    cancelled: &AtomicBool,
) -> Result<Placement, String> {
    let n = problem.blocks.len();
    let s = problem.wells;
    let mut order: Vec<_> = (0..n).collect();
    let mut sites = Vec::with_capacity(s);
    let mut assignment = vec![0; n];
    let mut partition = Partition {
        problem,
        weights,
        start,
        sites: &mut sites,
        assignment: &mut assignment,
        cancelled,
    };
    partition.split(&mut order, s, n % s, 0)?;
    let mut best = placement(problem, weights, sites, assignment);

    // Reassign all blocks only when the number of site/block pairs is modest.
    // Each restart begins with a feasible balanced partition.
    if n.saturating_mul(s) > 300_000 {
        return Ok(best);
    }
    order.sort_unstable_by(|&a, &b| weights[b].total_cmp(&weights[a]).then(a.cmp(&b)));
    let mut sites = best.sites.clone();
    for iteration in 0..4 {
        check_cancelled(cancelled)?;
        let assignment = assign_greedily(problem, &sites, &order);
        let mut group_for_site = vec![0; n];
        for (index, &site) in sites.iter().enumerate() {
            group_for_site[site] = index;
        }
        let mut groups = vec![Vec::new(); s];
        for (block, &site) in assignment.iter().enumerate() {
            groups[group_for_site[site]].push(block);
        }
        if s <= 24 {
            improve_assignment(problem, weights, &sites, &mut groups, cancelled)?;
        }
        let new_sites: Vec<_> = groups
            .iter()
            .zip(&sites)
            .map(|(group, &site)| {
                medoid(
                    problem,
                    weights,
                    group,
                    Some(site),
                    start.wrapping_add(iteration),
                )
            })
            .collect();
        let mut new_assignment = assignment;
        for (group, &site) in groups.iter().zip(&new_sites) {
            for &block in group {
                new_assignment[block] = site;
            }
        }
        let candidate = placement(problem, weights, new_sites.clone(), new_assignment);
        if candidate.cost < best.cost {
            best = candidate;
        }
        if sites == new_sites {
            break;
        }
        sites = new_sites;
    }
    Ok(best)
}

fn improve_assignment(
    problem: &Problem,
    weights: &[f64],
    sites: &[usize],
    groups: &mut [Vec<usize>],
    cancelled: &AtomicBool,
) -> Result<(), String> {
    let mut left_moves = Vec::new();
    let mut right_moves = Vec::new();
    for i in 0..sites.len() {
        check_cancelled(cancelled)?;
        for j in i + 1..sites.len() {
            let (before, after) = groups.split_at_mut(j);
            let (left, right) = (&mut before[i], &mut after[0]);
            let left_site = sites[i];
            let right_site = sites[j];
            let delta = |block: usize, from, to| {
                weights[block] * (problem.distance(to, block) - problem.distance(from, block))
            };
            // With unequal loads a move from the larger group flips which
            // site owns the extra slot. Swaps keep both loads intact.
            if left.len() != right.len() {
                let (larger, smaller, from, to) = if left.len() > right.len() {
                    (&mut *left, &mut *right, left_site, right_site)
                } else {
                    (&mut *right, &mut *left, right_site, left_site)
                };
                if let Some((index, _)) = larger
                    .iter()
                    .enumerate()
                    .filter(|&(_, &block)| block != from)
                    .map(|(index, &block)| (index, delta(block, from, to)))
                    .filter(|&(_, cost)| cost < 0.0)
                    .min_by(|a, b| a.1.total_cmp(&b.1))
                {
                    smaller.push(larger.swap_remove(index));
                }
            }
            left_moves.clear();
            left_moves.extend(
                left.iter()
                    .enumerate()
                    .filter(|&(_, &block)| block != left_site)
                    .map(|(index, &block)| (index, delta(block, left_site, right_site))),
            );
            right_moves.clear();
            right_moves.extend(
                right
                    .iter()
                    .enumerate()
                    .filter(|&(_, &block)| block != right_site)
                    .map(|(index, &block)| (index, delta(block, right_site, left_site))),
            );
            left_moves.sort_unstable_by(|a, b| a.1.total_cmp(&b.1));
            right_moves.sort_unstable_by(|a, b| a.1.total_cmp(&b.1));
            for (&(l, left_cost), &(r, right_cost)) in left_moves.iter().zip(&right_moves) {
                if left_cost + right_cost >= 0.0 {
                    break;
                }
                std::mem::swap(&mut left[l], &mut right[r]);
            }
        }
    }
    Ok(())
}

pub(super) fn polish(
    problem: &Problem,
    weights: &[f64],
    initial: &Placement,
    cancelled: &AtomicBool,
) -> Result<Placement, String> {
    let mut best = placement(
        problem,
        weights,
        initial.sites.clone(),
        initial.assignment.clone(),
    );
    let mut group_for_site = vec![0; problem.blocks.len()];
    for _ in 0..5 {
        check_cancelled(cancelled)?;
        for (index, &site) in best.sites.iter().enumerate() {
            group_for_site[site] = index;
        }
        let mut groups = vec![Vec::new(); best.sites.len()];
        for (block, &site) in best.assignment.iter().enumerate() {
            groups[group_for_site[site]].push(block);
        }
        improve_assignment(problem, weights, &best.sites, &mut groups, cancelled)?;
        let sites: Vec<_> = groups
            .iter()
            .zip(&best.sites)
            .map(|(group, &site)| {
                best_site(
                    problem,
                    weights,
                    group,
                    std::iter::once(site).chain(group.iter().copied()),
                )
            })
            .collect();
        let mut assignment = vec![0; problem.blocks.len()];
        for (group, &site) in groups.iter().zip(&sites) {
            for &block in group {
                assignment[block] = site;
            }
        }
        let candidate = placement(problem, weights, sites, assignment);
        if candidate.cost >= best.cost {
            break;
        }
        best = candidate;
    }
    Ok(best)
}

struct Partition<'a> {
    problem: &'a Problem,
    weights: &'a [f64],
    start: usize,
    sites: &'a mut Vec<usize>,
    assignment: &'a mut [usize],
    cancelled: &'a AtomicBool,
}

impl Partition<'_> {
    fn split(
        &mut self,
        blocks: &mut [usize],
        wells: usize,
        extra: usize,
        depth: usize,
    ) -> Result<(), String> {
        check_cancelled(self.cancelled)?;
        if wells == 1 {
            let site = medoid(self.problem, self.weights, blocks, None, self.start);
            self.sites.push(site);
            for &block in blocks.iter() {
                self.assignment[block] = site;
            }
            return Ok(());
        }
        let (min_row, max_row, min_col, max_col) = blocks.iter().fold(
            (usize::MAX, 0, usize::MAX, 0),
            |(r_min, r_max, c_min, c_max), &block| {
                let block = &self.problem.blocks[block];
                (
                    r_min.min(block.row),
                    r_max.max(block.row),
                    c_min.min(block.col),
                    c_max.max(block.col),
                )
            },
        );
        let distances = &self.problem.distances;
        let row_span = distances[(max_row - min_row) * self.problem.columns];
        let col_span = distances[max_col - min_col];
        let axes = match (self.start.wrapping_add(depth * 4)) % 8 {
            0 | 1 => usize::from(row_span < col_span),
            2 => 0,
            3 => 1,
            4 | 5 => usize::from(row_span >= col_span),
            6 => 2,
            _ => 3,
        };
        let left_wells = wells / 2;
        let floor = blocks.len() / wells - usize::from(extra == wells);
        let left_extra = if self.start.is_multiple_of(2) {
            extra.min(left_wells)
        } else {
            extra.saturating_sub(wells - left_wells)
        };
        let split = left_wells * floor + left_extra;
        blocks.select_nth_unstable_by_key(split, |&id| {
            let block = &self.problem.blocks[id];
            let row = block.row as i64;
            let col = block.col as i64;
            let projection = match axes {
                0 => row,
                1 => col,
                2 => row + col,
                _ => row - col,
            };
            (projection, id)
        });
        let (left, right) = blocks.split_at_mut(split);
        self.split(left, left_wells, left_extra, depth + 1)?;
        self.split(right, wells - left_wells, extra - left_extra, depth + 1)
    }
}

fn medoid(
    problem: &Problem,
    weights: &[f64],
    blocks: &[usize],
    current: Option<usize>,
    start: usize,
) -> usize {
    let samples = blocks.len().min(16);
    let strongest = blocks
        .iter()
        .copied()
        .max_by(|&a, &b| weights[a].total_cmp(&weights[b]))
        .unwrap();
    let candidates = (0..samples)
        .map(|sample| {
            blocks[(sample * blocks.len() / samples + start % blocks.len()) % blocks.len()]
        })
        .chain([strongest])
        .chain(current);
    best_site(problem, weights, blocks, candidates)
}

fn best_site(
    problem: &Problem,
    weights: &[f64],
    blocks: &[usize],
    candidates: impl Iterator<Item = usize>,
) -> usize {
    let mut best = (blocks[0], f64::INFINITY);
    for site in candidates {
        let mut cost = 0.0;
        for &block in blocks {
            cost += weights[block] * problem.distance(site, block);
            if cost > best.1 {
                break;
            }
        }
        if cost < best.1 || (cost == best.1 && site < best.0) {
            best = (site, cost);
        }
    }
    best.0
}

fn assign_greedily(problem: &Problem, sites: &[usize], order: &[usize]) -> Vec<usize> {
    let n = order.len();
    let floor = n / sites.len();
    let mut extra = n % sites.len();
    let mut counts = vec![1; sites.len()];
    let mut assignment = vec![usize::MAX; n];
    for &site in sites {
        assignment[site] = site;
    }
    for &block in order {
        if assignment[block] != usize::MAX {
            continue;
        }
        let index = (0..sites.len())
            .filter(|&i| counts[i] < floor || (counts[i] == floor && extra > 0))
            .min_by(|&a, &b| {
                problem
                    .distance(sites[a], block)
                    .total_cmp(&problem.distance(sites[b], block))
            })
            .unwrap();
        if counts[index] == floor {
            extra -= 1;
        }
        counts[index] += 1;
        assignment[block] = sites[index];
    }
    assignment
}
