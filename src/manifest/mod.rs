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
/// Files are sorted by path to ensure deterministic output
pub async fn write_manifest(
    project_path: &Path,
    manifest: &CentyManifest,
) -> Result<(), ManifestError> {
    let manifest_path = get_manifest_path(project_path);

    // Create a copy with sorted files for deterministic output
    let mut sorted_manifest = manifest.clone();
    sorted_manifest
        .managed_files
        .sort_by(|a, b| a.path.cmp(&b.path));

    let content = serde_json::to_string_pretty(&sorted_manifest)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_manifest() {
        let manifest = create_manifest();

        assert_eq!(manifest.schema_version, 1);
        assert_eq!(manifest.centy_version, CENTY_VERSION);
        assert!(manifest.managed_files.is_empty());
        assert!(!manifest.created_at.is_empty());
        assert!(!manifest.updated_at.is_empty());
    }

    #[test]
    fn test_create_managed_file() {
        let file = create_managed_file(
            "test.md".to_string(),
            "abc123".to_string(),
            ManagedFileType::File,
        );

        assert_eq!(file.path, "test.md");
        assert_eq!(file.hash, "abc123");
        assert_eq!(file.version, CENTY_VERSION);
        assert_eq!(file.file_type, ManagedFileType::File);
        assert!(!file.created_at.is_empty());
    }

    #[test]
    fn test_create_managed_directory() {
        let dir = create_managed_file(
            "issues/".to_string(),
            String::new(),
            ManagedFileType::Directory,
        );

        assert_eq!(dir.path, "issues/");
        assert_eq!(dir.hash, "");
        assert_eq!(dir.file_type, ManagedFileType::Directory);
    }

    #[test]
    fn test_add_file_to_manifest() {
        let mut manifest = create_manifest();
        let file = create_managed_file(
            "README.md".to_string(),
            "hash1".to_string(),
            ManagedFileType::File,
        );

        add_file_to_manifest(&mut manifest, file);

        assert_eq!(manifest.managed_files.len(), 1);
        assert_eq!(manifest.managed_files[0].path, "README.md");
    }

    #[test]
    fn test_add_file_to_manifest_updates_existing() {
        let mut manifest = create_manifest();

        // Add initial file
        let file1 = create_managed_file(
            "README.md".to_string(),
            "hash1".to_string(),
            ManagedFileType::File,
        );
        add_file_to_manifest(&mut manifest, file1);

        // Add file with same path but different hash
        let file2 = create_managed_file(
            "README.md".to_string(),
            "hash2".to_string(),
            ManagedFileType::File,
        );
        add_file_to_manifest(&mut manifest, file2);

        // Should still have only one file
        assert_eq!(manifest.managed_files.len(), 1);
        assert_eq!(manifest.managed_files[0].hash, "hash2");
    }

    #[test]
    fn test_add_multiple_files_to_manifest() {
        let mut manifest = create_manifest();

        add_file_to_manifest(
            &mut manifest,
            create_managed_file("file1.md".to_string(), "h1".to_string(), ManagedFileType::File),
        );
        add_file_to_manifest(
            &mut manifest,
            create_managed_file("file2.md".to_string(), "h2".to_string(), ManagedFileType::File),
        );
        add_file_to_manifest(
            &mut manifest,
            create_managed_file("dir/".to_string(), String::new(), ManagedFileType::Directory),
        );

        assert_eq!(manifest.managed_files.len(), 3);
    }

    #[test]
    fn test_find_managed_file_exists() {
        let mut manifest = create_manifest();
        add_file_to_manifest(
            &mut manifest,
            create_managed_file("README.md".to_string(), "hash".to_string(), ManagedFileType::File),
        );

        let found = find_managed_file(&manifest, "README.md");
        assert!(found.is_some());
        assert_eq!(found.unwrap().path, "README.md");
    }

    #[test]
    fn test_find_managed_file_not_exists() {
        let manifest = create_manifest();
        let found = find_managed_file(&manifest, "nonexistent.md");
        assert!(found.is_none());
    }

    #[test]
    fn test_manifest_serialization() {
        let mut manifest = create_manifest();
        add_file_to_manifest(
            &mut manifest,
            create_managed_file("test.md".to_string(), "hash".to_string(), ManagedFileType::File),
        );

        // Serialize
        let json = serde_json::to_string(&manifest).expect("Should serialize");

        // Deserialize
        let deserialized: CentyManifest = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(manifest.schema_version, deserialized.schema_version);
        assert_eq!(manifest.centy_version, deserialized.centy_version);
        assert_eq!(manifest.managed_files.len(), deserialized.managed_files.len());
    }

    #[test]
    fn test_manifest_json_uses_camel_case() {
        let manifest = create_manifest();
        let json = serde_json::to_string(&manifest).expect("Should serialize");

        // Check for camelCase keys
        assert!(json.contains("schemaVersion"));
        assert!(json.contains("centyVersion"));
        assert!(json.contains("createdAt"));
        assert!(json.contains("updatedAt"));
        assert!(json.contains("managedFiles"));

        // Should NOT contain snake_case
        assert!(!json.contains("schema_version"));
        assert!(!json.contains("centy_version"));
    }

    #[tokio::test]
    async fn test_write_manifest_sorts_files_by_path() {
        use tempfile::tempdir;

        // Create a manifest with files in random order
        let mut manifest = create_manifest();
        add_file_to_manifest(
            &mut manifest,
            create_managed_file("z-file.md".to_string(), "h1".to_string(), ManagedFileType::File),
        );
        add_file_to_manifest(
            &mut manifest,
            create_managed_file("a-file.md".to_string(), "h2".to_string(), ManagedFileType::File),
        );
        add_file_to_manifest(
            &mut manifest,
            create_managed_file("m-file.md".to_string(), "h3".to_string(), ManagedFileType::File),
        );

        // Create temp directory and write manifest
        let temp_dir = tempdir().expect("Should create temp dir");
        let centy_dir = temp_dir.path().join(".centy");
        fs::create_dir_all(&centy_dir)
            .await
            .expect("Should create .centy dir");

        write_manifest(temp_dir.path(), &manifest)
            .await
            .expect("Should write manifest");

        // Read back and verify files are sorted
        let read_manifest = read_manifest(temp_dir.path())
            .await
            .expect("Should read manifest")
            .expect("Manifest should exist");

        assert_eq!(read_manifest.managed_files.len(), 3);
        assert_eq!(read_manifest.managed_files[0].path, "a-file.md");
        assert_eq!(read_manifest.managed_files[1].path, "m-file.md");
        assert_eq!(read_manifest.managed_files[2].path, "z-file.md");
    }
}
