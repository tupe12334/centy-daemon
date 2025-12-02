use handlebars::Handlebars;
use std::path::Path;
use thiserror::Error;
use tokio::fs;

use super::types::{DocTemplateContext, IssueTemplateContext, TemplateType};
use crate::utils::get_centy_path;

#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Template error: {0}")]
    TemplateError(#[from] handlebars::TemplateError),

    #[error("Render error: {0}")]
    RenderError(#[from] handlebars::RenderError),

    #[error("Template '{0}' not found")]
    TemplateNotFound(String),
}

pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        let handlebars = Handlebars::new();
        Self { handlebars }
    }

    /// Get the templates directory path
    pub fn get_templates_path(project_path: &Path) -> std::path::PathBuf {
        get_centy_path(project_path).join("templates")
    }

    /// Get the path for a specific template type's folder
    pub fn get_template_type_path(
        project_path: &Path,
        template_type: TemplateType,
    ) -> std::path::PathBuf {
        Self::get_templates_path(project_path).join(template_type.folder_name())
    }

    /// Load a template from disk by name
    /// Looks for "{template_name}.md" in the appropriate template folder
    pub async fn load_template(
        &self,
        project_path: &Path,
        template_type: TemplateType,
        template_name: &str,
    ) -> Result<String, TemplateError> {
        let template_folder = Self::get_template_type_path(project_path, template_type);
        let file_name = format!("{}.md", template_name);
        let template_path = template_folder.join(&file_name);

        if template_path.exists() {
            let content = fs::read_to_string(&template_path).await?;
            Ok(content)
        } else {
            Err(TemplateError::TemplateNotFound(file_name))
        }
    }

    /// Render an issue using a template
    pub async fn render_issue(
        &self,
        project_path: &Path,
        template_name: &str,
        context: &IssueTemplateContext,
    ) -> Result<String, TemplateError> {
        let template_content = self
            .load_template(project_path, TemplateType::Issue, template_name)
            .await?;

        self.handlebars
            .render_template(&template_content, context)
            .map_err(TemplateError::from)
    }

    /// Render a doc using a template
    pub async fn render_doc(
        &self,
        project_path: &Path,
        template_name: &str,
        context: &DocTemplateContext,
    ) -> Result<String, TemplateError> {
        let template_content = self
            .load_template(project_path, TemplateType::Doc, template_name)
            .await?;

        self.handlebars
            .render_template(&template_content, context)
            .map_err(TemplateError::from)
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_type_folder_name() {
        assert_eq!(TemplateType::Issue.folder_name(), "issues");
        assert_eq!(TemplateType::Doc.folder_name(), "docs");
    }

    #[test]
    fn test_template_engine_creation() {
        let engine = TemplateEngine::new();
        // Basic test that engine can be created
        assert!(engine.handlebars.get_templates().is_empty());
    }

    #[test]
    fn test_get_templates_path() {
        let project_path = Path::new("/test/project");
        let templates_path = TemplateEngine::get_templates_path(project_path);
        assert_eq!(templates_path, Path::new("/test/project/.centy/templates"));
    }

    #[test]
    fn test_get_template_type_path() {
        let project_path = Path::new("/test/project");

        let issues_path = TemplateEngine::get_template_type_path(project_path, TemplateType::Issue);
        assert_eq!(
            issues_path,
            Path::new("/test/project/.centy/templates/issues")
        );

        let docs_path = TemplateEngine::get_template_type_path(project_path, TemplateType::Doc);
        assert_eq!(docs_path, Path::new("/test/project/.centy/templates/docs"));
    }
}
