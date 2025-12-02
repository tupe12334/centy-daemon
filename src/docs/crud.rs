use crate::manifest::{
    add_file_to_manifest, create_managed_file, read_manifest, write_manifest, CentyManifest,
    ManagedFileType,
};
use crate::template::{DocTemplateContext, TemplateEngine, TemplateError};
use crate::utils::{compute_hash, get_centy_path, now_iso};
use std::path::Path;
use thiserror::Error;
use tokio::fs;

#[derive(Error, Debug)]
pub enum DocError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Manifest error: {0}")]
    ManifestError(#[from] crate::manifest::ManifestError),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Centy not initialized. Run 'centy init' first.")]
    NotInitialized,

    #[error("Doc '{0}' not found")]
    DocNotFound(String),

    #[error("Title is required")]
    TitleRequired,

    #[error("Doc with slug '{0}' already exists")]
    SlugAlreadyExists(String),

    #[error("Invalid slug: {0}")]
    InvalidSlug(String),

    #[error("Template error: {0}")]
    TemplateError(#[from] TemplateError),
}

/// Full doc data
#[derive(Debug, Clone)]
pub struct Doc {
    pub slug: String,
    pub title: String,
    pub content: String,
    pub metadata: DocMetadata,
}

/// Doc metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocMetadata {
    pub created_at: String,
    pub updated_at: String,
}

