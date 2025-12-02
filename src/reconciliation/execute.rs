use crate::manifest::{
    add_file_to_manifest, create_managed_file, create_manifest, read_manifest, write_manifest,
    CentyManifest, ManagedFileType,
};
use crate::utils::{compute_hash, get_centy_path};
use super::managed_files::get_managed_files;
use super::plan::{build_reconciliation_plan, ReconciliationPlan};
use std::collections::HashSet;
use std::path::Path;
use thiserror::Error;
use tokio::fs;

#[derive(Error, Debug)]
pub enum ExecuteError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Manifest error: {0}")]
    ManifestError(#[from] crate::manifest::ManifestError),

    #[error("Plan error: {0}")]
    PlanError(#[from] super::plan::PlanError),
}

/// User decisions for reconciliation
#[derive(Debug, Clone, Default)]
pub struct ReconciliationDecisions {
    /// Paths of files to restore
    pub restore: HashSet<String>,
    /// Paths of files to reset
    pub reset: HashSet<String>,
}

/// Result of reconciliation execution
#[derive(Debug, Clone, Default)]
pub struct ReconciliationResult {
    pub created: Vec<String>,
    pub restored: Vec<String>,
    pub reset: Vec<String>,
    pub skipped: Vec<String>,
    pub manifest: CentyManifest,
}

/// Execute the reconciliation plan
pub async fn execute_reconciliation(
    project_path: &Path,
    decisions: ReconciliationDecisions,
    force: bool,
) -> Result<ReconciliationResult, ExecuteError> {
    let centy_path = get_centy_path(project_path);
    let managed_templates = get_managed_files();

    // Create .centy directory if it doesn't exist
    if !centy_path.exists() {
        fs::create_dir_all(&centy_path).await?;
    }

    // Get or create manifest
    let mut manifest = read_manifest(project_path)
        .await?
        .unwrap_or_else(create_manifest);

    // Build the plan
    let plan = build_reconciliation_plan(project_path).await?;

    let mut result = ReconciliationResult::default();

    // Process files to create
    for file_info in &plan.to_create {
        create_file(&centy_path, &file_info.path, &managed_templates, &mut manifest).await?;
        result.created.push(file_info.path.clone());
    }

    // Process files to restore
    for file_info in &plan.to_restore {
        if force || decisions.restore.contains(&file_info.path) {
            create_file(&centy_path, &file_info.path, &managed_templates, &mut manifest).await?;
            result.restored.push(file_info.path.clone());
        } else {
            result.skipped.push(file_info.path.clone());
        }
    }

    // Process files to reset
    for file_info in &plan.to_reset {
        if decisions.reset.contains(&file_info.path) {
            create_file(&centy_path, &file_info.path, &managed_templates, &mut manifest).await?;
            result.reset.push(file_info.path.clone());
        } else {
            // Keep existing file but ensure it's in manifest
            if let Some(template) = managed_templates.get(&file_info.path) {
                let managed_file = create_managed_file(
                    file_info.path.clone(),
                    file_info.hash.clone(),
                    template.file_type.clone(),
                );
                add_file_to_manifest(&mut manifest, managed_file);
            }
            result.skipped.push(file_info.path.clone());
        }
    }

    // Ensure up-to-date files are in manifest
    for file_info in &plan.up_to_date {
        if let Some(template) = managed_templates.get(&file_info.path) {
            let hash = template
                .content
                .as_ref()
                .map(|c| compute_hash(c))
                .unwrap_or_default();

            let managed_file = create_managed_file(
                file_info.path.clone(),
                hash,
                template.file_type.clone(),
            );
            add_file_to_manifest(&mut manifest, managed_file);
        }
    }

    // Write manifest
    write_manifest(project_path, &manifest).await?;

    result.manifest = manifest;
    Ok(result)
}

/// Create a file or directory from template
async fn create_file(
    centy_path: &Path,
    relative_path: &str,
    templates: &std::collections::HashMap<String, super::managed_files::ManagedFileTemplate>,
    manifest: &mut CentyManifest,
) -> Result<(), ExecuteError> {
    let template = templates
        .get(relative_path)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Template not found"))?;

    let full_path = centy_path.join(relative_path.trim_end_matches('/'));

    match &template.file_type {
        ManagedFileType::Directory => {
            fs::create_dir_all(&full_path).await?;

            let managed_file = create_managed_file(
                relative_path.to_string(),
                String::new(),
                ManagedFileType::Directory,
            );
            add_file_to_manifest(manifest, managed_file);
        }
        ManagedFileType::File => {
            // Ensure parent directory exists
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).await?;
            }

            let content = template.content.as_deref().unwrap_or("");
            fs::write(&full_path, content).await?;

            let hash = compute_hash(content);
            let managed_file = create_managed_file(
                relative_path.to_string(),
                hash,
                ManagedFileType::File,
            );
            add_file_to_manifest(manifest, managed_file);
        }
    }

    Ok(())
}
