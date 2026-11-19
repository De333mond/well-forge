use super::{
    Problem, check_cancelled, exact, parallel, penalty,
    placement::{self, Placement},
};
use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    sync::atomic::AtomicBool,
};

pub(super) fn solve(
    problem: &Problem,
    epsilon: f64,
    count: usize,
    cancelled: &AtomicBool,
) -> Result<(Placement, bool), String> {
    // Bound memory by keeping only one batch of candidates and one batch
    // of test scenarios. The sample is reproducible from its index.
    let batch_size = (1_000_000 / problem.blocks.len()).clamp(1, 256);
    let mut best: Option<(Placement, f64)> = None;
    let mut optimal = true;
    for first in (0..count).step_by(batch_size) {
        let solutions = parallel((count - first).min(batch_size), cancelled, |offset| {
            let scenario = first + offset;
            let weights = scenario_weights(problem, epsilon, scenario);
            match exact::solve(problem, &weights, cancelled)? {
                Some(solution) => Ok((solution, true)),
                None => Ok((
                    placement::improve(problem, &weights, scenario, cancelled)?,
                    false,
                )),
            }
        })?;

        let mut unique: HashMap<u64, Vec<usize>> = HashMap::new();
        let mut candidates: Vec<Placement> = Vec::new();
        for (solution, certified) in solutions {
            optimal &= certified;
            let mut hasher = DefaultHasher::new();
            solution.assignment.hash(&mut hasher);
            let matching = unique.entry(hasher.finish()).or_default();
            if !matching
                .iter()
                .any(|&i| candidates[i].assignment == solution.assignment)
            {
                matching.push(candidates.len());
                candidates.push(solution);
            }
        }

        let mut scores = vec![0.0_f64; candidates.len()];
        for sample_first in (0..count).step_by(batch_size) {
            let samples = parallel(
                (count - sample_first).min(batch_size),
                cancelled,
                |offset| Ok(scenario_weights(problem, epsilon, sample_first + offset)),
            )?;
            scores = parallel(candidates.len(), cancelled, |index| {
                let distances: Vec<_> = candidates[index]
                    .assignment
                    .iter()
                    .enumerate()
                    .map(|(block, &site)| problem.distance(site, block))
                    .collect();
                let mut worst = scores[index];
                for weights in &samples {
                    check_cancelled(cancelled)?;
                    worst = worst.max(penalty(&distances, weights));
                }
                Ok(worst)
            })?;
        }
        for (candidate, score) in candidates.into_iter().zip(scores) {
            if best.as_ref().is_none_or(|(_, current)| score < *current) {
                best = Some((candidate, score));
            }
        }
    }
    Ok((best.ok_or("Нет сценариев для расчёта.")?.0, optimal))
}

fn scenario_weights(problem: &Problem, epsilon: f64, scenario: usize) -> Vec<f64> {
    problem.weights(scenario_reserves(problem, epsilon, scenario))
}

pub(super) fn scenario_reserves(
    problem: &Problem,
    epsilon: f64,
    scenario: usize,
) -> impl Iterator<Item = f64> + '_ {
    // SplitMix64 gives reproducible, independent streams for scenario jobs.
    let mut seed = (scenario as u64).wrapping_mul(0xd134_2543_de82_ef95);
    problem.blocks.iter().map(move |block| {
        seed = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = seed;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        let uniform = ((z ^ (z >> 31)) >> 11) as f64 * (1.0 / (1_u64 << 53) as f64);
        block.reserve * (1.0 - epsilon + 2.0 * epsilon * uniform)
    })
}
