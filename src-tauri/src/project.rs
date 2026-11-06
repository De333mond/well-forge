use serde::{Deserialize, Serialize};

pub(crate) const MAX_CELLS: usize = 1_000_000;

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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Project {
    pub format_version: u32,
    pub grid: Option<Grid>,
    pub well_count: u32,
    pub scenarios: u32,
    pub epsilon: f64,
    pub gamma: f64,
    pub calculated_wells: Option<Vec<String>>,
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
            calculated_wells: None,
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
        {
            return Err("Некорректные параметры проекта.".into());
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
        Ok(())
    }
}
