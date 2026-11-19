use super::{
    Problem, check_cancelled, parallel,
    placement::{Placement, placement},
};
use std::{
    ops::Range,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed},
    thread,
};

pub(super) fn solve(
    problem: &Problem,
    weights: &[f64],
    cancelled: &AtomicBool,
) -> Result<Option<Placement>, String> {
    let n = problem.blocks.len();
    let s = problem.wells;
    if weights.iter().all(|&weight| weight == 0.0) || problem.gamma == 0.0 {
        let mut order: Vec<_> = (0..n).collect();
        order.select_nth_unstable_by(s, |&a, &b| {
            weights[b].total_cmp(&weights[a]).then(a.cmp(&b))
        });
        let (sites, other) = order.split_at(s);
        let mut assignment = vec![0; n];
        let mut remaining = other.iter().copied();
        for (index, &site) in sites.iter().enumerate() {
            assignment[site] = site;
            let capacity = n / s + usize::from(index < n % s);
            for block in remaining.by_ref().take(capacity - 1) {
                assignment[block] = site;
            }
        }
        return Ok(Some(placement(
            problem,
            weights,
            sites.to_vec(),
            assignment,
        )));
    }

    // Estimate work before enumerating subsets. Each subset needs one assignment
    // over the non-well blocks, plus the optional extra slots for balance.
    const WORK_LIMIT: usize = 8_000_000;
    let slots = n.div_ceil(s) * s - s;
    let work_per_subset = if s == 1 {
        n
    } else {
        slots.saturating_pow(3).saturating_add(n)
    };
    let mut combinations = 1_usize;
    for k in 1..=s.min(n - s) {
        combinations = combinations.saturating_mul(n - k + 1) / k;
        if combinations > WORK_LIMIT / work_per_subset {
            return Ok(None);
        }
    }

    enumerate(problem, weights, cancelled, 0..n - s + 1)
}

pub(super) fn exhaustive(
    problem: &Problem,
    weights: &[f64],
    cancelled: &AtomicBool,
) -> Result<Placement, String> {
    let end = problem.blocks.len() - problem.wells + 1;
    let workers = thread::available_parallelism()
        .map_or(1, usize::from)
        .min(8)
        .min(end);
    let next = AtomicUsize::new(0);
    let candidates = parallel(workers, cancelled, |_| {
        let mut best = None;
        loop {
            check_cancelled(cancelled)?;
            let first = next.fetch_add(1, Relaxed);
            if first >= end {
                return Ok(best);
            }
            if let Some(candidate) = enumerate(problem, weights, cancelled, first..first + 1)? {
                keep_best(&mut best, candidate);
            }
        }
    })?;
    candidates
        .into_iter()
        .flatten()
        .min_by(|a, b| a.cost.total_cmp(&b.cost).then(a.sites.cmp(&b.sites)))
        .ok_or_else(|| "Допустимая расстановка не найдена.".into())
}

fn enumerate(
    problem: &Problem,
    weights: &[f64],
    cancelled: &AtomicBool,
    first_sites: Range<usize>,
) -> Result<Option<Placement>, String> {
    let n = problem.blocks.len();
    let s = problem.wells;
    let slots = n.div_ceil(s) * s - s;
    let mut sites: Vec<_> = (first_sites.start..first_sites.start + s).collect();
    let mut best: Option<Placement> = None;
    while sites[0] < first_sites.end {
        check_cancelled(cancelled)?;
        let candidate = if s == 1 {
            placement(problem, weights, sites.clone(), vec![sites[0]; n])
        } else {
            assign_exact(problem, weights, &sites, slots, cancelled)?
        };
        keep_best(&mut best, candidate);
        let Some(index) = (0..s).rev().find(|&i| sites[i] < n - s + i) else {
            break;
        };
        sites[index] += 1;
        for i in index + 1..s {
            sites[i] = sites[i - 1] + 1;
        }
    }
    Ok(best)
}

fn keep_best(best: &mut Option<Placement>, candidate: Placement) {
    if best.as_ref().is_none_or(|current| {
        candidate
            .cost
            .total_cmp(&current.cost)
            .then(candidate.sites.cmp(&current.sites))
            .is_lt()
    }) {
        *best = Some(candidate);
    }
}

fn assign_exact(
    problem: &Problem,
    weights: &[f64],
    sites: &[usize],
    slots: usize,
    cancelled: &AtomicBool,
) -> Result<Placement, String> {
    let n = problem.blocks.len();
    let s = sites.len();
    let mut assignment = vec![usize::MAX; n];
    for &site in sites {
        assignment[site] = site;
    }
    let blocks: Vec<_> = (0..n)
        .filter(|&block| assignment[block] == usize::MAX)
        .collect();
    let per_site = slots / s;
    let optional = !n.is_multiple_of(s);
    // Reserve each well's own block. Dummy blocks fill unused extra slots
    // and can only go into the last slot at each well.
    let matched = minimum_assignment(slots, cancelled, |row, column| {
        if let Some(&block) = blocks.get(row) {
            problem.distance(sites[column / per_site], block) * weights[block]
        } else if optional && column % per_site == per_site - 1 {
            0.0
        } else {
            f64::INFINITY
        }
    })?;
    for (column, row) in matched.into_iter().enumerate() {
        if let Some(&block) = blocks.get(row) {
            assignment[block] = sites[column / per_site];
        }
    }
    Ok(placement(problem, weights, sites.to_vec(), assignment))
}

// Hungarian algorithm. Row and column zero are sentinels during augmentation.
fn minimum_assignment(
    size: usize,
    cancelled: &AtomicBool,
    cost: impl Fn(usize, usize) -> f64,
) -> Result<Vec<usize>, String> {
    let mut row_potential = vec![0.0; size + 1];
    let mut column_potential = vec![0.0; size + 1];
    let mut matched = vec![0; size + 1];
    let mut predecessor = vec![0; size + 1];
    let mut slack = vec![f64::INFINITY; size + 1];
    let mut used = vec![false; size + 1];
    for row in 1..=size {
        matched[0] = row;
        slack.fill(f64::INFINITY);
        used.fill(false);
        let mut column = 0;
        loop {
            check_cancelled(cancelled)?;
            used[column] = true;
            let assigned = matched[column];
            let mut delta = f64::INFINITY;
            let mut next = 0;
            for j in 1..=size {
                if used[j] {
                    continue;
                }
                let reduced =
                    cost(assigned - 1, j - 1) - row_potential[assigned] - column_potential[j];
                if reduced < slack[j] {
                    slack[j] = reduced;
                    predecessor[j] = column;
                }
                if slack[j] < delta {
                    delta = slack[j];
                    next = j;
                }
            }
            for j in 0..=size {
                if used[j] {
                    row_potential[matched[j]] += delta;
                    column_potential[j] -= delta;
                } else {
                    slack[j] -= delta;
                }
            }
            column = next;
            if matched[column] == 0 {
                break;
            }
        }
        while column != 0 {
            let previous = predecessor[column];
            matched[column] = matched[previous];
            column = previous;
        }
    }
    Ok(matched[1..].iter().map(|&row| row - 1).collect())
}
