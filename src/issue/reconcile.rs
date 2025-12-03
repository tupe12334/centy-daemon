//! Display number reconciliation for resolving conflicts.
//!
//! When multiple users create issues offline, they may assign the same display
//! number. This module detects and resolves such conflicts by:
//! 1. Keeping the oldest issue's display number (by created_at)
//! 2. Reassigning newer issues to the next available number

use super::id::is_valid_issue_folder;
use super::metadata::IssueMetadata;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;
use tokio::fs;

#[derive(Error, Debug)]
pub enum ReconcileError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Information about an issue needed for reconciliation
#[derive(Debug, Clone)]
struct IssueInfo {
    folder_name: String,
    display_number: u32,
    created_at: String,
}

/// Reconcile display numbers to resolve conflicts.
///
/// This function scans all issues, finds duplicate display numbers, and
/// reassigns them so each issue has a unique display number. The oldest
/// issue (by created_at) keeps its original number.
///
/// Returns the number of issues that were reassigned.
pub async fn reconcile_display_numbers(issues_path: &Path) -> Result<u32, ReconcileError> {
    if !issues_path.exists() {
        return Ok(0);
    }

    // Step 1: Read all issues and their display numbers
    let mut issues: Vec<IssueInfo> = Vec::new();
    let mut entries = fs::read_dir(issues_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        if !entry.file_type().await?.is_dir() {
            continue;
        }

        let folder_name = match entry.file_name().to_str() {
            Some(name) => name.to_string(),
            None => continue,
        };

        if !is_valid_issue_folder(&folder_name) {
            continue;
        }

        let metadata_path = entry.path().join("metadata.json");
        if !metadata_path.exists() {
            continue;
        }

        let content = fs::read_to_string(&metadata_path).await?;
        let metadata: IssueMetadata = match serde_json::from_str(&content) {
            Ok(m) => m,
            Err(_) => continue, // Skip malformed metadata
        };

        issues.push(IssueInfo {
            folder_name,
            display_number: metadata.display_number,
            created_at: metadata.created_at,
        });
    }

    // Step 2: Find duplicates (group by display_number)
    let mut by_display_number: HashMap<u32, Vec<&IssueInfo>> = HashMap::new();
    for issue in &issues {
        by_display_number
            .entry(issue.display_number)
            .or_default()
            .push(issue);
    }

    // Step 3: Find max display number for reassignment
    let max_display_number = issues
        .iter()
        .map(|i| i.display_number)
        .max()
        .unwrap_or(0);

    // Step 4: Process duplicates
    let mut reassignments: Vec<(String, u32)> = Vec::new(); // (folder_name, new_display_number)
    let mut next_available = max_display_number + 1;

    for (display_number, mut group) in by_display_number {
        if group.len() <= 1 {
            continue; // No conflict
        }

        // Skip display_number 0 (legacy issues without display numbers)
        if display_number == 0 {
            // Assign each legacy issue a unique number
            for issue in &group {
                reassignments.push((issue.folder_name.clone(), next_available));
                next_available += 1;
            }
            continue;
        }

        // Sort by created_at (oldest first)
        group.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        // Keep the first (oldest), reassign the rest
        for issue in group.iter().skip(1) {
            reassignments.push((issue.folder_name.clone(), next_available));
            next_available += 1;
        }
    }

    // Step 5: Write reassignments
    let reassignment_count = reassignments.len() as u32;

    for (folder_name, new_display_number) in reassignments {
        let metadata_path = issues_path.join(&folder_name).join("metadata.json");
        let content = fs::read_to_string(&metadata_path).await?;
        let mut metadata: IssueMetadata = serde_json::from_str(&content)?;

        metadata.display_number = new_display_number;
        metadata.updated_at = crate::utils::now_iso();

        let new_content = serde_json::to_string_pretty(&metadata)?;
        fs::write(&metadata_path, new_content).await?;
    }

    Ok(reassignment_count)
}

