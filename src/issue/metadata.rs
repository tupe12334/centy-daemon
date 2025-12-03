use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

use super::priority::migrate_string_priority;

/// Default priority levels for migration when config is not available
const DEFAULT_PRIORITY_LEVELS: u32 = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueMetadata {
    /// Human-readable display number (1, 2, 3...).
    /// Used for user-facing references while folder uses UUID.
    /// Default to 0 for backward compatibility with legacy issues.
    #[serde(default)]
    pub display_number: u32,
    pub status: String,
    /// Priority as a number (1 = highest, N = lowest).
    /// During deserialization, string values are automatically migrated to numbers.
    #[serde(deserialize_with = "deserialize_priority")]
    pub priority: u32,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom_fields: HashMap<String, serde_json::Value>,
}

impl IssueMetadata {
    pub fn new(
        display_number: u32,
        status: String,
        priority: u32,
        custom_fields: HashMap<String, serde_json::Value>,
    ) -> Self {
        let now = crate::utils::now_iso();
        Self {
            display_number,
            status,
            priority,
            created_at: now.clone(),
            updated_at: now,
            custom_fields,
        }
    }
}

/// Custom deserializer that handles both string and number formats for priority.
/// This enables backward compatibility with existing issues that use string priorities.
fn deserialize_priority<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{self, Visitor};

    struct PriorityVisitor;

    impl<'de> Visitor<'de> for PriorityVisitor {
        type Value = u32;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a priority number or string")
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value as u32)
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if value < 0 {
                Err(E::custom("priority cannot be negative"))
            } else {
                Ok(value as u32)
            }
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            // Migrate legacy string priority to number
            // Use default priority levels (3) for migration since we don't have config here
            Ok(migrate_string_priority(value, DEFAULT_PRIORITY_LEVELS))
        }
    }

    deserializer.deserialize_any(PriorityVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_priority_number() {
        let json = r#"{"status":"open","priority":1,"createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
        let metadata: IssueMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.priority, 1);
    }

    #[test]
    fn test_deserialize_priority_string_high() {
        let json = r#"{"status":"open","priority":"high","createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
        let metadata: IssueMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.priority, 1);
    }

    #[test]
    fn test_deserialize_priority_string_medium() {
        let json = r#"{"status":"open","priority":"medium","createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
        let metadata: IssueMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.priority, 2);
    }

    #[test]
    fn test_deserialize_priority_string_low() {
        let json = r#"{"status":"open","priority":"low","createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
        let metadata: IssueMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.priority, 3);
    }

    #[test]
    fn test_serialize_priority_as_number() {
        let metadata = IssueMetadata::new(1, "open".to_string(), 2, HashMap::new());
        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains(r#""priority":2"#));
    }

    #[test]
    fn test_metadata_new() {
        let metadata = IssueMetadata::new(1, "open".to_string(), 1, HashMap::new());
        assert_eq!(metadata.display_number, 1);
        assert_eq!(metadata.status, "open");
        assert_eq!(metadata.priority, 1);
        assert!(!metadata.created_at.is_empty());
        assert!(!metadata.updated_at.is_empty());
    }

    #[test]
    fn test_deserialize_legacy_without_display_number() {
        // Legacy issues without display_number should default to 0
        let json = r#"{"status":"open","priority":1,"createdAt":"2024-01-01","updatedAt":"2024-01-01"}"#;
        let metadata: IssueMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.display_number, 0);
    }

    #[test]
    fn test_serialize_display_number() {
        let metadata = IssueMetadata::new(42, "open".to_string(), 1, HashMap::new());
        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains(r#""displayNumber":42"#));
    }
}
