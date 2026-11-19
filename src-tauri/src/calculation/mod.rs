mod exact;
mod local_search;
mod placement;
mod scenarios;

use crate::project::{Grid, Project};
use placement::Placement;
use serde::{Deserialize, Serialize};
use std::{
    sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed},
    thread,
    time::Instant,
};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Algorithm {
    Scenarios,
    Interval,
    Exhaustive,
    LocalSearch,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Calculation {
    pub algorithm: Algorithm,
    pub worst_penalty: f64,
    pub nominal_penalty: f64,
    pub elapsed_ms: u64,
    pub optimal: bool,
    #[serde(default)]
    pub positive_reserves_only: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tail_penalty: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub well_by_cell: Option<Vec<u32>>,
}

struct Block {
    row: usize,
    col: usize,
    reserve: f64,
}

struct Problem {
    blocks: Vec<Block>,
    columns: usize,
    distances: Vec<f64>,
    wells: usize,
    gamma: f64,
}

impl Problem {
    fn new(grid: &Grid, wells: usize, gamma: f64) -> Result<Self, String> {
        if grid.values.iter().flatten().any(|&reserve| reserve < 0.0) {
            return Err("Запасы газа не могут быть отрицательными.".into());
        }
        let mut blocks: Vec<_> = grid
            .calculation_blocks()
            .map(|(index, reserve)| Block {
                row: index / grid.columns,
                col: index % grid.columns,
                reserve,
            })
            .collect();
        if wells == 0 || wells >= blocks.len() {
            return Err(
                "Число скважин должно быть меньше числа блоков с положительными запасами.".into(),
            );
        }
        let max_reserve = blocks.iter().map(|block| block.reserve).fold(0.0, f64::max);
        if max_reserve > 0.0 {
            for block in &mut blocks {
                block.reserve /= max_reserve;
            }
        }

        let width = grid.x_max - grid.x_min;
        let height = grid.y_max - grid.y_min;
        let dx = width / grid.columns.saturating_sub(1).max(1) as f64;
        let dy = height / grid.rows.saturating_sub(1).max(1) as f64;
        let maximum = width.hypot(height).powf(gamma);
        if !width.is_finite() || !height.is_finite() || !(maximum * blocks.len() as f64).is_finite()
        {
            return Err("Размеры сетки слишком велики для расчёта штрафа.".into());
        }
        let distances = if gamma == 0.0 {
            Vec::new()
        } else {
            (0..grid.values.len())
                .map(|offset| {
                    let distance = (offset % grid.columns) as f64 * dx;
                    let height = (offset / grid.columns) as f64 * dy;
                    distance.hypot(height).powf(gamma)
                })
                .collect()
        };
        Ok(Self {
            blocks,
            columns: grid.columns,
            distances,
            wells,
            gamma,
        })
    }

    fn distance(&self, site: usize, block: usize) -> f64 {
        if site == block {
            return 0.0;
        }
        if self.gamma == 0.0 {
            return 1.0;
        }
        let a = &self.blocks[site];
        let b = &self.blocks[block];
        self.distances[a.row.abs_diff(b.row) * self.columns + a.col.abs_diff(b.col)]
    }

    fn weights(&self, reserves: impl Iterator<Item = f64>) -> Vec<f64> {
        let mut weights: Vec<_> = reserves.collect();
        let maximum = weights.iter().copied().fold(0.0, f64::max);
        for weight in &mut weights {
            *weight = if self.gamma == 1.0 {
                1.0
            } else if maximum == 0.0 {
                0.0
            } else {
                (*weight / maximum).powf(1.0 - self.gamma)
            };
        }
        weights
    }

    fn worst_weights(&self, epsilon: f64) -> Vec<f64> {
        // Nominal reserves were scaled by their maximum. For epsilon = 1,
        // equal positive reserves in every positive block attain all weight bounds.
        self.weights(self.blocks.iter().map(|block| {
            if epsilon == 1.0 {
                f64::from(block.reserve > 0.0)
            } else {
                (block.reserve * (1.0 + epsilon)).min(1.0 - epsilon)
            }
        }))
    }
}

pub(crate) fn calculate(
    project: &Project,
    algorithm: Algorithm,
    cancelled: &AtomicBool,
) -> Result<(Vec<String>, Calculation), String> {
    let start = Instant::now();
    let grid = project.grid.as_ref().ok_or("Импортируйте сетку запасов.")?;
    let problem = Problem::new(grid, project.well_count as usize, project.gamma)?;
    let nominal = problem.weights(problem.blocks.iter().map(|block| block.reserve));
    let worst = problem.worst_weights(project.epsilon);
    let ((placement, optimal), tail_penalty) = match algorithm {
        Algorithm::Scenarios => {
            let scenarios = if project.epsilon == 0.0
                || project.gamma == 1.0
                || nominal.iter().all(|&weight| weight == 0.0)
            {
                1
            } else {
                project.scenarios as usize
            };
            (
                scenarios::solve(&problem, project.epsilon, scenarios, cancelled)?,
                None,
            )
        }
        Algorithm::Interval => (solve_interval(&problem, &worst, cancelled)?, None),
        Algorithm::Exhaustive => (
            (exact::exhaustive(&problem, &worst, cancelled)?, true),
            None,
        ),
        Algorithm::LocalSearch => {
            let geometry;
            let distances = if problem.gamma == 1.0 {
                &problem
            } else {
                geometry = Problem::new(grid, problem.wells, 1.0)?;
                &geometry
            };
            let (placement, tail_penalty) = local_search::solve(distances, project, cancelled)?;
            ((placement, false), Some(tail_penalty))
        }
    };
    check_cancelled(cancelled)?;
    let distances: Vec<_> = placement
        .assignment
        .iter()
        .enumerate()
        .map(|(block, &site)| problem.distance(site, block))
        .collect();
    let mut well_by_block = vec![0; problem.blocks.len()];
    let wells = placement
        .sites
        .iter()
        .enumerate()
        .map(|(index, &site)| {
            well_by_block[site] = index as u32 + 1;
            let block = &problem.blocks[site];
            format!("{}:{}", block.row + 1, block.col + 1)
        })
        .collect();
    // The grid keeps all cells. Zero denotes an unassigned cell, and the
    // other values are one-based indices into the returned well list.
    let mut well_by_cell = vec![0; grid.values.len()];
    for (block, &site) in problem.blocks.iter().zip(&placement.assignment) {
        well_by_cell[block.row * grid.columns + block.col] = well_by_block[site];
    }
    Ok((
        wells,
        Calculation {
            algorithm,
            worst_penalty: penalty(&distances, &worst),
            nominal_penalty: penalty(&distances, &nominal),
            elapsed_ms: start.elapsed().as_millis() as u64,
            optimal,
            positive_reserves_only: true,
            tail_penalty,
            well_by_cell: Some(well_by_cell),
        },
    ))
}

fn penalty(distances: &[f64], weights: &[f64]) -> f64 {
    distances
        .iter()
        .zip(weights)
        .map(|(distance, weight)| distance * weight)
        .sum()
}

fn solve_interval(
    problem: &Problem,
    weights: &[f64],
    cancelled: &AtomicBool,
) -> Result<(Placement, bool), String> {
    if let Some(solution) = exact::solve(problem, weights, cancelled)? {
        return Ok((solution, true));
    }
    let restarts = (300_000 / problem.blocks.len()).clamp(8, 128);
    let mut candidates = parallel(restarts, cancelled, |start| {
        placement::improve(problem, weights, start, cancelled)
    })?;
    candidates.sort_by(|a, b| a.cost.total_cmp(&b.cost));
    // Reevaluate several promising partitions with all candidate sites in
    // each group when that fits within the work budget.
    if problem
        .blocks
        .len()
        .saturating_mul(problem.blocks.len().div_ceil(problem.wells))
        <= 2_000_000
        && problem.wells <= 24
    {
        let polished = parallel(candidates.len().min(8), cancelled, |index| {
            placement::polish(problem, weights, &candidates[index], cancelled)
        })?;
        candidates.extend(polished);
    }
    Ok((
        candidates
            .into_iter()
            .min_by(|a, b| a.cost.total_cmp(&b.cost))
            .unwrap(),
        false,
    ))
}

fn check_cancelled(cancelled: &AtomicBool) -> Result<(), String> {
    if cancelled.load(Relaxed) {
        Err("Расчёт отменён.".into())
    } else {
        Ok(())
    }
}

fn parallel<T: Send>(
    count: usize,
    cancelled: &AtomicBool,
    job: impl Fn(usize) -> Result<T, String> + Sync,
) -> Result<Vec<T>, String> {
    let workers = thread::available_parallelism()
        .map_or(1, usize::from)
        .min(8)
        .min(count);
    if workers <= 1 {
        return (0..count)
            .map(|index| {
                check_cancelled(cancelled)?;
                job(index)
            })
            .collect();
    }
    let next = AtomicUsize::new(0);
    thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|_| {
                scope.spawn(|| {
                    let mut results = Vec::new();
                    loop {
                        check_cancelled(cancelled)?;
                        let index = next.fetch_add(1, Relaxed);
                        if index >= count {
                            return Ok::<_, String>(results);
                        }
                        results.push((index, job(index)?));
                    }
                })
            })
            .collect();
        let mut indexed = Vec::with_capacity(count);
        for handle in handles {
            indexed.extend(handle.join().map_err(|_| "Ошибка потока расчёта.")??);
        }
        indexed.sort_unstable_by_key(|(index, _)| *index);
        Ok(indexed.into_iter().map(|(_, value)| value).collect())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(reserves: Vec<Option<f64>>, wells: u32, epsilon: f64) -> Project {
        Project {
            well_count: wells,
            scenarios: 8,
            epsilon,
            grid: Some(Grid {
                rows: 1,
                columns: reserves.len(),
                x_min: 0.0,
                x_max: (reserves.len() - 1) as f64,
                y_min: 0.0,
                y_max: 0.0,
                values: reserves,
            }),
            ..Project::default()
        }
    }

    #[test]
    fn interval_optimum_uses_the_common_worst_weights() {
        let project = line(vec![Some(1.0); 3], 1, 0.6);
        let cancelled = AtomicBool::new(false);
        let (wells, result) = calculate(&project, Algorithm::Interval, &cancelled).unwrap();
        assert_eq!(wells, ["1:2"]);
        assert_eq!(result.worst_penalty, 2.0);
        assert_eq!(result.nominal_penalty, 2.0);
        assert!(result.optimal);

        let project = line(vec![Some(1.0), Some(0.0), None, Some(0.25)], 1, 1.0);
        let problem = Problem::new(project.grid.as_ref().unwrap(), 1, project.gamma).unwrap();
        assert_eq!(problem.worst_weights(1.0), [1.0, 1.0]);
        let result = calculate(&project, Algorithm::Scenarios, &cancelled)
            .unwrap()
            .1;
        assert!(result.worst_penalty >= result.nominal_penalty);
    }

    #[test]
    fn exhaustive_search_runs_beyond_the_automatic_work_limit() {
        let project = Project {
            gamma: 1.0,
            ..line(vec![Some(1.0); 32], 2, 0.2)
        };
        let cancelled = AtomicBool::new(false);
        let problem = Problem::new(project.grid.as_ref().unwrap(), 2, project.gamma).unwrap();
        assert!(
            exact::solve(
                &problem,
                &problem.worst_weights(project.epsilon),
                &cancelled
            )
            .unwrap()
            .is_none()
        );
        let (wells, result) = calculate(&project, Algorithm::Exhaustive, &cancelled).unwrap();
        assert_eq!(wells, ["1:8", "1:24"]);
        assert_eq!(result.worst_penalty, 128.0);
        assert!(result.optimal);
        cancelled.store(true, Relaxed);
        assert!(calculate(&project, Algorithm::Exhaustive, &cancelled).is_err());
    }

    #[test]
    fn local_search_uses_geometric_distance_and_reserve_weights() {
        let project = Project {
            gamma: 0.0,
            ..line(
                vec![Some(0.0), None, Some(5.0), Some(0.0), Some(2.0)],
                1,
                0.0,
            )
        };
        let (wells, result) = calculate(&project, Algorithm::LocalSearch, &false.into()).unwrap();
        assert_eq!(wells, ["1:3"]);
        assert!((result.tail_penalty.unwrap() - 4.0 / 7.0).abs() < 1e-12);
        assert!((result.nominal_penalty - 0.4).abs() < 1e-12);
    }

    #[test]
    fn all_algorithms_skip_zero_and_missing_reserves() {
        let project = Project {
            gamma: 1.0,
            ..line(
                vec![Some(1.0), Some(0.0), None, Some(-0.0), Some(1.0), Some(1.0)],
                1,
                0.0,
            )
        };
        for algorithm in [
            Algorithm::Scenarios,
            Algorithm::Interval,
            Algorithm::Exhaustive,
            Algorithm::LocalSearch,
        ] {
            let (wells, result) = calculate(&project, algorithm, &false.into()).unwrap();
            assert_eq!(wells, ["1:5"], "{algorithm:?}");
            assert_eq!(result.worst_penalty, 5.0, "{algorithm:?}");
            assert!(result.positive_reserves_only);
            assert_eq!(
                result.well_by_cell.as_deref(),
                Some([1, 0, 0, 0, 1, 1].as_slice())
            );
            assert!(
                calculate(
                    &line(vec![Some(0.0), None, Some(0.0)], 1, 0.0),
                    algorithm,
                    &false.into()
                )
                .is_err()
            );
        }
    }

    #[test]
    fn rounded_capacity_is_feasible_in_both_searches() {
        let project = line(vec![Some(1.0); 7], 3, 0.2);
        let cancelled = AtomicBool::new(false);
        let problem = Problem::new(project.grid.as_ref().unwrap(), 3, project.gamma).unwrap();
        let weights = problem.worst_weights(project.epsilon);
        let exact = exact::solve(&problem, &weights, &cancelled)
            .unwrap()
            .unwrap();
        assert!((exact.cost - 4.0).abs() < 1e-10);
        let (wells, result) = calculate(&project, Algorithm::Exhaustive, &cancelled).unwrap();
        let assigned = result.well_by_cell.unwrap();
        let mut sizes = Vec::new();
        for (index, well) in wells.iter().enumerate() {
            let column = well.split_once(':').unwrap().1.parse::<usize>().unwrap();
            let number = index as u32 + 1;
            assert_eq!(assigned[column - 1], number);
            sizes.push(assigned.iter().filter(|&&owner| owner == number).count());
        }
        sizes.sort_unstable();
        assert_eq!(sizes, [2, 2, 3]);
        for candidate in std::iter::once(exact).chain(
            (0..8).map(|start| placement::improve(&problem, &weights, start, &cancelled).unwrap()),
        ) {
            let mut sizes: Vec<_> = candidate
                .sites
                .iter()
                .map(|&site| {
                    assert_eq!(candidate.assignment[site], site);
                    candidate
                        .assignment
                        .iter()
                        .filter(|&&assigned| assigned == site)
                        .count()
                })
                .collect();
            sizes.sort_unstable();
            assert_eq!(sizes, [2, 2, 3]);
        }
    }
}
