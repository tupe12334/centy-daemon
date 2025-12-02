mod hash;

pub use hash::{compute_hash, compute_file_hash};

use std::path::Path;

/// The name of the centy folder
pub const CENTY_FOLDER: &str = ".centy";

/// The name of the manifest file
pub const MANIFEST_FILE: &str = ".centy-manifest.json";

/// Current centy version
pub const CENTY_VERSION: &str = "0.1.0";

/// Get the path to the .centy folder
pub fn get_centy_path(project_path: &Path) -> std::path::PathBuf {
    project_path.join(CENTY_FOLDER)
}

/// Get the path to the manifest file
pub fn get_manifest_path(project_path: &Path) -> std::path::PathBuf {
    get_centy_path(project_path).join(MANIFEST_FILE)
}

/// Get current timestamp in ISO 8601 format
pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}
