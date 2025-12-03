use crate::utils::get_centy_path;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;
use tokio::fs;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Custom field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomFieldDefinition {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enum_values: Vec<String>,
}

/// Default priority levels (3 = high/medium/low)
fn default_priority_levels() -> u32 {
    3
}

/// Default allowed states for issues
fn default_allowed_states() -> Vec<String> {
    vec![
        "open".to_string(),
        "in-progress".to_string(),
        "closed".to_string(),
    ]
}

/// Default state for new issues
fn default_state() -> String {
    "open".to_string()
}

/// LLM configuration for automated issue management
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LlmConfig {
    /// If true, LLM should auto-close issues after completing the work
    #[serde(default)]
    pub auto_close_on_complete: bool,
    /// If true, LLM should update status to "in-progress" when starting work
    #[serde(default)]
    pub update_status_on_start: bool,
    /// If true, LLM may directly edit metadata.json files. If false, use centy CLI
    #[serde(default)]
    pub allow_direct_edits: bool,
}

/// Centy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CentyConfig {
    /// Project version (semver string). Defaults to daemon version if not set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Number of priority levels (1-10). Default is 3 (high/medium/low).
    /// - 2 levels: high, low
    /// - 3 levels: high, medium, low
    /// - 4 levels: critical, high, medium, low
    /// - 5+ levels: P1, P2, P3, etc.
    #[serde(default = "default_priority_levels")]
    pub priority_levels: u32,
    #[serde(default)]
    pub custom_fields: Vec<CustomFieldDefinition>,
    #[serde(default)]
    pub defaults: HashMap<String, String>,
    /// Allowed status values for issues (default: ["open", "in-progress", "closed"])
    #[serde(default = "default_allowed_states")]
    pub allowed_states: Vec<String>,
    /// Default state for new issues (default: "open")
    #[serde(default = "default_state")]
    pub default_state: String,
    /// State colors: state name → hex color (e.g., "open" → "#10b981")
    #[serde(default)]
    pub state_colors: HashMap<String, String>,
    /// Priority colors: priority level → hex color (e.g., "1" → "#ef4444")
    #[serde(default)]
    pub priority_colors: HashMap<String, String>,
    /// LLM configuration for automated issue management
    #[serde(default)]
    pub llm: LlmConfig,
}

impl CentyConfig {
    /// Get the effective version (config version or daemon default).
    pub fn effective_version(&self) -> String {
        self.version
            .clone()
            .unwrap_or_else(|| crate::utils::CENTY_VERSION.to_string())
    }
}

impl Default for CentyConfig {
    fn default() -> Self {
        Self {
            version: None,
            priority_levels: default_priority_levels(),
            custom_fields: Vec::new(),
            defaults: HashMap::new(),
            allowed_states: default_allowed_states(),
            default_state: default_state(),
            state_colors: HashMap::new(),
            priority_colors: HashMap::new(),
            llm: LlmConfig::default(),
        }
    }
}

/// Read the configuration file
pub async fn read_config(project_path: &Path) -> Result<Option<CentyConfig>, ConfigError> {
    let config_path = get_centy_path(project_path).join("config.json");

    if !config_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&config_path).await?;
    let config: CentyConfig = serde_json::from_str(&content)?;
    Ok(Some(config))
}

/// Write the configuration file
pub async fn write_config(project_path: &Path, config: &CentyConfig) -> Result<(), ConfigError> {
    let config_path = get_centy_path(project_path).join("config.json");
    let content = serde_json::to_string_pretty(config)?;
    fs::write(&config_path, content).await?;
    Ok(())
}
