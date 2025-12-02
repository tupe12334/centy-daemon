mod common;

use centy_daemon::docs::{
    create_doc, delete_doc, get_doc, list_docs, update_doc, CreateDocOptions, UpdateDocOptions,
};
use common::{create_test_dir, init_centy_project, verify_centy_structure};
use std::path::Path;

// ============ Create Doc Tests ============

#[tokio::test]
async fn test_create_doc_success() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    let result = create_doc(
        project_path,
        CreateDocOptions {
            title: "Getting Started".to_string(),
            content: "Welcome to our project!".to_string(),
            slug: None,
        },
    )
    .await
    .expect("Should create doc");

    assert_eq!(result.slug, "getting-started");
    assert!(result.created_file.contains("getting-started.md"));

    // Verify file exists
    let doc_path = project_path.join(".centy/docs/getting-started.md");
    assert!(doc_path.exists(), "Doc file should exist");
}

#[tokio::test]
async fn test_create_doc_with_custom_slug() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    let result = create_doc(
        project_path,
        CreateDocOptions {
            title: "Getting Started".to_string(),
            content: "Content here".to_string(),
            slug: Some("quickstart".to_string()),
        },
    )
    .await
    .expect("Should create doc with custom slug");

    assert_eq!(result.slug, "quickstart");
}

#[tokio::test]
async fn test_create_doc_empty_title_fails() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    let result = create_doc(
        project_path,
        CreateDocOptions {
            title: "   ".to_string(),
            content: "Content".to_string(),
            slug: None,
        },
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Title is required"));
}

#[tokio::test]
async fn test_create_doc_duplicate_slug_fails() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    // Create first doc
    create_doc(
        project_path,
        CreateDocOptions {
            title: "Test Doc".to_string(),
            content: "Content".to_string(),
            slug: Some("test-doc".to_string()),
        },
    )
    .await
    .expect("First doc should succeed");

    // Try to create with same slug
    let result = create_doc(
        project_path,
        CreateDocOptions {
            title: "Another Doc".to_string(),
            content: "Content".to_string(),
            slug: Some("test-doc".to_string()),
        },
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
}

#[tokio::test]
async fn test_create_doc_not_initialized_fails() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    // Don't initialize

    let result = create_doc(
        project_path,
        CreateDocOptions {
            title: "Test".to_string(),
            content: "Content".to_string(),
            slug: None,
        },
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not initialized"));
}

// ============ Get Doc Tests ============

#[tokio::test]
async fn test_get_doc_success() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    // Create a doc
    create_doc(
        project_path,
        CreateDocOptions {
            title: "API Reference".to_string(),
            content: "This is the API documentation.".to_string(),
            slug: Some("api-reference".to_string()),
        },
    )
    .await
    .expect("Should create doc");

    // Get the doc
    let doc = get_doc(project_path, "api-reference")
        .await
        .expect("Should get doc");

    assert_eq!(doc.slug, "api-reference");
    assert_eq!(doc.title, "API Reference");
    assert_eq!(doc.content, "This is the API documentation.");
    assert!(!doc.metadata.created_at.is_empty());
    assert!(!doc.metadata.updated_at.is_empty());
}

#[tokio::test]
async fn test_get_doc_not_found() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    let result = get_doc(project_path, "nonexistent").await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

// ============ List Docs Tests ============

#[tokio::test]
async fn test_list_docs_empty() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    let docs = list_docs(project_path).await.expect("Should list docs");

    assert!(docs.is_empty());
}

#[tokio::test]
async fn test_list_docs_multiple() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    // Create multiple docs
    create_doc(
        project_path,
        CreateDocOptions {
            title: "Zebra Doc".to_string(),
            content: "Z content".to_string(),
            slug: Some("zebra".to_string()),
        },
    )
    .await
    .unwrap();

    create_doc(
        project_path,
        CreateDocOptions {
            title: "Alpha Doc".to_string(),
            content: "A content".to_string(),
            slug: Some("alpha".to_string()),
        },
    )
    .await
    .unwrap();

    create_doc(
        project_path,
        CreateDocOptions {
            title: "Beta Doc".to_string(),
            content: "B content".to_string(),
            slug: Some("beta".to_string()),
        },
    )
    .await
    .unwrap();

    let docs = list_docs(project_path).await.expect("Should list docs");

    assert_eq!(docs.len(), 3);
    // Should be sorted by slug
    assert_eq!(docs[0].slug, "alpha");
    assert_eq!(docs[1].slug, "beta");
    assert_eq!(docs[2].slug, "zebra");
}

// ============ Update Doc Tests ============

#[tokio::test]
async fn test_update_doc_title() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    create_doc(
        project_path,
        CreateDocOptions {
            title: "Original Title".to_string(),
            content: "Original content".to_string(),
            slug: Some("test-doc".to_string()),
        },
    )
    .await
    .unwrap();

    let result = update_doc(
        project_path,
        "test-doc",
        UpdateDocOptions {
            title: Some("New Title".to_string()),
            content: None,
            new_slug: None,
        },
    )
    .await
    .expect("Should update doc");

    assert_eq!(result.doc.title, "New Title");
    assert_eq!(result.doc.content, "Original content"); // Unchanged
    assert_eq!(result.doc.slug, "test-doc"); // Unchanged
}

#[tokio::test]
async fn test_update_doc_content() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    create_doc(
        project_path,
        CreateDocOptions {
            title: "Test Doc".to_string(),
            content: "Original content".to_string(),
            slug: Some("test-doc".to_string()),
        },
    )
    .await
    .unwrap();

    let result = update_doc(
        project_path,
        "test-doc",
        UpdateDocOptions {
            title: None,
            content: Some("Updated content here".to_string()),
            new_slug: None,
        },
    )
    .await
    .expect("Should update doc");

    assert_eq!(result.doc.content, "Updated content here");
    assert_eq!(result.doc.title, "Test Doc"); // Unchanged
}

