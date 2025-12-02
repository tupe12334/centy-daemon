use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Stored in registry file (minimal - only timestamps)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackedProject {
    pub first_accessed: String,
    pub last_accessed: String,
}

/// The global project registry stored in ~/.centy/projects.json
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRegistry {
    /// Schema version for future migrations
    pub schema_version: u32,

    /// When the registry was last modified
    pub updated_at: String,

    /// Map of project path -> TrackedProject (timestamps only)
    pub projects: HashMap<String, TrackedProject>,
}

impl ProjectRegistry {
    pub fn new() -> Self {
        Self {
            schema_version: 1,
            updated_at: crate::utils::now_iso(),
            projects: HashMap::new(),
        }
    }
}

/// Returned by API (enriched with live data from disk)
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    /// Absolute path to the project root
    pub path: String,

    /// When the project was first tracked
    pub first_accessed: String,

    /// When the project was last accessed via any RPC
    pub last_accessed: String,

    /// Number of issues in the project (fetched live)
    pub issue_count: u32,

    /// Number of docs in the project (fetched live)
    pub doc_count: u32,

    /// Whether the project has been initialized (fetched live)
    pub initialized: bool,

    /// Project name (directory name, fetched live)
    pub name: Option<String>,
}
