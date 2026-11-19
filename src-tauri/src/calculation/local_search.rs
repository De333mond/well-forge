use super::{
    Problem, check_cancelled, parallel, penalty, placement::Placement, scenarios::scenario_reserves,
};
use crate::project::Project;
use std::sync::atomic::AtomicBool;

pub(super) fn solve(
    problem: &Problem,
    project: &Project,
    cancelled: &AtomicBool,
) -> Result<(Placement, f64), String> {
    let samples = Samples::new(problem, project, cancelled)?;
    let starts = if samples.normalizer > 0.0 { 6 } else { 1 };
    let solutions = parallel(starts, cancelled, |start| {
        search(&samples, start, cancelled)
    })?;
    let (sites, closest, score) = solutions
        .into_iter()
        .min_by(|a, b| a.2.total_cmp(&b.2).then(a.0.cmp(&b.0)))
        .ok_or("Допустимая расстановка не найдена.")?;
    let assignment = closest.iter().map(|near| sites[near.slot]).collect();
    Ok((
        Placement {
            sites,
            assignment,
            cost: score,
        },
        score,
    ))
}

struct Samples<'a> {
    problem: &'a Problem,
    epsilon: f64,
    count: usize,
    tail_percent: u32,
    normalizer: f64,
    cached: Option<Vec<Vec<f64>>>,
}

impl<'a> Samples<'a> {
    fn new(
        problem: &'a Problem,
        project: &Project,
        cancelled: &AtomicBool,
    ) -> Result<Self, String> {
        let total: f64 = problem.blocks.iter().map(|block| block.reserve).sum();
        let mut samples = Self {
            problem,
            epsilon: project.epsilon,
            count: if project.epsilon == 0.0 || total == 0.0 {
                1
            } else {
                project.scenarios as usize
            },
            tail_percent: project.tail_percent,
            // Scale every scenario by the same nominal total to preserve changes in gas volume.
            normalizer: if total == 0.0 { 0.0 } else { 1.0 / total },
            cached: None,
        };
        if samples.count.saturating_mul(problem.blocks.len()) <= 8_000_000 {
            samples.cached = Some(parallel(samples.count, cancelled, |index| {
                Ok(samples.weights(index).collect())
            })?);
        }
        Ok(samples)
    }

    fn weights(&self, index: usize) -> impl Iterator<Item = f64> + '_ {
        scenario_reserves(self.problem, self.epsilon, index)
            .map(|reserve| reserve * self.normalizer)
    }

    fn tail(&self, distances: &[f64], cancelled: &AtomicBool) -> Result<Tail, String> {
        let mut losses = Vec::with_capacity(self.count.min(4096));
        for index in 0..self.count {
            check_cancelled(cancelled)?;
            let loss = if let Some(samples) = &self.cached {
                penalty(distances, &samples[index])
            } else {
                distances
                    .iter()
                    .zip(self.weights(index))
                    .map(|(distance, weight)| distance * weight)
                    .sum()
            };
            losses.push((index, loss));
        }
        Ok(upper_tail(losses, self.tail_percent))
    }

    fn tail_weights(&self, tail: &Tail, cancelled: &AtomicBool) -> Result<Vec<f64>, String> {
        let mut combined = vec![0.0; self.problem.blocks.len()];
        for &(index, share) in &tail.scenarios {
            check_cancelled(cancelled)?;
            if let Some(samples) = &self.cached {
                for (weight, &reserve) in combined.iter_mut().zip(&samples[index]) {
                    *weight += share * reserve;
                }
            } else {
                for (weight, reserve) in combined.iter_mut().zip(self.weights(index)) {
                    *weight += share * reserve;
                }
            }
        }
        Ok(combined)
    }
}

struct Tail {
    score: f64,
    scenarios: Vec<(usize, f64)>,
}

