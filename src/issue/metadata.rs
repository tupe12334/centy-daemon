use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueMetadata {
    pub status: String,
    pub priority: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom_fields: HashMap<String, serde_json::Value>,
}

impl IssueMetadata {
    pub fn new(status: String, priority: String, custom_fields: HashMap<String, serde_json::Value>) -> Self {
        let now = crate::utils::now_iso();
        Self {
            status,
            priority,
            created_at: now.clone(),
            updated_at: now,
            custom_fields,
        }
    }
}
