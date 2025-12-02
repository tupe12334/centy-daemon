mod common;

use centy_daemon::docs::{create_doc, CreateDocOptions};
use centy_daemon::issue::{create_issue, CreateIssueOptions};
use common::{create_test_dir, init_centy_project};
use std::collections::HashMap;
use tokio::fs;

// ============ Issue Template Tests ============

#[tokio::test]
async fn test_create_issue_with_explicit_template() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    // Create a custom template
    let template_path = project_path.join(".centy/templates/issues/bug-report.md");
    fs::write(
        &template_path,
        "# BUG: {{title}}\n\n**Status:** {{status}}\n\n{{description}}",
    )
    .await
    .expect("Should write template");

    let options = CreateIssueOptions {
        title: "Login Crash".to_string(),
        description: "App crashes on login".to_string(),
        status: Some("open".to_string()),
        template: Some("bug-report".to_string()),
        ..Default::default()
    };

    let result = create_issue(project_path, options)
        .await
        .expect("Should create issue with template");

    let issue_content = fs::read_to_string(
        project_path.join(format!(".centy/issues/{}/issue.md", result.issue_number)),
    )
    .await
    .expect("Should read issue file");

    assert!(issue_content.contains("# BUG: Login Crash"));
    assert!(issue_content.contains("**Status:** open"));
    assert!(issue_content.contains("App crashes on login"));
}

#[tokio::test]
async fn test_create_issue_without_template_uses_default() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    // Don't create any templates - should use default format
    let options = CreateIssueOptions {
        title: "Simple Issue".to_string(),
        description: "Description here".to_string(),
        ..Default::default()
    };

    let result = create_issue(project_path, options)
        .await
        .expect("Should create issue");

    let issue_content = fs::read_to_string(
        project_path.join(format!(".centy/issues/{}/issue.md", result.issue_number)),
    )
    .await
    .expect("Should read issue file");

    // Should use hardcoded default format
    assert_eq!(issue_content, "# Simple Issue\n\nDescription here\n");
}

