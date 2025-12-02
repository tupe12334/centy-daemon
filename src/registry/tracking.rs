use super::storage::{get_lock, read_registry, write_registry_unlocked};
use super::types::{ProjectInfo, TrackedProject};
use super::RegistryError;
use crate::utils::{get_centy_path, now_iso};
use std::path::Path;
use tokio::fs;
use tracing::warn;

/// Track a project access - called on any RPC operation
/// Updates last_accessed timestamp, creates new entry if not exists
pub async fn track_project(project_path: &str) -> Result<(), RegistryError> {
    let path = Path::new(project_path);

    // Canonicalize path to ensure consistent keys
    let canonical_path = path
        .canonicalize()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| project_path.to_string());

    // Lock the entire read-modify-write cycle to prevent race conditions
    let _guard = get_lock().lock().await;

    let mut registry = read_registry().await?;
    let now = now_iso();

    if let Some(entry) = registry.projects.get_mut(&canonical_path) {
        // Update existing entry
        entry.last_accessed = now.clone();
    } else {
        // Create new entry
        let entry = TrackedProject {
            first_accessed: now.clone(),
            last_accessed: now.clone(),
        };
        registry.projects.insert(canonical_path, entry);
    }

    registry.updated_at = now;
    write_registry_unlocked(&registry).await?;

    Ok(())
}

/// Track project access asynchronously (fire-and-forget)
/// Failures are logged but don't block the main operation
pub fn track_project_async(project_path: String) {
    tokio::spawn(async move {
        if let Err(e) = track_project(&project_path).await {
            warn!("Failed to track project {}: {}", project_path, e);
        }
    });
}

/// Remove a project from tracking
pub async fn untrack_project(project_path: &str) -> Result<(), RegistryError> {
    let path = Path::new(project_path);

    // Try canonical path first, fall back to original
    let canonical_path = path
        .canonicalize()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| project_path.to_string());

    // Lock the entire read-modify-write cycle to prevent race conditions
    let _guard = get_lock().lock().await;

    let mut registry = read_registry().await?;

    // Try to remove by canonical path first
    if registry.projects.remove(&canonical_path).is_none() {
        // If not found, try original path
        if registry.projects.remove(project_path).is_none() {
            return Err(RegistryError::ProjectNotFound(project_path.to_string()));
        }
    }

    registry.updated_at = now_iso();
    write_registry_unlocked(&registry).await?;

    Ok(())
}

/// Enrich a tracked project with live data from disk
pub async fn enrich_project(path: &str, tracked: &TrackedProject) -> ProjectInfo {
    let project_path = Path::new(path);
    let centy_path = get_centy_path(project_path);

    // Check if initialized (manifest exists)
    let manifest_path = centy_path.join(".centy-manifest.json");
    let initialized = manifest_path.exists();

    // Count issues (directories in .centy/issues/)
    let issues_path = centy_path.join("issues");
    let issue_count = count_directories(&issues_path).await.unwrap_or(0);

    // Count docs (markdown files in .centy/docs/)
    let docs_path = centy_path.join("docs");
    let doc_count = count_md_files(&docs_path).await.unwrap_or(0);

    // Get project name (directory name)
    let name = project_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string());

    ProjectInfo {
        path: path.to_string(),
        first_accessed: tracked.first_accessed.clone(),
        last_accessed: tracked.last_accessed.clone(),
        issue_count,
        doc_count,
        initialized,
        name,
    }
}

/// List all tracked projects, enriched with live data
pub async fn list_projects(include_stale: bool) -> Result<Vec<ProjectInfo>, RegistryError> {
    let registry = read_registry().await?;

    let mut projects = Vec::new();

    for (path, tracked) in &registry.projects {
        let path_exists = Path::new(path).exists();

        if !include_stale && !path_exists {
            // Skip stale (non-existent) projects
            continue;
        }

        let info = enrich_project(path, tracked).await;
        projects.push(info);
    }

    // Sort by last_accessed (most recent first)
    projects.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));

    Ok(projects)
}

/// Get info for a specific project
pub async fn get_project_info(project_path: &str) -> Result<Option<ProjectInfo>, RegistryError> {
    let path = Path::new(project_path);

    // Canonicalize path
    let canonical_path = path
        .canonicalize()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| project_path.to_string());

    let registry = read_registry().await?;

    // Try canonical path first, then original
    let tracked = registry
        .projects
        .get(&canonical_path)
        .or_else(|| registry.projects.get(project_path));

    match tracked {
        Some(tracked) => Ok(Some(enrich_project(&canonical_path, tracked).await)),
        None => Ok(None),
    }
}

/// Count directories in a path (for counting issues)
async fn count_directories(path: &Path) -> Result<u32, std::io::Error> {
    if !path.exists() {
        return Ok(0);
    }

    let mut count = 0;
    let mut entries = fs::read_dir(path).await?;

    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_dir() {
            count += 1;
        }
    }

    Ok(count)
}

/// Count markdown files in a path (for counting docs)
async fn count_md_files(path: &Path) -> Result<u32, std::io::Error> {
    if !path.exists() {
        return Ok(0);
    }

    let mut count = 0;
    let mut entries = fs::read_dir(path).await?;

    while let Some(entry) = entries.next_entry().await? {
        let file_type = entry.file_type().await?;
        if file_type.is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "md" {
                    count += 1;
                }
            }
        }
    }

    Ok(count)
}
