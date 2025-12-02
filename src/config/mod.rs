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

/// Centy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CentyConfig {
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
}

impl Default for CentyConfig {
    fn default() -> Self {
        Self {
            priority_levels: default_priority_levels(),
            custom_fields: Vec::new(),
            defaults: HashMap::new(),
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