#[tokio::test]
async fn test_update_doc_rename_slug() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    create_doc(
        project_path,
        CreateDocOptions {
            title: "Test Doc".to_string(),
            content: "Content".to_string(),
            slug: Some("old-slug".to_string()),
        },
    )
    .await
    .unwrap();

    let result = update_doc(
        project_path,
        "old-slug",
        UpdateDocOptions {
            title: None,
            content: None,
            new_slug: Some("new-slug".to_string()),
        },
    )
    .await
    .expect("Should update doc");

    assert_eq!(result.doc.slug, "new-slug");

    // Old slug should not exist
    let old_result = get_doc(project_path, "old-slug").await;
    assert!(old_result.is_err());

    // New slug should exist
    let new_doc = get_doc(project_path, "new-slug")
        .await
        .expect("New slug should exist");
    assert_eq!(new_doc.title, "Test Doc");
}

#[tokio::test]
async fn test_update_doc_not_found() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    let result = update_doc(
        project_path,
        "nonexistent",
        UpdateDocOptions {
            title: Some("New Title".to_string()),
            content: None,
            new_slug: None,
        },
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_update_doc_rename_to_existing_slug_fails() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    // Create two docs
    create_doc(
        project_path,
        CreateDocOptions {
            title: "Doc One".to_string(),
            content: "Content".to_string(),
            slug: Some("doc-one".to_string()),
        },
    )
    .await
    .unwrap();

    create_doc(
        project_path,
        CreateDocOptions {
            title: "Doc Two".to_string(),
            content: "Content".to_string(),
            slug: Some("doc-two".to_string()),
        },
    )
    .await
    .unwrap();

    // Try to rename doc-one to doc-two
    let result = update_doc(
        project_path,
        "doc-one",
        UpdateDocOptions {
            title: None,
            content: None,
            new_slug: Some("doc-two".to_string()),
        },
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
}

// ============ Delete Doc Tests ============

#[tokio::test]
async fn test_delete_doc_success() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    create_doc(
        project_path,
        CreateDocOptions {
            title: "To Delete".to_string(),
            content: "Content".to_string(),
            slug: Some("to-delete".to_string()),
        },
    )
    .await
    .unwrap();

    // Verify it exists
    assert!(get_doc(project_path, "to-delete").await.is_ok());

    // Delete it
    delete_doc(project_path, "to-delete")
        .await
        .expect("Should delete doc");

    // Verify it's gone
    let result = get_doc(project_path, "to-delete").await;
    assert!(result.is_err());

    // Verify file is gone
    let doc_path = project_path.join(".centy/docs/to-delete.md");
    assert!(!doc_path.exists());
}

#[tokio::test]
async fn test_delete_doc_not_found() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    let result = delete_doc(project_path, "nonexistent").await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_delete_doc_updates_manifest() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    let create_result = create_doc(
        project_path,
        CreateDocOptions {
            title: "Test Doc".to_string(),
            content: "Content".to_string(),
            slug: Some("test-doc".to_string()),
        },
    )
    .await
    .unwrap();

    // Manifest should contain the doc
    assert!(create_result
        .manifest
        .managed_files
        .iter()
        .any(|f| f.path == "docs/test-doc.md"));

    let delete_result = delete_doc(project_path, "test-doc").await.unwrap();

    // Manifest should NOT contain the doc
    assert!(!delete_result
        .manifest
        .managed_files
        .iter()
        .any(|f| f.path == "docs/test-doc.md"));
}

// ============ Slug Generation Tests ============

#[tokio::test]
async fn test_slug_generation_from_title() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    let result = create_doc(
        project_path,
        CreateDocOptions {
            title: "How to Use the API v2".to_string(),
            content: "Content".to_string(),
            slug: None,
        },
    )
    .await
    .unwrap();

    assert_eq!(result.slug, "how-to-use-the-api-v2");
}

#[tokio::test]
async fn test_slug_handles_special_characters() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    let result = create_doc(
        project_path,
        CreateDocOptions {
            title: "C++ & Rust: A Comparison!".to_string(),
            content: "Content".to_string(),
            slug: None,
        },
    )
    .await
    .unwrap();

    // Should only contain alphanumeric and hyphens
    assert!(result.slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'));
    assert!(!result.slug.starts_with('-'));
    assert!(!result.slug.ends_with('-'));
}

// ============ Frontmatter Tests ============

#[tokio::test]
async fn test_doc_preserves_metadata_on_update() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    // Create doc
    create_doc(
        project_path,
        CreateDocOptions {
            title: "Test Doc".to_string(),
            content: "Original".to_string(),
            slug: Some("test".to_string()),
        },
    )
    .await
    .unwrap();

    let original = get_doc(project_path, "test").await.unwrap();
    let original_created_at = original.metadata.created_at.clone();

    // Wait a tiny bit to ensure timestamps differ
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Update doc
    update_doc(
        project_path,
        "test",
        UpdateDocOptions {
            title: None,
            content: Some("Updated".to_string()),
            new_slug: None,
        },
    )
    .await
    .unwrap();

    let updated = get_doc(project_path, "test").await.unwrap();

    // created_at should be preserved
    assert_eq!(updated.metadata.created_at, original_created_at);
    // updated_at should be different
    assert_ne!(updated.metadata.updated_at, original_created_at);
}
