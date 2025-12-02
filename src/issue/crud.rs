use crate::config::read_config;
use crate::manifest::{
    add_file_to_manifest, create_managed_file, read_manifest, write_manifest, CentyManifest,
    ManagedFileType,
};
use crate::utils::{compute_hash, get_centy_path, now_iso};
use super::metadata::IssueMetadata;
use super::priority::{validate_priority, PriorityError};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;
use tokio::fs;

#[derive(Error, Debug)]
pub enum IssueCrudError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Manifest error: {0}")]
    ManifestError(#[from] crate::manifest::ManifestError),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Centy not initialized. Run 'centy init' first.")]
    NotInitialized,

    #[error("Issue {0} not found")]
    IssueNotFound(String),

    #[error("Invalid issue format: {0}")]
    InvalidIssueFormat(String),

    #[error("Invalid priority: {0}")]
    InvalidPriority(#[from] PriorityError),
}

/// Full issue data
#[derive(Debug, Clone)]
pub struct Issue {
    pub issue_number: String,
    pub title: String,
    pub description: String,
    pub metadata: IssueMetadataFlat,
}

/// Flattened metadata for API responses
#[derive(Debug, Clone)]
pub struct IssueMetadataFlat {
    pub status: String,
    /// Priority as a number (1 = highest, N = lowest)
    pub priority: u32,
    pub created_at: String,
    pub updated_at: String,
    pub custom_fields: HashMap<String, String>,
}

/// Options for updating an issue
#[derive(Debug, Clone, Default)]
pub struct UpdateIssueOptions {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    /// Priority as a number (1 = highest). None = don't update.
    pub priority: Option<u32>,
    pub custom_fields: HashMap<String, String>,
}

/// Result of issue update
#[derive(Debug, Clone)]
pub struct UpdateIssueResult {
    pub issue: Issue,
    pub manifest: CentyManifest,
}

/// Result of issue deletion
#[derive(Debug, Clone)]
pub struct DeleteIssueResult {
    pub manifest: CentyManifest,
}

/// Get a single issue by its number
pub async fn get_issue(
    project_path: &Path,
    issue_number: &str,
) -> Result<Issue, IssueCrudError> {
    // Check if centy is initialized
    read_manifest(project_path)
        .await?
        .ok_or(IssueCrudError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let issue_path = centy_path.join("issues").join(issue_number);

    if !issue_path.exists() {
        return Err(IssueCrudError::IssueNotFound(issue_number.to_string()));
    }

    read_issue_from_disk(&issue_path, issue_number).await
}

/// List all issues with optional filtering
pub async fn list_issues(
    project_path: &Path,
    status_filter: Option<&str>,
    priority_filter: Option<u32>,
) -> Result<Vec<Issue>, IssueCrudError> {
    // Check if centy is initialized
    read_manifest(project_path)
        .await?
        .ok_or(IssueCrudError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let issues_path = centy_path.join("issues");

    if !issues_path.exists() {
        return Ok(Vec::new());
    }

    let mut issues = Vec::new();
    let mut entries = fs::read_dir(&issues_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_dir() {
            if let Some(issue_number) = entry.file_name().to_str() {
                // Skip non-numeric directories
                if issue_number.parse::<u32>().is_err() {
                    continue;
                }

                match read_issue_from_disk(&entry.path(), issue_number).await {
                    Ok(issue) => {
                        // Apply filters
                        let status_match = status_filter
                            .map(|s| issue.metadata.status == s)
                            .unwrap_or(true);
                        let priority_match = priority_filter
                            .map(|p| issue.metadata.priority == p)
                            .unwrap_or(true);

                        if status_match && priority_match {
                            issues.push(issue);
                        }
                    }
                    Err(_) => {
                        // Skip issues that can't be read
                        continue;
                    }
                }
            }
        }
    }

    // Sort by issue number
    issues.sort_by(|a, b| a.issue_number.cmp(&b.issue_number));

    Ok(issues)
}

/// Update an existing issue
pub async fn update_issue(
    project_path: &Path,
    issue_number: &str,
    options: UpdateIssueOptions,
) -> Result<UpdateIssueResult, IssueCrudError> {
    // Check if centy is initialized
    let mut manifest = read_manifest(project_path)
        .await?
        .ok_or(IssueCrudError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let issue_path = centy_path.join("issues").join(issue_number);

    if !issue_path.exists() {
        return Err(IssueCrudError::IssueNotFound(issue_number.to_string()));
    }

    // Read config for priority_levels validation
    let config = read_config(project_path).await.ok().flatten();
    let priority_levels = config.as_ref().map(|c| c.priority_levels).unwrap_or(3);

    // Read current issue
    let current = read_issue_from_disk(&issue_path, issue_number).await?;

    // Apply updates
    let new_title = options.title.unwrap_or(current.title);
    let new_description = options.description.unwrap_or(current.description);
    let new_status = options.status.unwrap_or(current.metadata.status);

    // Validate and apply priority update
    let new_priority = match options.priority {
        Some(p) => {
            validate_priority(p, priority_levels)?;
            p
        }
        None => current.metadata.priority,
    };

    // Merge custom fields
    let mut new_custom_fields = current.metadata.custom_fields;
    for (key, value) in options.custom_fields {
        new_custom_fields.insert(key, value);
    }

    // Create updated metadata
    let updated_metadata = IssueMetadata {
        status: new_status.clone(),
        priority: new_priority,
        created_at: current.metadata.created_at.clone(),
        updated_at: now_iso(),
        custom_fields: new_custom_fields
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
            .collect(),
    };

    // Generate updated content
    let issue_md = generate_issue_md(&new_title, &new_description);

    // Write files
    let issue_md_path = issue_path.join("issue.md");
    let metadata_path = issue_path.join("metadata.json");

    fs::write(&issue_md_path, &issue_md).await?;
    fs::write(&metadata_path, serde_json::to_string_pretty(&updated_metadata)?).await?;

    // Update manifest
    let base_path = format!("issues/{}/", issue_number);

    add_file_to_manifest(
        &mut manifest,
        create_managed_file(
            format!("{}issue.md", base_path),
            compute_hash(&issue_md),
            ManagedFileType::File,
        ),
    );

    let metadata_json = serde_json::to_string_pretty(&updated_metadata)?;
    add_file_to_manifest(
        &mut manifest,
        create_managed_file(
            format!("{}metadata.json", base_path),
            compute_hash(&metadata_json),
            ManagedFileType::File,
        ),
    );

    write_manifest(project_path, &manifest).await?;

    let issue = Issue {
        issue_number: issue_number.to_string(),
        title: new_title,
        description: new_description,
        metadata: IssueMetadataFlat {
            status: new_status,
            priority: new_priority,
            created_at: current.metadata.created_at,
            updated_at: updated_metadata.updated_at,
            custom_fields: new_custom_fields,
        },
    };

    Ok(UpdateIssueResult { issue, manifest })
}

/// Delete an issue
pub async fn delete_issue(
    project_path: &Path,
    issue_number: &str,
) -> Result<DeleteIssueResult, IssueCrudError> {
    // Check if centy is initialized
    let mut manifest = read_manifest(project_path)
        .await?
        .ok_or(IssueCrudError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let issue_path = centy_path.join("issues").join(issue_number);

    if !issue_path.exists() {
        return Err(IssueCrudError::IssueNotFound(issue_number.to_string()));
    }

    // Remove the issue directory
    fs::remove_dir_all(&issue_path).await?;

    // Remove from manifest
    let base_path = format!("issues/{}/", issue_number);
    manifest.managed_files.retain(|f| !f.path.starts_with(&base_path));
    manifest.updated_at = now_iso();

    write_manifest(project_path, &manifest).await?;

    Ok(DeleteIssueResult { manifest })
}

/// Read an issue from disk
async fn read_issue_from_disk(issue_path: &Path, issue_number: &str) -> Result<Issue, IssueCrudError> {
    let issue_md_path = issue_path.join("issue.md");
    let metadata_path = issue_path.join("metadata.json");

    if !issue_md_path.exists() || !metadata_path.exists() {
        return Err(IssueCrudError::InvalidIssueFormat(format!(
            "Issue {} is missing required files",
            issue_number
        )));
    }

    // Read issue.md
    let issue_md = fs::read_to_string(&issue_md_path).await?;
    let (title, description) = parse_issue_md(&issue_md);

    // Read metadata (serde will auto-migrate string priorities to numbers)
    let metadata_content = fs::read_to_string(&metadata_path).await?;
    let metadata: IssueMetadata = serde_json::from_str(&metadata_content)?;

    // Convert custom fields to strings
    let custom_fields: HashMap<String, String> = metadata
        .custom_fields
        .into_iter()
        .map(|(k, v)| {
            let str_val = match v {
                serde_json::Value::String(s) => s,
                other => other.to_string(),
            };
            (k, str_val)
        })
        .collect();

    Ok(Issue {
        issue_number: issue_number.to_string(),
        title,
        description,
        metadata: IssueMetadataFlat {
            status: metadata.status,
            priority: metadata.priority,
            created_at: metadata.created_at,
            updated_at: metadata.updated_at,
            custom_fields,
        },
    })
}

/// Parse issue.md content to extract title and description
fn parse_issue_md(content: &str) -> (String, String) {
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        return (String::new(), String::new());
    }

    // First line should be the title (# Title)
    let title = lines[0]
        .strip_prefix('#')
        .map(|s| s.trim())
        .unwrap_or(lines[0])
        .to_string();

    // Rest is description (skip empty lines after title)
    let description_lines: Vec<&str> = lines[1..]
        .iter()
        .skip_while(|line| line.is_empty())
        .copied()
        .collect();

    let description = description_lines.join("\n").trim_end().to_string();

    (title, description)
}

/// Generate the issue markdown content
fn generate_issue_md(title: &str, description: &str) -> String {
    if description.is_empty() {
        format!("# {}\n", title)
    } else {
        format!("# {}\n\n{}\n", title, description)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_issue_md_with_description() {
        let content = "# My Issue Title\n\nThis is the description.\nWith multiple lines.";
        let (title, description) = parse_issue_md(content);
        assert_eq!(title, "My Issue Title");
        assert_eq!(description, "This is the description.\nWith multiple lines.");
    }

    #[test]
    fn test_parse_issue_md_title_only() {
        let content = "# My Issue Title\n";
        let (title, description) = parse_issue_md(content);
        assert_eq!(title, "My Issue Title");
        assert_eq!(description, "");
    }

    #[test]
    fn test_parse_issue_md_empty() {
        let content = "";
        let (title, description) = parse_issue_md(content);
        assert_eq!(title, "");
        assert_eq!(description, "");
    }
}
