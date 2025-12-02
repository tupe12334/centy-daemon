mod storage;
mod tracking;
mod types;

pub use storage::{get_registry_path, read_registry, write_registry};
pub use tracking::{
    enrich_project, get_project_info, list_projects, track_project, track_project_async,
    untrack_project,
};
pub use types::{ProjectInfo, ProjectRegistry, TrackedProject};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Failed to determine home directory")]
    HomeDirNotFound,

    #[error("Project not found in registry: {0}")]
    ProjectNotFound(String),
}
