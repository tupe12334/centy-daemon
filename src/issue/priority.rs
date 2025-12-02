use thiserror::Error;

#[derive(Error, Debug)]
pub enum PriorityError {
    #[error("Priority {0} is out of range. Valid values: 1 to {1}")]
    OutOfRange(u32, u32),

    #[error("Unknown priority label: {0}")]
    UnknownLabel(String),
}

/// Validate that priority is within the configured range
pub fn validate_priority(priority: u32, max_levels: u32) -> Result<(), PriorityError> {
    if priority < 1 || priority > max_levels {
        return Err(PriorityError::OutOfRange(priority, max_levels));
    }
    Ok(())
}

/// Get the default priority (middle value, or lower-middle for even counts)
///
/// Examples:
/// - priority_levels=1 -> 1
/// - priority_levels=2 -> 1 (high)
/// - priority_levels=3 -> 2 (medium)
/// - priority_levels=4 -> 2 (high-medium)
/// - priority_levels=5 -> 3 (medium)
pub fn default_priority(priority_levels: u32) -> u32 {
    if priority_levels == 0 {
        return 1;
    }
    (priority_levels + 1) / 2
}

/// Get a human-readable label for a priority level
///
/// Label mapping:
/// - 1 level: "normal"
/// - 2 levels: high (1), low (2)
/// - 3 levels: high (1), medium (2), low (3)
/// - 4 levels: critical (1), high (2), medium (3), low (4)
/// - 5+ levels: P1, P2, P3, etc.
pub fn priority_label(priority: u32, max_levels: u32) -> String {
    match max_levels {
        0 | 1 => "normal".to_string(),
        2 => match priority {
            1 => "high".to_string(),
            _ => "low".to_string(),
        },
        3 => match priority {
            1 => "high".to_string(),
            2 => "medium".to_string(),
            _ => "low".to_string(),
        },
        4 => match priority {
            1 => "critical".to_string(),
            2 => "high".to_string(),
            3 => "medium".to_string(),
            _ => "low".to_string(),
        },
        _ => format!("P{}", priority),
    }
}

/// Convert a string priority label to a numeric priority
/// Returns None if the label is not recognized
pub fn label_to_priority(label: &str, max_levels: u32) -> Option<u32> {
    match label.to_lowercase().as_str() {
        "critical" | "urgent" => Some(1),
        "high" => {
            if max_levels >= 4 {
                Some(2)
            } else {
                Some(1)
            }
        }
        "medium" | "normal" => Some(default_priority(max_levels)),
        "low" => Some(max_levels),
        _ => {
            // Try parsing as P{N} format
            if let Some(stripped) = label.strip_prefix('P').or_else(|| label.strip_prefix('p')) {
                stripped.parse::<u32>().ok()
            } else {
                // Try parsing as plain number
                label.parse::<u32>().ok()
            }
        }
    }
}

