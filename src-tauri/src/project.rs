use crate::calculation::Calculation;
use serde::{Deserialize, Serialize};

pub(crate) const MAX_CELLS: usize = 1_000_000;

fn default_tail_percent() -> u32 {
    10
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Grid {
    pub rows: usize,
    pub columns: usize,
    pub values: Vec<Option<f64>>,
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

impl Grid {
    pub(crate) fn calculation_blocks(&self) -> impl Iterator<Item = (usize, f64)> + '_ {
        self.values
            .iter()
            .enumerate()
            .filter_map(|(index, &value)| {
                value
                    .filter(|reserve| *reserve > 0.0)
                    .map(|reserve| (index, reserve))
            })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Project {
    pub format_version: u32,
    pub grid: Option<Grid>,
    pub well_count: u32,
    pub scenarios: u32,
    pub epsilon: f64,
    pub gamma: f64,
    #[serde(default = "default_tail_percent")]
    pub tail_percent: u32,
    pub calculated_wells: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calculation: Option<Calculation>,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            format_version: 1,
            grid: None,
            well_count: 5,
            scenarios: 100,
            epsilon: 0.2,
            gamma: 0.5,
            tail_percent: default_tail_percent(),
            calculated_wells: None,
            calculation: None,
        }
    }
}

impl Project {
    pub fn validate(&self) -> Result<(), String> {
        if self.format_version != 1 {
            return Err("Неподдерживаемая версия проекта.".into());
        }
        if self.well_count == 0
            || self.scenarios == 0
            || !self.epsilon.is_finite()
            || !(0.0..=1.0).contains(&self.epsilon)
            || !self.gamma.is_finite()
            || !(0.0..=1.0).contains(&self.gamma)
            || !(1..=100).contains(&self.tail_percent)
        {
            return Err("Некорректные параметры проекта.".into());
        }
        if let Some(result) = &self.calculation
            && (self
                .calculated_wells
                .as_ref()
                .is_none_or(|wells| wells.len() != self.well_count as usize)
                || [result.nominal_penalty, result.worst_penalty]
                    .iter()
                    .any(|penalty| !penalty.is_finite() || *penalty < 0.0)
                || result
                    .tail_penalty
                    .is_some_and(|penalty| !penalty.is_finite() || penalty < 0.0))
        {
            return Err("Некорректный результат расчёта.".into());
        }
        if let Some(grid) = &self.grid {
            let count = grid
                .rows
                .checked_mul(grid.columns)
                .ok_or("Некорректный размер сетки.")?;
            if count == 0
                || count > MAX_CELLS
                || grid.values.len() != count
                || ![grid.x_min, grid.x_max, grid.y_min, grid.y_max]
                    .iter()
                    .all(|v| v.is_finite())
                || grid.x_max < grid.x_min
                || grid.y_max < grid.y_min
                || (grid.columns > 1 && grid.x_max == grid.x_min)
                || (grid.rows > 1 && grid.y_max == grid.y_min)
                || grid.values.iter().flatten().any(|v| !v.is_finite())
            {
                return Err("Некорректная сетка проекта.".into());
            }
        }
        if let Some(owners) = self
            .calculation
            .as_ref()
            .and_then(|result| result.well_by_cell.as_deref())
        {
            let grid = self
                .grid
                .as_ref()
                .ok_or("Для распределения блоков отсутствует сетка.")?;
            let wells = self.calculated_wells.as_deref().unwrap_or_default();
            if owners.len() != grid.values.len()
                || owners.iter().zip(&grid.values).any(|(&owner, &reserve)| {
                    owner > self.well_count
                        || (owner > 0) != reserve.is_some_and(|value| value > 0.0)
                })
                || !wells.iter().enumerate().all(|(index, position)| {
                    let Some((row, column)) = position.split_once(':').and_then(|(row, column)| {
                        Some((row.parse::<usize>().ok()?, column.parse::<usize>().ok()?))
                    }) else {
                        return false;
                    };
                    (1..=grid.rows).contains(&row)
                        && (1..=grid.columns).contains(&column)
                        && owners[(row - 1) * grid.columns + column - 1] as usize == index + 1
                })
            {
                return Err("Некорректное распределение блоков по скважинам.".into());
            }
        }
        Ok(())
    }
}
