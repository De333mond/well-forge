use crate::project::{Grid, MAX_CELLS, Project};

pub fn parse_cps(text: &str) -> Result<Grid, String> {
    let (header, data) = text
        .split_once("->Surface")
        .ok_or("Не найден раздел Surface.")?;
    if data.contains("->Surface") {
        return Err("Найдено несколько разделов Surface.".into());
    }
    let mut rows: usize = 0;
    let mut columns: usize = 0;
    let mut missing = None;
    let mut bounds = None;
    for line in header.lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        match fields.first().copied() {
            Some("FSNROW") if fields.len() >= 3 => {
                rows = fields[1].parse().map_err(|_| "Некорректное число строк.")?;
                columns = fields[2]
                    .parse()
                    .map_err(|_| "Некорректное число столбцов.")?;
            }
            Some("FSASCI") if fields.len() >= 6 => {
                missing = Some(
                    fields[5]
                        .parse::<f64>()
                        .map_err(|_| "Некорректный маркер пустого значения.")?,
                );
            }
            Some("FSLIMI") if fields.len() >= 5 => {
                bounds = Some((
                    fields[1]
                        .parse::<f64>()
                        .map_err(|_| "Некорректные координаты.")?,
                    fields[2]
                        .parse::<f64>()
                        .map_err(|_| "Некорректные координаты.")?,
                    fields[3]
                        .parse::<f64>()
                        .map_err(|_| "Некорректные координаты.")?,
                    fields[4]
                        .parse::<f64>()
                        .map_err(|_| "Некорректные координаты.")?,
                ));
            }
            _ => {}
        }
    }
    let missing = missing
        .filter(|v| v.is_finite())
        .ok_or("Отсутствует маркер пустого значения.")?;
    let (x_min, x_max, y_min, y_max) = bounds.ok_or("Отсутствуют координаты сетки.")?;
    let count = rows
        .checked_mul(columns)
        .filter(|n| *n > 0 && *n <= MAX_CELLS)
        .ok_or("Некорректный размер сетки.")?;
    let mut values = vec![None; count];
    let mut length = 0;
    for token in data.split_whitespace() {
        if length >= count {
            return Err("Слишком много значений в файле.".into());
        }
        let value: f64 = token
            .parse()
            .map_err(|_| format!("Некорректное значение в позиции {}.", length + 1))?;
        if !value.is_finite() {
            return Err("Некорректное значение сетки.".into());
        }
        let column = length / rows;
        let row = rows - 1 - length % rows;
        values[row * columns + column] = if value == missing { None } else { Some(value) };
        length += 1;
    }
    if length != count {
        return Err(format!("Ожидается {count} значений, найдено {length}."));
    }
    let grid = Grid {
        rows,
        columns,
        values,
        x_min,
        x_max,
        y_min,
        y_max,
    };
    Project {
        grid: Some(grid.clone()),
        ..Project::default()
    }
    .validate()?;
    Ok(grid)
}
