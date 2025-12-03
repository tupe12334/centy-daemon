use tracing::warn;

/// Validate that a status is in the allowed states list.
/// This is lenient: it logs a warning but accepts unknown states.
///
/// Returns `true` if the status is valid, `false` if not (but still accepted).
pub fn validate_status(status: &str, allowed_states: &[String]) -> bool {
    let is_valid = allowed_states.iter().any(|s| s == status);
    if !is_valid {
        warn!(
            status = %status,
            allowed = ?allowed_states,
            "Status '{}' is not in the allowed states list. Accepting anyway.",
            status
        );
    }
    is_valid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_status_valid() {
        let allowed = vec!["open".to_string(), "closed".to_string()];
        assert!(validate_status("open", &allowed));
        assert!(validate_status("closed", &allowed));
    }

    #[test]
    fn test_validate_status_invalid_returns_false() {
        let allowed = vec!["open".to_string(), "closed".to_string()];
        // Should return false but not error
        assert!(!validate_status("unknown", &allowed));
    }

    #[test]
    fn test_validate_status_empty_allowed() {
        let allowed: Vec<String> = vec![];
        // Any status is invalid when allowed_states is empty
        assert!(!validate_status("open", &allowed));
    }

    #[test]
    fn test_validate_status_case_sensitive() {
        let allowed = vec!["open".to_string(), "closed".to_string()];
        // Should be case-sensitive
        assert!(!validate_status("Open", &allowed));
        assert!(!validate_status("CLOSED", &allowed));
    }
}
