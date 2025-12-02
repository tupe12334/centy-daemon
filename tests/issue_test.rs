mod common;

use centy_daemon::issue::{
    create_issue, delete_issue, get_issue, list_issues, update_issue, CreateIssueOptions,
    IssueError, IssueCrudError, UpdateIssueOptions,
};
use common::{create_test_dir, init_centy_project};
use std::collections::HashMap;

#[tokio::test]
async fn test_create_issue_success() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Initialize centy first
    init_centy_project(project_path).await;

    // Create an issue with numeric priority (1 = highest)
    let options = CreateIssueOptions {
        title: "Test Issue".to_string(),
        description: "This is a test issue".to_string(),
        priority: Some(1), // high priority
        status: Some("open".to_string()),
        custom_fields: HashMap::new(),
        ..Default::default()
    };

    let result = create_issue(project_path, options)
        .await
        .expect("Should create issue");

    assert_eq!(result.issue_number, "0001");
    assert_eq!(result.created_files.len(), 3); // issue.md, metadata.json, assets/

    // Verify files exist
    let issue_path = project_path.join(".centy/issues/0001");
    assert!(issue_path.join("issue.md").exists());
    assert!(issue_path.join("metadata.json").exists());
    assert!(issue_path.join("assets").exists());
}

#[tokio::test]
async fn test_create_issue_increments_number() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create first issue
    let options1 = CreateIssueOptions {
        title: "First Issue".to_string(),
        ..Default::default()
    };
    let result1 = create_issue(project_path, options1).await.expect("Should create");
    assert_eq!(result1.issue_number, "0001");

    // Create second issue
    let options2 = CreateIssueOptions {
        title: "Second Issue".to_string(),
        ..Default::default()
    };
    let result2 = create_issue(project_path, options2).await.expect("Should create");
    assert_eq!(result2.issue_number, "0002");

    // Create third issue
    let options3 = CreateIssueOptions {
        title: "Third Issue".to_string(),
        ..Default::default()
    };
    let result3 = create_issue(project_path, options3).await.expect("Should create");
    assert_eq!(result3.issue_number, "0003");
}

#[tokio::test]
async fn test_create_issue_requires_init() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    // Don't initialize - try to create issue
    let options = CreateIssueOptions {
        title: "Test Issue".to_string(),
        ..Default::default()
    };

    let result = create_issue(project_path, options).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), IssueError::NotInitialized));
}

#[tokio::test]
async fn test_create_issue_requires_title() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Try to create issue without title
    let options = CreateIssueOptions {
        title: "".to_string(),
        ..Default::default()
    };

    let result = create_issue(project_path, options).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), IssueError::TitleRequired));
}

#[tokio::test]
async fn test_create_issue_default_priority_and_status() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create issue without specifying priority/status
    let options = CreateIssueOptions {
        title: "Test Issue".to_string(),
        ..Default::default()
    };

    create_issue(project_path, options).await.expect("Should create");

    // Get the issue and verify defaults
    // Default priority with 3 levels (high/medium/low) is 2 (medium)
    let issue = get_issue(project_path, "0001").await.expect("Should get issue");
    assert_eq!(issue.metadata.priority, 2); // medium
    assert_eq!(issue.metadata.status, "open");
}

#[tokio::test]
async fn test_get_issue_success() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create an issue with numeric priority
    let options = CreateIssueOptions {
        title: "My Test Issue".to_string(),
        description: "Description here".to_string(),
        priority: Some(1), // high
        status: Some("in-progress".to_string()),
        custom_fields: HashMap::new(),
        ..Default::default()
    };
    create_issue(project_path, options).await.expect("Should create");

    // Get the issue
    let issue = get_issue(project_path, "0001").await.expect("Should get issue");

    assert_eq!(issue.issue_number, "0001");
    assert_eq!(issue.title, "My Test Issue");
    assert_eq!(issue.description, "Description here");
    assert_eq!(issue.metadata.priority, 1); // high
    assert_eq!(issue.metadata.status, "in-progress");
}

#[tokio::test]
async fn test_get_issue_not_found() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let result = get_issue(project_path, "9999").await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        IssueCrudError::IssueNotFound(_)
    ));
}

