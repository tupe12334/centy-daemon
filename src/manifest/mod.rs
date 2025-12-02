mod types;

pub use types::{CentyManifest, ManagedFile, ManagedFileType};

use crate::utils::{get_manifest_path, now_iso, CENTY_VERSION};
use std::path::Path;
use thiserror::Error;
use tokio::fs;

#[derive(Error, Debug)]
pub enum ManifestError {
    #[error("Failed to read manifest: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Failed to parse manifest: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Manifest not found at {0}")]
    NotFound(String),
}

/// Read the manifest from the project path
pub async fn read_manifest(project_path: &Path) -> Result<Option<CentyManifest>, ManifestError> {
    let manifest_path = get_manifest_path(project_path);

    if !manifest_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&manifest_path).await?;
    let manifest: CentyManifest = serde_json::from_str(&content)?;
    Ok(Some(manifest))
}

/// Write the manifest to the project path
pub async fn write_manifest(
    project_path: &Path,
    manifest: &CentyManifest,
) -> Result<(), ManifestError> {
    let manifest_path = get_manifest_path(project_path);
    let content = serde_json::to_string_pretty(manifest)?;
    fs::write(&manifest_path, content).await?;
    Ok(())
}

/// Create a new empty manifest
pub fn create_manifest() -> CentyManifest {
    let now = now_iso();
    CentyManifest {
        schema_version: 1,
        centy_version: CENTY_VERSION.to_string(),
        created_at: now.clone(),
        updated_at: now,
        managed_files: Vec::new(),
    }
}

/// Add or update a file in the manifest
pub fn add_file_to_manifest(manifest: &mut CentyManifest, file: ManagedFile) {
    // Remove existing entry if present
    manifest.managed_files.retain(|f| f.path != file.path);
    // Add the new entry
    manifest.managed_files.push(file);
    // Update timestamp
    manifest.updated_at = now_iso();
}

/// Find a managed file by path
pub fn find_managed_file<'a>(manifest: &'a CentyManifest, path: &str) -> Option<&'a ManagedFile> {
    manifest.managed_files.iter().find(|f| f.path == path)
}

/// Create a ManagedFile entry
pub fn create_managed_file(
    path: String,
    hash: String,
    file_type: ManagedFileType,
) -> ManagedFile {
    let now = now_iso();
    ManagedFile {
        path,
        hash,
        version: CENTY_VERSION.to_string(),
        created_at: now,
        file_type,
    }
}