/// Migrate a string-based priority to the numeric system
///
/// Handles legacy priority strings like "high", "medium", "low"
/// and falls back to the default priority for unrecognized values
pub fn migrate_string_priority(priority_str: &str, max_levels: u32) -> u32 {
    label_to_priority(priority_str, max_levels).unwrap_or_else(|| default_priority(max_levels))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_priority_valid() {
        assert!(validate_priority(1, 3).is_ok());
        assert!(validate_priority(2, 3).is_ok());
        assert!(validate_priority(3, 3).is_ok());
    }

    #[test]
    fn test_validate_priority_out_of_range() {
        assert!(validate_priority(0, 3).is_err());
        assert!(validate_priority(4, 3).is_err());
        assert!(validate_priority(10, 3).is_err());
    }

    #[test]
    fn test_validate_priority_single_level() {
        assert!(validate_priority(1, 1).is_ok());
        assert!(validate_priority(0, 1).is_err());
        assert!(validate_priority(2, 1).is_err());
    }

    #[test]
    fn test_default_priority() {
        assert_eq!(default_priority(1), 1);
        assert_eq!(default_priority(2), 1);
        assert_eq!(default_priority(3), 2);
        assert_eq!(default_priority(4), 2);
        assert_eq!(default_priority(5), 3);
        assert_eq!(default_priority(6), 3);
        assert_eq!(default_priority(10), 5);
    }

    #[test]
    fn test_default_priority_edge_case() {
        // Zero should return 1 (safe default)
        assert_eq!(default_priority(0), 1);
    }

    #[test]
    fn test_priority_label_1_level() {
        assert_eq!(priority_label(1, 1), "normal");
    }

    #[test]
    fn test_priority_label_2_levels() {
        assert_eq!(priority_label(1, 2), "high");
        assert_eq!(priority_label(2, 2), "low");
    }

    #[test]
    fn test_priority_label_3_levels() {
        assert_eq!(priority_label(1, 3), "high");
        assert_eq!(priority_label(2, 3), "medium");
        assert_eq!(priority_label(3, 3), "low");
    }

    #[test]
    fn test_priority_label_4_levels() {
        assert_eq!(priority_label(1, 4), "critical");
        assert_eq!(priority_label(2, 4), "high");
        assert_eq!(priority_label(3, 4), "medium");
        assert_eq!(priority_label(4, 4), "low");
    }

    #[test]
    fn test_priority_label_5_plus_levels() {
        assert_eq!(priority_label(1, 5), "P1");
        assert_eq!(priority_label(2, 5), "P2");
        assert_eq!(priority_label(3, 5), "P3");
        assert_eq!(priority_label(4, 5), "P4");
        assert_eq!(priority_label(5, 5), "P5");
    }

    #[test]
    fn test_label_to_priority_2_levels() {
        assert_eq!(label_to_priority("high", 2), Some(1));
        assert_eq!(label_to_priority("low", 2), Some(2));
    }

    #[test]
    fn test_label_to_priority_3_levels() {
        assert_eq!(label_to_priority("high", 3), Some(1));
        assert_eq!(label_to_priority("medium", 3), Some(2));
        assert_eq!(label_to_priority("low", 3), Some(3));
    }

    #[test]
    fn test_label_to_priority_4_levels() {
        assert_eq!(label_to_priority("critical", 4), Some(1));
        assert_eq!(label_to_priority("urgent", 4), Some(1));
        assert_eq!(label_to_priority("high", 4), Some(2));
        assert_eq!(label_to_priority("medium", 4), Some(2)); // default
        assert_eq!(label_to_priority("low", 4), Some(4));
    }

    #[test]
    fn test_label_to_priority_p_notation() {
        assert_eq!(label_to_priority("P1", 5), Some(1));
        assert_eq!(label_to_priority("P2", 5), Some(2));
        assert_eq!(label_to_priority("p3", 5), Some(3));
    }

    #[test]
    fn test_label_to_priority_numeric_string() {
        assert_eq!(label_to_priority("1", 5), Some(1));
        assert_eq!(label_to_priority("3", 5), Some(3));
    }

    #[test]
    fn test_label_to_priority_case_insensitive() {
        assert_eq!(label_to_priority("HIGH", 3), Some(1));
        assert_eq!(label_to_priority("Medium", 3), Some(2));
        assert_eq!(label_to_priority("LOW", 3), Some(3));
    }

    #[test]
    fn test_label_to_priority_unknown() {
        assert_eq!(label_to_priority("unknown", 3), None);
        assert_eq!(label_to_priority("", 3), None);
    }

    #[test]
    fn test_migrate_string_priority() {
        // Known labels
        assert_eq!(migrate_string_priority("high", 3), 1);
        assert_eq!(migrate_string_priority("medium", 3), 2);
        assert_eq!(migrate_string_priority("low", 3), 3);

        // Unknown falls back to default
        assert_eq!(migrate_string_priority("unknown", 3), 2);
        assert_eq!(migrate_string_priority("", 3), 2);
    }

    #[test]
    fn test_migrate_string_priority_numeric() {
        assert_eq!(migrate_string_priority("1", 3), 1);
        assert_eq!(migrate_string_priority("2", 3), 2);
        assert_eq!(migrate_string_priority("P1", 5), 1);
    }
}
