use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use tauri::Manager;

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Settings {
    pub(crate) last_project_path: Option<PathBuf>,
}
pub(crate) fn settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?
        .join("settings.json"))
}
pub(crate) fn save_last_path(app: &tauri::AppHandle, path: &Path) -> Result<(), String> {
    let settings = settings_path(app)?;
    let dir = settings.parent().ok_or("Не найден каталог настроек.")?;
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let mut file = tempfile::NamedTempFile::new_in(dir).map_err(|e| e.to_string())?;
    serde_json::to_writer(
        &mut file,
        &Settings {
            last_project_path: Some(path.to_path_buf()),
        },
    )
    .map_err(|e| e.to_string())?;
    file.flush().map_err(|e| e.to_string())?;
    file.persist(settings).map_err(|e| e.error.to_string())?;
    Ok(())
}