impl DocMetadata {
    pub fn new() -> Self {
        let now = now_iso();
        Self {
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

impl Default for DocMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Options for creating a doc
#[derive(Debug, Clone, Default)]
pub struct CreateDocOptions {
    pub title: String,
    pub content: String,
    pub slug: Option<String>,
    /// Optional template name (without .md extension)
    pub template: Option<String>,
}

/// Result of doc creation
#[derive(Debug, Clone)]
pub struct CreateDocResult {
    pub slug: String,
    pub created_file: String,
    pub manifest: CentyManifest,
}

/// Options for updating a doc
#[derive(Debug, Clone, Default)]
pub struct UpdateDocOptions {
    pub title: Option<String>,
    pub content: Option<String>,
    pub new_slug: Option<String>,
}

/// Result of doc update
#[derive(Debug, Clone)]
pub struct UpdateDocResult {
    pub doc: Doc,
    pub manifest: CentyManifest,
}

/// Result of doc deletion
#[derive(Debug, Clone)]
pub struct DeleteDocResult {
    pub manifest: CentyManifest,
}

/// Create a new doc
pub async fn create_doc(
    project_path: &Path,
    options: CreateDocOptions,
) -> Result<CreateDocResult, DocError> {
    // Validate title
    if options.title.trim().is_empty() {
        return Err(DocError::TitleRequired);
    }

    // Check if centy is initialized
    let mut manifest = read_manifest(project_path)
        .await?
        .ok_or(DocError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let docs_path = centy_path.join("docs");

    // Ensure docs directory exists
    if !docs_path.exists() {
        fs::create_dir_all(&docs_path).await?;
    }

    // Generate or validate slug
    let slug = match options.slug {
        Some(s) if !s.trim().is_empty() => {
            let slug = slugify(&s);
            validate_slug(&slug)?;
            slug
        }
        _ => slugify(&options.title),
    };

    // Check if slug already exists
    let doc_path = docs_path.join(format!("{}.md", slug));
    if doc_path.exists() {
        return Err(DocError::SlugAlreadyExists(slug));
    }

    // Create metadata
    let metadata = DocMetadata::new();

    // Generate doc content with frontmatter
    let doc_content = if let Some(ref template_name) = options.template {
        // Use template engine
        let template_engine = TemplateEngine::new();
        let context = DocTemplateContext {
            title: options.title.clone(),
            content: options.content.clone(),
            slug: slug.clone(),
            created_at: metadata.created_at.clone(),
            updated_at: metadata.updated_at.clone(),
        };
        template_engine
            .render_doc(project_path, template_name, &context)
            .await?
    } else {
        // Use default format
        generate_doc_content(&options.title, &options.content, &metadata)
    };

    // Write the doc file
    fs::write(&doc_path, &doc_content).await?;

    // Update manifest
    let relative_path = format!("docs/{}.md", slug);
    add_file_to_manifest(
        &mut manifest,
        create_managed_file(relative_path.clone(), compute_hash(&doc_content), ManagedFileType::File),
    );

    write_manifest(project_path, &manifest).await?;

    let created_file = format!(".centy/docs/{}.md", slug);

    Ok(CreateDocResult {
        slug,
        created_file,
        manifest,
    })
}

/// Get a single doc by its slug
pub async fn get_doc(project_path: &Path, slug: &str) -> Result<Doc, DocError> {
    // Check if centy is initialized
    read_manifest(project_path)
        .await?
        .ok_or(DocError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let doc_path = centy_path.join("docs").join(format!("{}.md", slug));

    if !doc_path.exists() {
        return Err(DocError::DocNotFound(slug.to_string()));
    }

    read_doc_from_disk(&doc_path, slug).await
}

/// List all docs
pub async fn list_docs(project_path: &Path) -> Result<Vec<Doc>, DocError> {
    // Check if centy is initialized
    read_manifest(project_path)
        .await?
        .ok_or(DocError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let docs_path = centy_path.join("docs");

    if !docs_path.exists() {
        return Ok(Vec::new());
    }

    let mut docs = Vec::new();
    let mut entries = fs::read_dir(&docs_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
            if let Some(slug) = path.file_stem().and_then(|s| s.to_str()) {
                // Skip the README.md that's managed by centy
                if slug == "README" {
                    continue;
                }
                match read_doc_from_disk(&path, slug).await {
                    Ok(doc) => docs.push(doc),
                    Err(_) => continue, // Skip docs that can't be read
                }
            }
        }
    }

    // Sort by slug
    docs.sort_by(|a, b| a.slug.cmp(&b.slug));

    Ok(docs)
}

/// Update an existing doc
pub async fn update_doc(
    project_path: &Path,
    slug: &str,
    options: UpdateDocOptions,
) -> Result<UpdateDocResult, DocError> {
    // Check if centy is initialized
    let mut manifest = read_manifest(project_path)
        .await?
        .ok_or(DocError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let docs_path = centy_path.join("docs");
    let doc_path = docs_path.join(format!("{}.md", slug));

    if !doc_path.exists() {
        return Err(DocError::DocNotFound(slug.to_string()));
    }

    // Read current doc
    let current = read_doc_from_disk(&doc_path, slug).await?;

    // Apply updates
    let new_title = options.title.unwrap_or(current.title);
    let new_content = options.content.unwrap_or(current.content);

    // Handle slug rename
    let new_slug = match options.new_slug {
        Some(s) if !s.trim().is_empty() && s != slug => {
            let new_slug = slugify(&s);
            validate_slug(&new_slug)?;

            // Check if new slug already exists
            let new_path = docs_path.join(format!("{}.md", new_slug));
            if new_path.exists() {
                return Err(DocError::SlugAlreadyExists(new_slug));
            }

            Some(new_slug)
        }
        _ => None,
    };

    // Create updated metadata
    let updated_metadata = DocMetadata {
        created_at: current.metadata.created_at.clone(),
        updated_at: now_iso(),
    };

    // Generate updated content
    let doc_content = generate_doc_content(&new_title, &new_content, &updated_metadata);

    // Handle file rename or update
    let final_slug = if let Some(ref new_slug) = new_slug {
        // Remove old file
        fs::remove_file(&doc_path).await?;

        // Write new file
        let new_path = docs_path.join(format!("{}.md", new_slug));
        fs::write(&new_path, &doc_content).await?;

        // Update manifest - remove old entry
        let old_path = format!("docs/{}.md", slug);
        manifest.managed_files.retain(|f| f.path != old_path);

        // Add new entry
        let new_relative_path = format!("docs/{}.md", new_slug);
        add_file_to_manifest(
            &mut manifest,
            create_managed_file(new_relative_path, compute_hash(&doc_content), ManagedFileType::File),
        );

        new_slug.clone()
    } else {
        // Just update the existing file
        fs::write(&doc_path, &doc_content).await?;

        // Update manifest hash
        let relative_path = format!("docs/{}.md", slug);
        add_file_to_manifest(
            &mut manifest,
            create_managed_file(relative_path, compute_hash(&doc_content), ManagedFileType::File),
        );

        slug.to_string()
    };

    manifest.updated_at = now_iso();
    write_manifest(project_path, &manifest).await?;

    let doc = Doc {
        slug: final_slug,
        title: new_title,
        content: new_content,
        metadata: updated_metadata,
    };

    Ok(UpdateDocResult { doc, manifest })
}

/// Delete a doc
pub async fn delete_doc(project_path: &Path, slug: &str) -> Result<DeleteDocResult, DocError> {
    // Check if centy is initialized
    let mut manifest = read_manifest(project_path)
        .await?
        .ok_or(DocError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let doc_path = centy_path.join("docs").join(format!("{}.md", slug));

    if !doc_path.exists() {
        return Err(DocError::DocNotFound(slug.to_string()));
    }

    // Remove the file
    fs::remove_file(&doc_path).await?;

    // Remove from manifest
    let relative_path = format!("docs/{}.md", slug);
    manifest.managed_files.retain(|f| f.path != relative_path);
    manifest.updated_at = now_iso();

    write_manifest(project_path, &manifest).await?;

    Ok(DeleteDocResult { manifest })
}

/// Read a doc from disk
async fn read_doc_from_disk(doc_path: &Path, slug: &str) -> Result<Doc, DocError> {
    let content = fs::read_to_string(doc_path).await?;
    let (title, body, metadata) = parse_doc_content(&content);

    Ok(Doc {
        slug: slug.to_string(),
        title,
        content: body,
        metadata,
    })
}

/// Generate doc content with YAML frontmatter
fn generate_doc_content(title: &str, content: &str, metadata: &DocMetadata) -> String {
    format!(
        "---\ntitle: \"{}\"\ncreatedAt: \"{}\"\nupdatedAt: \"{}\"\n---\n\n# {}\n\n{}",
        escape_yaml_string(title),
        metadata.created_at,
        metadata.updated_at,
        title,
        content
    )
}

/// Parse doc content extracting title, body, and metadata from frontmatter
fn parse_doc_content(content: &str) -> (String, String, DocMetadata) {
    let lines: Vec<&str> = content.lines().collect();

    // Check for frontmatter
    if lines.first() == Some(&"---") {
        // Find closing ---
        if let Some(end_idx) = lines.iter().skip(1).position(|&line| line == "---") {
            let frontmatter: Vec<&str> = lines[1..=end_idx].to_vec();
            let body_start = end_idx + 2;

            // Parse frontmatter
            let mut title = String::new();
            let mut created_at = String::new();
            let mut updated_at = String::new();

            for line in frontmatter {
                if let Some(value) = line.strip_prefix("title:") {
                    title = value.trim().trim_matches('"').to_string();
                } else if let Some(value) = line.strip_prefix("createdAt:") {
                    created_at = value.trim().trim_matches('"').to_string();
                } else if let Some(value) = line.strip_prefix("updatedAt:") {
                    updated_at = value.trim().trim_matches('"').to_string();
                }
            }

            // Get body (skip empty lines after frontmatter)
            let body_lines: Vec<&str> = lines[body_start..]
                .iter()
                .skip_while(|line| line.is_empty())
                .copied()
                .collect();

            // Skip the title line if it matches (# Title)
            let body = if body_lines.first().map_or(false, |l| l.starts_with("# ")) {
                body_lines[1..]
                    .iter()
                    .skip_while(|line| line.is_empty())
                    .copied()
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                body_lines.join("\n")
            };

            let metadata = DocMetadata {
                created_at: if created_at.is_empty() { now_iso() } else { created_at },
                updated_at: if updated_at.is_empty() { now_iso() } else { updated_at },
            };

            return (title, body.trim_end().to_string(), metadata);
        }
    }

    // No frontmatter - extract title from first # heading
    let mut title = String::new();
    let mut body_start = 0;

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("# ") {
            title = line.strip_prefix("# ").unwrap_or("").to_string();
            body_start = i + 1;
            break;
        }
    }

    let body = lines[body_start..]
        .iter()
        .skip_while(|line| line.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string();

    (title, body, DocMetadata::new())
}

/// Convert a string to a URL-friendly slug
fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c
            } else if c == ' ' || c == '_' || c == '-' {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Validate a slug
fn validate_slug(slug: &str) -> Result<(), DocError> {
    if slug.is_empty() {
        return Err(DocError::InvalidSlug("Slug cannot be empty".to_string()));
    }

    if !slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(DocError::InvalidSlug(
            "Slug can only contain alphanumeric characters and hyphens".to_string(),
        ));
    }

    if slug.starts_with('-') || slug.ends_with('-') {
        return Err(DocError::InvalidSlug(
            "Slug cannot start or end with a hyphen".to_string(),
        ));
    }

    Ok(())
}

/// Escape special characters in YAML strings
fn escape_yaml_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Getting Started Guide"), "getting-started-guide");
        assert_eq!(slugify("API v2"), "api-v2");
        assert_eq!(slugify("  Spaces  "), "spaces");
        assert_eq!(slugify("multiple---hyphens"), "multiple-hyphens");
        assert_eq!(slugify("Under_score"), "under-score");
    }

    #[test]
    fn test_validate_slug() {
        assert!(validate_slug("hello-world").is_ok());
        assert!(validate_slug("api-v2").is_ok());
        assert!(validate_slug("").is_err());
        assert!(validate_slug("-start").is_err());
        assert!(validate_slug("end-").is_err());
        assert!(validate_slug("has space").is_err());
    }

    #[test]
    fn test_parse_doc_content_with_frontmatter() {
        let content = r#"---
title: "My Doc"
createdAt: "2024-01-01T00:00:00Z"
updatedAt: "2024-01-02T00:00:00Z"
---

# My Doc

This is the content."#;

        let (title, body, metadata) = parse_doc_content(content);
        assert_eq!(title, "My Doc");
        assert_eq!(body, "This is the content.");
        assert_eq!(metadata.created_at, "2024-01-01T00:00:00Z");
        assert_eq!(metadata.updated_at, "2024-01-02T00:00:00Z");
    }

    #[test]
    fn test_parse_doc_content_without_frontmatter() {
        let content = "# Simple Doc\n\nJust some content here.";
        let (title, body, _metadata) = parse_doc_content(content);
        assert_eq!(title, "Simple Doc");
        assert_eq!(body, "Just some content here.");
    }

    #[test]
    fn test_generate_doc_content() {
        let metadata = DocMetadata {
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-02T00:00:00Z".to_string(),
        };
        let content = generate_doc_content("Test Title", "Body text", &metadata);

        assert!(content.contains("title: \"Test Title\""));
        assert!(content.contains("# Test Title"));
        assert!(content.contains("Body text"));
    }

    #[test]
    fn test_escape_yaml_string() {
        assert_eq!(escape_yaml_string("simple"), "simple");
        assert_eq!(escape_yaml_string("with \"quotes\""), "with \\\"quotes\\\"");
        assert_eq!(escape_yaml_string("back\\slash"), "back\\\\slash");
    }
}