#[tokio::test]
async fn test_list_issues_empty() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let issues = list_issues(project_path, None, None)
        .await
        .expect("Should list issues");

    assert!(issues.is_empty());
}

#[tokio::test]
async fn test_list_issues_returns_all() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create multiple issues
    for i in 1..=3 {
        let options = CreateIssueOptions {
            title: format!("Issue {}", i),
            ..Default::default()
        };
        create_issue(project_path, options).await.expect("Should create");
    }

    let issues = list_issues(project_path, None, None)
        .await
        .expect("Should list issues");

    assert_eq!(issues.len(), 3);
    // Should be sorted by issue number
    assert_eq!(issues[0].issue_number, "0001");
    assert_eq!(issues[1].issue_number, "0002");
    assert_eq!(issues[2].issue_number, "0003");
}

#[tokio::test]
async fn test_list_issues_filter_by_status() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create issues with different statuses
    create_issue(
        project_path,
        CreateIssueOptions {
            title: "Open Issue".to_string(),
            status: Some("open".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    create_issue(
        project_path,
        CreateIssueOptions {
            title: "Closed Issue".to_string(),
            status: Some("closed".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Filter by status
    let open_issues = list_issues(project_path, Some("open"), None)
        .await
        .expect("Should list");
    assert_eq!(open_issues.len(), 1);
    assert_eq!(open_issues[0].title, "Open Issue");

    let closed_issues = list_issues(project_path, Some("closed"), None)
        .await
        .expect("Should list");
    assert_eq!(closed_issues.len(), 1);
    assert_eq!(closed_issues[0].title, "Closed Issue");
}

#[tokio::test]
async fn test_list_issues_filter_by_priority() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create issues with different priorities (numeric)
    create_issue(
        project_path,
        CreateIssueOptions {
            title: "High Priority".to_string(),
            priority: Some(1), // high
            ..Default::default()
        },
    )
    .await
    .unwrap();

    create_issue(
        project_path,
        CreateIssueOptions {
            title: "Low Priority".to_string(),
            priority: Some(3), // low
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Filter by priority (numeric)
    let high_issues = list_issues(project_path, None, Some(1))
        .await
        .expect("Should list");
    assert_eq!(high_issues.len(), 1);
    assert_eq!(high_issues[0].title, "High Priority");
}

#[tokio::test]
async fn test_update_issue_title() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create issue
    create_issue(
        project_path,
        CreateIssueOptions {
            title: "Original Title".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Update title
    let options = UpdateIssueOptions {
        title: Some("Updated Title".to_string()),
        ..Default::default()
    };

    let result = update_issue(project_path, "0001", options)
        .await
        .expect("Should update");

    assert_eq!(result.issue.title, "Updated Title");

    // Verify persisted
    let issue = get_issue(project_path, "0001").await.unwrap();
    assert_eq!(issue.title, "Updated Title");
}

#[tokio::test]
async fn test_update_issue_status() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    create_issue(
        project_path,
        CreateIssueOptions {
            title: "Test".to_string(),
            status: Some("open".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Update status
    let result = update_issue(
        project_path,
        "0001",
        UpdateIssueOptions {
            status: Some("closed".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect("Should update");

    assert_eq!(result.issue.metadata.status, "closed");
}

#[tokio::test]
async fn test_update_issue_preserves_unchanged_fields() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create with specific values
    create_issue(
        project_path,
        CreateIssueOptions {
            title: "Original".to_string(),
            description: "Original description".to_string(),
            priority: Some(1), // high
            status: Some("open".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Only update title
    let result = update_issue(
        project_path,
        "0001",
        UpdateIssueOptions {
            title: Some("New Title".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Other fields should be preserved
    assert_eq!(result.issue.title, "New Title");
    assert_eq!(result.issue.description, "Original description");
    assert_eq!(result.issue.metadata.priority, 1); // high
    assert_eq!(result.issue.metadata.status, "open");
}

#[tokio::test]
async fn test_update_issue_not_found() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let result = update_issue(
        project_path,
        "9999",
        UpdateIssueOptions {
            title: Some("New".to_string()),
            ..Default::default()
        },
    )
    .await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        IssueCrudError::IssueNotFound(_)
    ));
}

#[tokio::test]
async fn test_delete_issue_success() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create issue
    create_issue(
        project_path,
        CreateIssueOptions {
            title: "To Delete".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Verify it exists
    let issue_path = project_path.join(".centy/issues/0001");
    assert!(issue_path.exists());

    // Delete it
    delete_issue(project_path, "0001")
        .await
        .expect("Should delete");

    // Verify it's gone
    assert!(!issue_path.exists());

    // Verify not in list
    let issues = list_issues(project_path, None, None).await.unwrap();
    assert!(issues.is_empty());
}

#[tokio::test]
async fn test_delete_issue_removes_from_manifest() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create issue
    create_issue(
        project_path,
        CreateIssueOptions {
            title: "Test".to_string(),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Verify issue is in manifest
    let manifest = centy_daemon::manifest::read_manifest(project_path)
        .await
        .unwrap()
        .unwrap();
    let issue_files: Vec<_> = manifest
        .managed_files
        .iter()
        .filter(|f| f.path.starts_with("issues/0001"))
        .collect();
    assert!(!issue_files.is_empty(), "Issue should be in manifest");

    // Delete issue
    let result = delete_issue(project_path, "0001").await.unwrap();

    // Verify removed from manifest
    let issue_files: Vec<_> = result
        .manifest
        .managed_files
        .iter()
        .filter(|f| f.path.starts_with("issues/0001"))
        .collect();
    assert!(issue_files.is_empty(), "Issue should be removed from manifest");
}

#[tokio::test]
async fn test_delete_issue_not_found() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    let result = delete_issue(project_path, "9999").await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        IssueCrudError::IssueNotFound(_)
    ));
}

#[tokio::test]
async fn test_issue_with_custom_fields() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create issue with custom fields
    let mut custom_fields = HashMap::new();
    custom_fields.insert("assignee".to_string(), "alice".to_string());
    custom_fields.insert("component".to_string(), "backend".to_string());

    create_issue(
        project_path,
        CreateIssueOptions {
            title: "Custom Fields Test".to_string(),
            custom_fields,
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Get and verify
    let issue = get_issue(project_path, "0001").await.unwrap();
    assert_eq!(
        issue.metadata.custom_fields.get("assignee"),
        Some(&"alice".to_string())
    );
    assert_eq!(
        issue.metadata.custom_fields.get("component"),
        Some(&"backend".to_string())
    );
}

#[tokio::test]
async fn test_update_issue_custom_fields() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create with initial custom fields
    let mut initial_fields = HashMap::new();
    initial_fields.insert("assignee".to_string(), "alice".to_string());

    create_issue(
        project_path,
        CreateIssueOptions {
            title: "Test".to_string(),
            custom_fields: initial_fields,
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Update custom fields
    let mut new_fields = HashMap::new();
    new_fields.insert("assignee".to_string(), "bob".to_string());
    new_fields.insert("reviewer".to_string(), "charlie".to_string());

    let result = update_issue(
        project_path,
        "0001",
        UpdateIssueOptions {
            custom_fields: new_fields,
            ..Default::default()
        },
    )
    .await
    .unwrap();

    assert_eq!(
        result.issue.metadata.custom_fields.get("assignee"),
        Some(&"bob".to_string())
    );
    assert_eq!(
        result.issue.metadata.custom_fields.get("reviewer"),
        Some(&"charlie".to_string())
    );
}

#[tokio::test]
async fn test_create_issue_validates_priority_range() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Try to create issue with out-of-range priority (default is 3 levels)
    let options = CreateIssueOptions {
        title: "Invalid Priority".to_string(),
        priority: Some(5), // Invalid - max is 3
        ..Default::default()
    };

    let result = create_issue(project_path, options).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), IssueError::InvalidPriority(_)));
}

#[tokio::test]
async fn test_update_issue_priority() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();

    init_centy_project(project_path).await;

    // Create with low priority
    create_issue(
        project_path,
        CreateIssueOptions {
            title: "Test".to_string(),
            priority: Some(3), // low
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Update to high priority
    let result = update_issue(
        project_path,
        "0001",
        UpdateIssueOptions {
            priority: Some(1), // high
            ..Default::default()
        },
    )
    .await
    .expect("Should update");

    assert_eq!(result.issue.metadata.priority, 1);
}