/// Get the next available display number.
///
/// Scans all existing issues and returns max + 1.
pub async fn get_next_display_number(issues_path: &Path) -> Result<u32, ReconcileError> {
    if !issues_path.exists() {
        return Ok(1);
    }

    let mut max_number: u32 = 0;
    let mut entries = fs::read_dir(issues_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        if !entry.file_type().await?.is_dir() {
            continue;
        }

        let folder_name = match entry.file_name().to_str() {
            Some(name) => name.to_string(),
            None => continue,
        };

        if !is_valid_issue_folder(&folder_name) {
            continue;
        }

        let metadata_path = entry.path().join("metadata.json");
        if !metadata_path.exists() {
            continue;
        }

        if let Ok(content) = fs::read_to_string(&metadata_path).await {
            if let Ok(metadata) = serde_json::from_str::<IssueMetadata>(&content) {
                max_number = max_number.max(metadata.display_number);
            }
        }
    }

    Ok(max_number + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_issue(
        issues_path: &Path,
        folder_name: &str,
        display_number: u32,
        created_at: &str,
    ) {
        let issue_path = issues_path.join(folder_name);
        fs::create_dir_all(&issue_path).await.unwrap();

        let metadata = serde_json::json!({
            "displayNumber": display_number,
            "status": "open",
            "priority": 2,
            "createdAt": created_at,
            "updatedAt": created_at
        });

        fs::write(
            issue_path.join("metadata.json"),
            serde_json::to_string_pretty(&metadata).unwrap(),
        )
        .await
        .unwrap();

        fs::write(issue_path.join("issue.md"), "# Test Issue\n")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_reconcile_no_conflicts() {
        let temp = TempDir::new().unwrap();
        let issues_path = temp.path().join("issues");
        fs::create_dir_all(&issues_path).await.unwrap();

        create_test_issue(
            &issues_path,
            "550e8400-e29b-41d4-a716-446655440001",
            1,
            "2024-01-01T10:00:00Z",
        )
        .await;
        create_test_issue(
            &issues_path,
            "550e8400-e29b-41d4-a716-446655440002",
            2,
            "2024-01-01T11:00:00Z",
        )
        .await;

        let reassigned = reconcile_display_numbers(&issues_path).await.unwrap();
        assert_eq!(reassigned, 0);
    }

    #[tokio::test]
    async fn test_reconcile_with_conflict() {
        let temp = TempDir::new().unwrap();
        let issues_path = temp.path().join("issues");
        fs::create_dir_all(&issues_path).await.unwrap();

        // Both have display_number 4, but different created_at
        create_test_issue(
            &issues_path,
            "550e8400-e29b-41d4-a716-446655440001",
            4,
            "2024-01-01T10:00:00Z", // Older
        )
        .await;
        create_test_issue(
            &issues_path,
            "550e8400-e29b-41d4-a716-446655440002",
            4,
            "2024-01-01T10:05:00Z", // Newer
        )
        .await;
        create_test_issue(
            &issues_path,
            "550e8400-e29b-41d4-a716-446655440003",
            5,
            "2024-01-01T10:10:00Z",
        )
        .await;

        let reassigned = reconcile_display_numbers(&issues_path).await.unwrap();
        assert_eq!(reassigned, 1);

        // Check the older one kept display_number 4
        let metadata1: IssueMetadata = serde_json::from_str(
            &fs::read_to_string(
                issues_path
                    .join("550e8400-e29b-41d4-a716-446655440001")
                    .join("metadata.json"),
            )
            .await
            .unwrap(),
        )
        .unwrap();
        assert_eq!(metadata1.display_number, 4);

        // Check the newer one was reassigned to 6 (max was 5, so next is 6)
        let metadata2: IssueMetadata = serde_json::from_str(
            &fs::read_to_string(
                issues_path
                    .join("550e8400-e29b-41d4-a716-446655440002")
                    .join("metadata.json"),
            )
            .await
            .unwrap(),
        )
        .unwrap();
        assert_eq!(metadata2.display_number, 6);
    }

    #[tokio::test]
    async fn test_get_next_display_number_empty() {
        let temp = TempDir::new().unwrap();
        let issues_path = temp.path().join("issues");

        let next = get_next_display_number(&issues_path).await.unwrap();
        assert_eq!(next, 1);
    }

    #[tokio::test]
    async fn test_get_next_display_number_with_existing() {
        let temp = TempDir::new().unwrap();
        let issues_path = temp.path().join("issues");
        fs::create_dir_all(&issues_path).await.unwrap();

        create_test_issue(
            &issues_path,
            "550e8400-e29b-41d4-a716-446655440001",
            5,
            "2024-01-01T10:00:00Z",
        )
        .await;

        let next = get_next_display_number(&issues_path).await.unwrap();
        assert_eq!(next, 6);
    }
}
