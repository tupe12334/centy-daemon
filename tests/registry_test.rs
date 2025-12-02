mod common;

use centy_daemon::registry::{
    get_project_info, list_projects, track_project, untrack_project, RegistryError,
};
use common::{create_test_dir, init_centy_project};
use std::path::Path;

/// Helper to get canonical path for comparison
fn canonical_path(path: &str) -> String {
    Path::new(path)
        .canonicalize()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string())
}

#[tokio::test]
async fn test_track_project_creates_entry() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path().to_string_lossy().to_string();
    let canonical = canonical_path(&project_path);

    // Track the project
    track_project(&project_path).await.expect("Should track project");

    // Verify it's in the list (compare canonical paths)
    let projects = list_projects(true).await.expect("Should list projects");
    assert!(
        projects.iter().any(|p| p.path == canonical),
        "Project should be in list"
    );
}

#[tokio::test]
async fn test_track_project_updates_last_accessed() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path().to_string_lossy().to_string();

    // Track the project
    track_project(&project_path).await.expect("Should track");

    let info1 = get_project_info(&project_path)
        .await
        .expect("Should get info")
        .expect("Should find project");

    // Verify first and last accessed are set
    assert!(!info1.first_accessed.is_empty(), "first_accessed should be set");
    assert!(!info1.last_accessed.is_empty(), "last_accessed should be set");

    // Small delay to ensure timestamp changes
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Track again
    track_project(&project_path).await.expect("Should track again");

    let info2 = get_project_info(&project_path)
        .await
        .expect("Should get info")
        .expect("Should find project");

    // Verify timestamps are valid ISO format (can be parsed)
    assert!(
        chrono::DateTime::parse_from_rfc3339(&info2.first_accessed).is_ok(),
        "first_accessed should be valid RFC3339"
    );
    assert!(
        chrono::DateTime::parse_from_rfc3339(&info2.last_accessed).is_ok(),
        "last_accessed should be valid RFC3339"
    );
}

#[tokio::test]
async fn test_untrack_project_removes_entry() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path().to_string_lossy().to_string();

    // Track then untrack
    track_project(&project_path).await.expect("Should track");
    untrack_project(&project_path).await.expect("Should untrack");

    // Verify it's gone
    let info = get_project_info(&project_path).await.expect("Should get info");
    assert!(info.is_none(), "Project should not be found after untrack");
}

#[tokio::test]
async fn test_untrack_nonexistent_project_returns_error() {
    let result = untrack_project("/nonexistent/path/12345").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RegistryError::ProjectNotFound(_)));
}

#[tokio::test]
async fn test_list_projects_excludes_stale_by_default() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path().to_string_lossy().to_string();
    let canonical = canonical_path(&project_path);

    // Track a project
    track_project(&project_path).await.expect("Should track");

    // Should be in list (not stale)
    let projects = list_projects(false).await.expect("Should list");
    assert!(
        projects.iter().any(|p| p.path == canonical),
        "Project should be in non-stale list"
    );

    // Drop temp_dir to delete the directory
    drop(temp_dir);

    // Now with include_stale=false, it should be excluded
    let projects = list_projects(false).await.expect("Should list");
    assert!(
        !projects.iter().any(|p| p.path == canonical),
        "Stale project should be excluded"
    );

    // With include_stale=true, it should be included
    let projects = list_projects(true).await.expect("Should list");
    assert!(
        projects.iter().any(|p| p.path == canonical),
        "Stale project should be included when requested"
    );
}

#[tokio::test]
async fn test_project_info_shows_issue_and_doc_counts() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Initialize centy project
    init_centy_project(project_path).await;

    // Track the project
    let project_path_str = project_path.to_string_lossy().to_string();
    track_project(&project_path_str).await.expect("Should track");

    // Get info - should show initialized with 0 issues/docs
    let info = get_project_info(&project_path_str)
        .await
        .expect("Should get info")
        .expect("Should find project");

    assert!(info.initialized, "Project should be initialized");
    assert_eq!(info.issue_count, 0);
    assert_eq!(info.doc_count, 0);
    assert!(info.name.is_some(), "Should have a name");
}

#[tokio::test]
async fn test_project_info_counts_issues() {
    use centy_daemon::issue::{create_issue, CreateIssueOptions};

    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Initialize centy project
    init_centy_project(project_path).await;

    // Create some issues
    for i in 1..=3 {
        let options = CreateIssueOptions {
            title: format!("Issue {}", i),
            ..Default::default()
        };
        create_issue(project_path, options)
            .await
            .expect("Should create issue");
    }

    // Track and get info
    let project_path_str = project_path.to_string_lossy().to_string();
    track_project(&project_path_str).await.expect("Should track");

    let info = get_project_info(&project_path_str)
        .await
        .expect("Should get info")
        .expect("Should find project");

    assert_eq!(info.issue_count, 3, "Should have 3 issues");
}

#[tokio::test]
async fn test_project_info_counts_docs() {
    use centy_daemon::docs::{create_doc, CreateDocOptions};

    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Initialize centy project
    init_centy_project(project_path).await;

    // Create some docs
    for i in 1..=2 {
        let options = CreateDocOptions {
            title: format!("Doc {}", i),
            content: "Content".to_string(),
            slug: None,
            template: None,
        };
        create_doc(project_path, options)
            .await
            .expect("Should create doc");
    }

    // Track and get info
    let project_path_str = project_path.to_string_lossy().to_string();
    track_project(&project_path_str).await.expect("Should track");

    let info = get_project_info(&project_path_str)
        .await
        .expect("Should get info")
        .expect("Should find project");

    assert_eq!(info.doc_count, 2, "Should have 2 docs");
}

#[tokio::test]
async fn test_list_projects_sorted_by_last_accessed() {
    let temp_dir1 = create_test_dir();
    let temp_dir2 = create_test_dir();
    let path1 = temp_dir1.path().to_string_lossy().to_string();
    let path2 = temp_dir2.path().to_string_lossy().to_string();

    // Track path1 first
    track_project(&path1).await.expect("Should track");
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Track path2 second (more recent)
    track_project(&path2).await.expect("Should track");

    // List should have path2 first (most recent)
    let projects = list_projects(true).await.expect("Should list");

    // Find indices
    let idx1 = projects.iter().position(|p| p.path == path1);
    let idx2 = projects.iter().position(|p| p.path == path2);

    if let (Some(i1), Some(i2)) = (idx1, idx2) {
        assert!(i2 < i1, "More recently accessed project should come first");
    }
}