fn upper_tail(mut losses: Vec<(usize, f64)>, percent: u32) -> Tail {
    // Express mass in hundredths of one sample. The boundary sample may
    // contribute only a fraction of its loss.
    let mass = losses.len() as u64 * u64::from(percent);
    let last = (mass.div_ceil(100) - 1) as usize;
    losses.select_nth_unstable_by(last, |a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    losses.truncate(last + 1);
    let mut score = 0.0;
    for (rank, (_, value)) in losses.iter_mut().enumerate() {
        let units = if rank == last {
            (mass - 1) % 100 + 1
        } else {
            100
        };
        let share = units as f64 / mass as f64;
        score += *value * share;
        *value = share;
    }
    Tail {
        score,
        scenarios: losses,
    }
}

#[derive(Clone, Copy)]
struct Nearest {
    slot: usize,
    distance: f64,
    second: f64,
}

fn search(
    samples: &Samples<'_>,
    start: usize,
    cancelled: &AtomicBool,
) -> Result<(Vec<usize>, Vec<Nearest>, f64), String> {
    let problem = samples.problem;
    let mut sites = starting_sites(problem, start, cancelled)?;
    let mut closest = nearest(problem, &sites, cancelled)?;
    let mut distances: Vec<_> = closest.iter().map(|near| near.distance).collect();
    let mut tail = samples.tail(&distances, cancelled)?;
    for step in 0..16 {
        check_cancelled(cancelled)?;
        if tail.score == 0.0 {
            break;
        }
        let weights = samples.tail_weights(&tail, cancelled)?;
        let tolerance = tail.score * 1e-12;
        let moves = ranked_moves(
            problem,
            &sites,
            &closest,
            &weights,
            (start, step),
            tolerance,
            cancelled,
        )?;
        let mut best_score = tail.score;
        let mut best = None;
        for movement in moves {
            // The current tail weights yield a lower bound for the true
            // tail loss after the swap. A swap cannot outrun that bound.
            if tail.score + movement.delta >= best_score - tolerance {
                break;
            }
            for (block, (distance, near)) in distances.iter_mut().zip(&closest).enumerate() {
                let incumbent = if near.slot == movement.slot {
                    near.second
                } else {
                    near.distance
                };
                *distance = incumbent.min(problem.distance(movement.incoming, block));
            }
            let candidate = samples.tail(&distances, cancelled)?;
            if candidate.score < best_score - tolerance {
                best_score = candidate.score;
                best = Some((movement, candidate));
            }
        }
        let Some((movement, candidate)) = best else {
            break;
        };
        sites[movement.slot] = movement.incoming;
        tail = candidate;
        closest = nearest(problem, &sites, cancelled)?;
    }
    Ok((sites, closest, tail.score))
}

fn starting_sites(
    problem: &Problem,
    start: usize,
    cancelled: &AtomicBool,
) -> Result<Vec<usize>, String> {
    let n = problem.blocks.len();
    let s = problem.wells;
    let count = n.min((8_000_000 / n / s).clamp(8, 512));
    if count < s || n.saturating_mul(s) > 1_000_000 {
        return Ok((0..s)
            .map(|i| ((i as u64 * n as u64 / s as u64) as usize + start) % n)
            .collect());
    }
    let mut candidates: Vec<_> = sampled_sites(n, count, start).collect();
    let mut sites = Vec::with_capacity(s);
    let mut closest = vec![f64::INFINITY; n];
    for step in 0..s {
        check_cancelled(cancelled)?;
        let mut ranked = Vec::with_capacity(candidates.len());
        for &candidate in &candidates {
            check_cancelled(cancelled)?;
            let cost: f64 = closest
                .iter()
                .zip(&problem.blocks)
                .enumerate()
                .map(|(block, (&prior, reserve))| {
                    prior.min(problem.distance(candidate, block)) * reserve.reserve
                })
                .sum();
            ranked.push((candidate, cost));
        }
        ranked.sort_unstable_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
        let rank = if start == 0 {
            0
        } else {
            (start + step) % ranked.len().min(3)
        };
        let site = ranked[rank].0;
        candidates.retain(|&candidate| candidate != site);
        sites.push(site);
        for (block, distance) in closest.iter_mut().enumerate() {
            *distance = distance.min(problem.distance(site, block));
        }
    }
    Ok(sites)
}

fn sampled_sites(n: usize, count: usize, offset: usize) -> impl Iterator<Item = usize> {
    (0..count).map(move |i| ((i as u64 * n as u64 / count as u64) as usize + offset) % n)
}

fn nearest(
    problem: &Problem,
    sites: &[usize],
    cancelled: &AtomicBool,
) -> Result<Vec<Nearest>, String> {
    (0..problem.blocks.len())
        .map(|block| {
            check_cancelled(cancelled)?;
            let mut near = Nearest {
                slot: 0,
                distance: f64::INFINITY,
                second: f64::INFINITY,
            };
            for (slot, &site) in sites.iter().enumerate() {
                let distance = problem.distance(site, block);
                if distance < near.distance || (distance == near.distance && site == block) {
                    near = Nearest {
                        slot,
                        distance,
                        second: near.distance,
                    };
                } else {
                    near.second = near.second.min(distance);
                }
            }
            Ok(near)
        })
        .collect()
}

#[derive(Clone, Copy)]
struct Move {
    slot: usize,
    incoming: usize,
    delta: f64,
}

fn ranked_moves(
    problem: &Problem,
    sites: &[usize],
    closest: &[Nearest],
    weights: &[f64],
    pass: (usize, usize),
    tolerance: f64,
    cancelled: &AtomicBool,
) -> Result<Vec<Move>, String> {
    const SHORTLIST: usize = 12;
    let n = problem.blocks.len();
    let count = n.min((2_000_000 / n).clamp(12, 1024));
    let mut candidates: Vec<_> = sampled_sites(n, count, pass.0 + pass.1 * 3).collect();

    let mut occupied = vec![false; n];
    for &site in sites {
        occupied[site] = true;
        if count == n {
            continue;
        }
        let center = &problem.blocks[site];
        for row in center.row.saturating_sub(1)..=center.row + 1 {
            for col in center.col.saturating_sub(1)..=center.col + 1 {
                if let Ok(block) = problem
                    .blocks
                    .binary_search_by_key(&(row, col), |cell| (cell.row, cell.col))
                {
                    candidates.push(block);
                }
            }
        }
    }
    candidates.sort_unstable();
    candidates.dedup();

    let mut moves: Vec<Move> = Vec::with_capacity(SHORTLIST);
    let mut correction = vec![0.0; sites.len()];
    for incoming in candidates {
        check_cancelled(cancelled)?;
        if occupied[incoming] {
            continue;
        }
        correction.fill(0.0);
        let mut gain = 0.0;
        for (block, (near, &weight)) in closest.iter().zip(weights).enumerate() {
            let distance = problem.distance(incoming, block);
            let with_incoming = distance.min(near.distance);
            gain += weight * (with_incoming - near.distance);
            correction[near.slot] += weight * (distance.min(near.second) - with_incoming);
        }
        for (slot, loss) in correction.iter().enumerate() {
            let delta = gain + loss;
            if delta >= -tolerance
                || (moves.len() == SHORTLIST && delta >= moves[SHORTLIST - 1].delta)
            {
                continue;
            }
            let index = moves.partition_point(|movement| movement.delta <= delta);
            if moves.len() == SHORTLIST {
                moves.pop();
            }
            moves.insert(
                index,
                Move {
                    slot,
                    incoming,
                    delta,
                },
            );
        }
    }
    Ok(moves)
}

#[cfg(test)]
mod tests {
    use super::upper_tail;

    #[test]
    fn tail_uses_only_the_included_fraction_of_the_boundary_scenario() {
        let tail = upper_tail(vec![(0, 2.0), (1, 6.0), (2, 8.0)], 50);
        assert!((tail.score - 22.0 / 3.0).abs() < 1e-12);
        assert_eq!(tail.scenarios, [(2, 2.0 / 3.0), (1, 1.0 / 3.0)]);
    }
}
