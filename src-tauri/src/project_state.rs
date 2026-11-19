use crate::project::Project;
use serde::Serialize;
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering::Relaxed},
    },
};

#[derive(Default)]
pub(crate) struct ProjectState {
    pub(crate) project: Project,
    pub(crate) path: Option<PathBuf>,
    pub(crate) dirty: bool,
    pub(crate) calculation: Option<Arc<AtomicBool>>,
}

impl ProjectState {
    pub(crate) fn cancel_calculation(&self) {
        if let Some(cancelled) = &self.calculation {
            cancelled.store(true, Relaxed);
        }
    }

    pub(crate) fn invalidate_result(&mut self) {
        self.cancel_calculation();
        self.project.calculated_wells = None;
        self.project.calculation = None;
        self.dirty = true;
    }
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
