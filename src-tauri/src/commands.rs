use crate::{
    cps, project_file,
    project_state::{AppState, Snapshot, snapshot},
    settings::{Settings, save_last_path, settings_path},
};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::State;
use tauri_plugin_fs::FsExt;

fn warning(result: Result<(), String>) -> Option<String> {
    result
        .err()
        .map(|e| format!("Не удалось сохранить путь последнего проекта: {e}"))
}
fn selected_path(app: &tauri::AppHandle, path: &Path) -> Result<(), String> {
    if app.fs_scope().is_allowed(path) {
        Ok(())
    } else {
        Err("Доступ к файлу не разрешён. Выберите его через диалог.".into())
    }
}
#[tauri::command]
pub(crate) async fn restore_project(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Snapshot, String> {
    let path = settings_path(&app)?;
    let mut state = state.0.lock().map_err(|e| e.to_string())?;
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(snapshot(&state, None)),
        Err(e) => {
            return Ok(snapshot(
                &state,
                Some(format!("Не удалось прочитать настройки: {e}")),
            ));
        }
    };
    let settings: Settings = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(e) => {
            return Ok(snapshot(
                &state,
                Some(format!("Не удалось прочитать настройки: {e}")),
            ));
        }
    };
    if let Some(path) = settings.last_project_path {
        match project_file::read_project(&path) {
            Ok(project) => {
                state.project = project;
                state.path = Some(path);
                state.dirty = false;
            }
            Err(e) => {
                return Ok(snapshot(
                    &state,
                    Some(format!("Не удалось открыть последний проект: {e}")),
                ));
            }
        }
    }
    Ok(snapshot(&state, None))
}
#[tauri::command]
pub(crate) async fn import_cps(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    path: PathBuf,
) -> Result<Snapshot, String> {
    selected_path(&app, &path)?;
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let grid = cps::parse_cps(&text)?;
    let mut state = state.0.lock().map_err(|e| e.to_string())?;
    let count = grid.values.iter().filter(|v| v.is_some()).count();
    state.project.grid = Some(grid);
    state.project.well_count = state
        .project
        .well_count
        .min(count.saturating_sub(1).max(1) as u32);
    state.project.calculated_wells = None;
    state.dirty = true;
    Ok(snapshot(&state, None))
}
#[tauri::command]
pub(crate) async fn open_project(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    path: PathBuf,
) -> Result<Snapshot, String> {
    selected_path(&app, &path)?;
    let project = project_file::read_project(&path)?;
    let mut state = state.0.lock().map_err(|e| e.to_string())?;
    state.project = project;
    state.path = Some(path.clone());
    state.dirty = false;
    Ok(snapshot(&state, warning(save_last_path(&app, &path))))
}
#[tauri::command]
pub(crate) async fn save_project(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    path: Option<PathBuf>,
) -> Result<Snapshot, String> {
    if let Some(path) = &path {
        selected_path(&app, path)?;
    }
    let mut state = state.0.lock().map_err(|e| e.to_string())?;
    let path = path
        .or_else(|| state.path.clone())
        .ok_or("Выберите файл проекта.")?;
    project_file::write_project(&path, &state.project)?;
    state.path = Some(path.clone());
    state.dirty = false;
    Ok(snapshot(&state, warning(save_last_path(&app, &path))))
}
#[tauri::command]
pub(crate) async fn update_setting(
    state: State<'_, AppState>,
    key: &str,
    value: f64,
) -> Result<Snapshot, String> {
    if !value.is_finite() {
        return Err("Некорректное значение параметра.".into());
    }
    let mut state = state.0.lock().map_err(|e| e.to_string())?;
    match key {
        "wellCount" => {
            let max = state
                .project
                .grid
                .as_ref()
                .map(|g| {
                    g.values
                        .iter()
                        .filter(|v| v.is_some())
                        .count()
                        .saturating_sub(1)
                        .max(1)
                })
                .unwrap_or(1);
            state.project.well_count = value.trunc().clamp(1.0, max as f64) as u32;
        }
        "scenarios" => state.project.scenarios = value.trunc().clamp(1.0, u32::MAX as f64) as u32,
        "epsilon" => state.project.epsilon = value.clamp(0.0, 1.0),
        "gamma" => state.project.gamma = value.clamp(0.0, 1.0),
        _ => return Err("Неизвестный параметр.".into()),
    }
    state.project.calculated_wells = None;
    state.dirty = true;
    Ok(snapshot(&state, None))
}