#[tokio::test]
async fn test_create_issue_template_not_found_returns_error() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    let options = CreateIssueOptions {
        title: "Test".to_string(),
        template: Some("nonexistent-template".to_string()),
        ..Default::default()
    };

    let result = create_issue(project_path, options).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_issue_template_with_custom_fields_loop() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    // Create template using custom_fields with Handlebars loop
    let template_path = project_path.join(".centy/templates/issues/detailed.md");
    fs::write(
        &template_path,
        r#"# {{title}}

{{#each custom_fields}}
- **{{@key}}:** {{this}}
{{/each}}

{{description}}"#,
    )
    .await
    .expect("Should write template");

    let mut custom_fields = HashMap::new();
    custom_fields.insert("assignee".to_string(), "alice".to_string());
    custom_fields.insert("component".to_string(), "auth".to_string());

    let options = CreateIssueOptions {
        title: "Auth Bug".to_string(),
        description: "Fix auth".to_string(),
        custom_fields,
        template: Some("detailed".to_string()),
        ..Default::default()
    };

    let result = create_issue(project_path, options)
        .await
        .expect("Should create issue");

    let issue_content = fs::read_to_string(
        project_path.join(format!(".centy/issues/{}/issue.md", result.issue_number)),
    )
    .await
    .expect("Should read issue file");

    assert!(issue_content.contains("assignee"));
    assert!(issue_content.contains("alice"));
    assert!(issue_content.contains("component"));
    assert!(issue_content.contains("auth"));
}

#[tokio::test]
async fn test_issue_template_with_conditionals() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    // Create template with conditional
    let template_path = project_path.join(".centy/templates/issues/conditional.md");
    fs::write(
        &template_path,
        r#"# {{title}}

{{#if description}}
## Description
{{description}}
{{/if}}"#,
    )
    .await
    .expect("Should write template");

    // Test with description
    let options_with_desc = CreateIssueOptions {
        title: "With Desc".to_string(),
        description: "Has description".to_string(),
        template: Some("conditional".to_string()),
        ..Default::default()
    };
    let result = create_issue(project_path, options_with_desc)
        .await
        .expect("Should create issue");

    let content = fs::read_to_string(
        project_path.join(format!(".centy/issues/{}/issue.md", result.issue_number)),
    )
    .await
    .unwrap();
    assert!(content.contains("## Description"));
    assert!(content.contains("Has description"));

    // Test without description
    let options_no_desc = CreateIssueOptions {
        title: "No Desc".to_string(),
        description: "".to_string(),
        template: Some("conditional".to_string()),
        ..Default::default()
    };
    let result2 = create_issue(project_path, options_no_desc)
        .await
        .expect("Should create issue");

    let content2 = fs::read_to_string(
        project_path.join(format!(".centy/issues/{}/issue.md", result2.issue_number)),
    )
    .await
    .unwrap();
    assert!(!content2.contains("## Description"));
}

// ============ Doc Template Tests ============

#[tokio::test]
async fn test_create_doc_with_explicit_template() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    // Create a custom API doc template
    let template_path = project_path.join(".centy/templates/docs/api.md");
    fs::write(
        &template_path,
        r#"---
title: "{{title}}"
slug: "{{slug}}"
type: "api-reference"
---

# API: {{title}}

{{content}}"#,
    )
    .await
    .expect("Should write template");

    let options = CreateDocOptions {
        title: "Users Endpoint".to_string(),
        content: "The users API...".to_string(),
        template: Some("api".to_string()),
        ..Default::default()
    };

    create_doc(project_path, options)
        .await
        .expect("Should create doc with template");

    let doc_content = fs::read_to_string(project_path.join(".centy/docs/users-endpoint.md"))
        .await
        .expect("Should read doc file");

    assert!(doc_content.contains("type: \"api-reference\""));
    assert!(doc_content.contains("# API: Users Endpoint"));
    assert!(doc_content.contains("The users API..."));
}

#[tokio::test]
async fn test_create_doc_without_template_uses_default() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    let options = CreateDocOptions {
        title: "Getting Started".to_string(),
        content: "Welcome!".to_string(),
        ..Default::default()
    };

    create_doc(project_path, options)
        .await
        .expect("Should create doc");

    let doc_content = fs::read_to_string(project_path.join(".centy/docs/getting-started.md"))
        .await
        .expect("Should read doc file");

    // Should use default format with frontmatter
    assert!(doc_content.contains("title: \"Getting Started\""));
    assert!(doc_content.contains("# Getting Started"));
    assert!(doc_content.contains("Welcome!"));
}

#[tokio::test]
async fn test_create_doc_template_not_found_returns_error() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    let options = CreateDocOptions {
        title: "Test".to_string(),
        content: "Content".to_string(),
        template: Some("nonexistent".to_string()),
        ..Default::default()
    };

    let result = create_doc(project_path, options).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

// ============ Init Tests for Template Folders ============

#[tokio::test]
async fn test_init_creates_template_folders() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    let templates_path = project_path.join(".centy/templates");
    assert!(templates_path.exists(), "templates/ should exist");
    assert!(
        templates_path.join("issues").exists(),
        "templates/issues/ should exist"
    );
    assert!(
        templates_path.join("docs").exists(),
        "templates/docs/ should exist"
    );
    assert!(
        templates_path.join("README.md").exists(),
        "templates/README.md should exist"
    );
}

#[tokio::test]
async fn test_templates_readme_contains_documentation() {
    let temp_dir = create_test_dir();
    let project_path = temp_dir.path();
    init_centy_project(project_path).await;

    let readme_content =
        fs::read_to_string(project_path.join(".centy/templates/README.md"))
            .await
            .expect("Should read templates README");

    // Verify README contains documentation about templates
    assert!(readme_content.contains("Handlebars"));
    assert!(readme_content.contains("{{title}}"));
    assert!(readme_content.contains("{{description}}"));
    assert!(readme_content.contains("{{#if"));
    assert!(readme_content.contains("{{#each"));
}
