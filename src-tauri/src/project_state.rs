use crate::project::Project;
use serde::Serialize;
use std::{path::PathBuf, sync::Mutex};

#[derive(Default)]
pub(crate) struct ProjectState {
    pub(crate) project: Project,
    pub(crate) path: Option<PathBuf>,
    pub(crate) dirty: bool,
}
#[derive(Default)]
pub(crate) struct AppState(pub(crate) Mutex<ProjectState>);
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Snapshot {
    project: Project,
    path: Option<PathBuf>,
    dirty: bool,
    warning: Option<String>,
}
pub(crate) fn snapshot(state: &ProjectState, warning: Option<String>) -> Snapshot {
    Snapshot {
        project: state.project.clone(),
        path: state.path.clone(),
        dirty: state.dirty,
        warning,
    }
}
