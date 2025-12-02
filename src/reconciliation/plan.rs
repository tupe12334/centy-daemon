use crate::manifest::{read_manifest, find_managed_file, ManagedFileType};
use crate::utils::{compute_file_hash, compute_hash, get_centy_path};
use super::managed_files::get_managed_files;
use std::collections::HashSet;
use std::path::Path;
use thiserror::Error;
use tokio::fs;
use walkdir::WalkDir;

#[derive(Error, Debug)]
pub enum PlanError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Manifest error: {0}")]
    ManifestError(#[from] crate::manifest::ManifestError),
}

/// Information about a file in the reconciliation plan
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub file_type: ManagedFileType,
    pub hash: String,
    pub content_preview: Option<String>,
}

/// The reconciliation plan
#[derive(Debug, Clone, Default)]
pub struct ReconciliationPlan {
    /// Files that need to be created (not on disk, not in manifest)
    pub to_create: Vec<FileInfo>,

    /// Files that were deleted but exist in manifest (can be restored)
    pub to_restore: Vec<FileInfo>,

    /// Files that were modified from original (hash mismatch)
    pub to_reset: Vec<FileInfo>,

    /// Files that are up to date
    pub up_to_date: Vec<FileInfo>,

    /// User-created files (not managed by centy)
    pub user_files: Vec<FileInfo>,
}

impl ReconciliationPlan {
    /// Check if user decisions are needed
    pub fn needs_decisions(&self) -> bool {
        !self.to_restore.is_empty() || !self.to_reset.is_empty()
    }
}

/// Build a reconciliation plan for the given project path
pub async fn build_reconciliation_plan(project_path: &Path) -> Result<ReconciliationPlan, PlanError> {
    let centy_path = get_centy_path(project_path);
    let managed_templates = get_managed_files();
    let manifest = read_manifest(project_path).await?;

    let mut plan = ReconciliationPlan::default();

    // Get set of files on disk
    let files_on_disk = scan_centy_folder(&centy_path).await;

    // Get set of managed file paths
    let managed_paths: HashSet<String> = managed_templates.keys().cloned().collect();

    // Check each managed file template
    for (path, template) in &managed_templates {
        let full_path = centy_path.join(path.trim_end_matches('/'));
        let exists_on_disk = files_on_disk.contains(path);

        // Check manifest for this file
        let in_manifest = manifest
            .as_ref()
            .and_then(|m| find_managed_file(m, path));

        let file_info = FileInfo {
            path: path.clone(),
            file_type: template.file_type.clone(),
            hash: template
                .content
                .as_ref()
                .map(|c| compute_hash(c))
                .unwrap_or_default(),
            content_preview: template.content.as_ref().map(|c| {
                c.chars().take(100).collect::<String>()
            }),
        };

        if !exists_on_disk {
            if in_manifest.is_some() {
                // File was in manifest but deleted - can be restored
                plan.to_restore.push(file_info);
            } else {
                // File never existed - needs to be created
                plan.to_create.push(file_info);
            }
        } else {
            // File exists on disk
            match &template.file_type {
                ManagedFileType::Directory => {
                    // Directories are always considered up to date if they exist
                    plan.up_to_date.push(file_info);
                }
                ManagedFileType::File => {
                    // Check if content matches
                    if let Some(expected_content) = &template.content {
                        let expected_hash = compute_hash(expected_content);
                        let actual_hash = compute_file_hash(&full_path).await.unwrap_or_default();

                        if actual_hash == expected_hash {
                            plan.up_to_date.push(file_info);
                        } else {
                            // File was modified
                            plan.to_reset.push(FileInfo {
                                hash: actual_hash,
                                ..file_info
                            });
                        }
                    } else {
                        plan.up_to_date.push(file_info);
                    }
                }
            }
        }
    }

    // Find user-created files (files on disk not in managed templates)
    for disk_path in &files_on_disk {
        if !managed_paths.contains(disk_path) {
            let full_path = centy_path.join(disk_path.trim_end_matches('/'));
            let is_dir = full_path.is_dir();

            let hash = if !is_dir {
                compute_file_hash(&full_path).await.unwrap_or_default()
            } else {
                String::new()
            };

            plan.user_files.push(FileInfo {
                path: disk_path.clone(),
                file_type: if is_dir {
                    ManagedFileType::Directory
                } else {
                    ManagedFileType::File
                },
                hash,
                content_preview: None,
            });
        }
    }

    Ok(plan)
}

/// Scan the .centy folder and return relative paths of all files/directories
async fn scan_centy_folder(centy_path: &Path) -> HashSet<String> {
    let mut files = HashSet::new();

    if !centy_path.exists() {
        return files;
    }

    for entry in WalkDir::new(centy_path)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip the manifest file
        if path.file_name().map(|f| f.to_str()) == Some(Some(".centy-manifest.json")) {
            continue;
        }

        if let Ok(relative) = path.strip_prefix(centy_path) {
            let mut relative_str = relative.to_string_lossy().to_string();

            // Add trailing slash for directories
            if path.is_dir() {
                relative_str.push('/');
            }

            files.insert(relative_str);
        }
    }

    files
}
