//! Issue ID utilities for UUID-based folder names.
//!
//! This module provides utilities for generating and validating issue IDs.
//! Issue folders use UUIDs to prevent conflicts when multiple users create
//! issues on different computers.

use uuid::Uuid;

/// Check if a string is a valid UUID
pub fn is_uuid(s: &str) -> bool {
    Uuid::parse_str(s).is_ok()
}

/// Check if a string is a legacy issue number (4 digits like "0001")
pub fn is_legacy_number(s: &str) -> bool {
    s.len() == 4 && s.chars().all(|c| c.is_ascii_digit())
}

/// Check if a folder name is a valid issue folder (UUID or legacy 4-digit)
pub fn is_valid_issue_folder(name: &str) -> bool {
    is_uuid(name) || is_legacy_number(name)
}

/// Generate a new UUID for an issue folder
pub fn generate_issue_id() -> String {
    Uuid::new_v4().to_string()
}

/// Get the short form of an issue ID (first 8 characters)
/// Useful for display purposes
pub fn short_id(id: &str) -> &str {
    if id.len() >= 8 {
        &id[..8]
    } else {
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_uuid_valid() {
        assert!(is_uuid("a3f2b1c9-4d5e-6f7a-8b9c-0d1e2f3a4b5c"));
        assert!(is_uuid("550e8400-e29b-41d4-a716-446655440000"));
    }

    #[test]
    fn test_is_uuid_invalid() {
        assert!(!is_uuid("not-a-uuid"));
        assert!(!is_uuid("0001"));
        assert!(!is_uuid(""));
        assert!(!is_uuid("a3f2b1c9-4d5e-6f7a-8b9c")); // incomplete
    }

    #[test]
    fn test_is_legacy_number_valid() {
        assert!(is_legacy_number("0001"));
        assert!(is_legacy_number("0042"));
        assert!(is_legacy_number("9999"));
    }

    #[test]
    fn test_is_legacy_number_invalid() {
        assert!(!is_legacy_number("001")); // too short
        assert!(!is_legacy_number("00001")); // too long
        assert!(!is_legacy_number("abcd")); // not digits
        assert!(!is_legacy_number("")); // empty
    }

    #[test]
    fn test_is_valid_issue_folder() {
        // Valid UUIDs
        assert!(is_valid_issue_folder("a3f2b1c9-4d5e-6f7a-8b9c-0d1e2f3a4b5c"));
        // Valid legacy numbers
        assert!(is_valid_issue_folder("0001"));
        assert!(is_valid_issue_folder("0042"));
        // Invalid
        assert!(!is_valid_issue_folder("random-folder"));
        assert!(!is_valid_issue_folder(".DS_Store"));
    }

    #[test]
    fn test_generate_issue_id() {
        let id = generate_issue_id();
        assert!(is_uuid(&id));
        // Ensure uniqueness
        let id2 = generate_issue_id();
        assert_ne!(id, id2);
    }

    #[test]
    fn test_short_id() {
        assert_eq!(short_id("a3f2b1c9-4d5e-6f7a-8b9c-0d1e2f3a4b5c"), "a3f2b1c9");
        assert_eq!(short_id("0001"), "0001");
        assert_eq!(short_id("abc"), "abc");
    }
}
