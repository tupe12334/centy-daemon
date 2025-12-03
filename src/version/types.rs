//! Version types for semantic versioning support.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use thiserror::Error;

/// Error types for version operations.
#[derive(Error, Debug)]
pub enum VersionError {
    #[error("Invalid version format: {0}")]
    InvalidFormat(String),

    #[error("Version not found in config")]
    NotFound,
}

/// Represents a semantic version (major.minor.patch).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemVer {
    /// Create a new SemVer instance.
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Parse a version string (e.g., "1.2.3") into a SemVer.
    pub fn parse(s: &str) -> Result<Self, VersionError> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(VersionError::InvalidFormat(s.to_string()));
        }

        let major = parts[0]
            .parse()
            .map_err(|_| VersionError::InvalidFormat(s.to_string()))?;
        let minor = parts[1]
            .parse()
            .map_err(|_| VersionError::InvalidFormat(s.to_string()))?;
        let patch = parts[2]
            .parse()
            .map_err(|_| VersionError::InvalidFormat(s.to_string()))?;

        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

impl fmt::Display for SemVer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Ord for SemVer {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => self.patch.cmp(&other.patch),
                other => other,
            },
            other => other,
        }
    }
}

impl PartialOrd for SemVer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Result of comparing project version against daemon version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionComparison {
    /// Project is at same version as daemon.
    Equal,
    /// Project is older than daemon (can upgrade).
    ProjectBehind,
    /// Project is newer than daemon (degraded mode).
    ProjectAhead,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_parse_valid() {
        let v = SemVer::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_semver_parse_zero() {
        let v = SemVer::parse("0.0.0").unwrap();
        assert_eq!(v.major, 0);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
    }

    #[test]
    fn test_semver_parse_invalid_format() {
        assert!(SemVer::parse("1.2").is_err());
        assert!(SemVer::parse("1").is_err());
        assert!(SemVer::parse("1.2.3.4").is_err());
        assert!(SemVer::parse("").is_err());
    }

    #[test]
    fn test_semver_parse_invalid_number() {
        assert!(SemVer::parse("a.b.c").is_err());
        assert!(SemVer::parse("1.2.x").is_err());
    }

    #[test]
    fn test_semver_display() {
        let v = SemVer::new(1, 2, 3);
        assert_eq!(v.to_string(), "1.2.3");
    }

    #[test]
    fn test_semver_comparison() {
        let v1 = SemVer::new(1, 0, 0);
        let v2 = SemVer::new(2, 0, 0);
        let v3 = SemVer::new(1, 1, 0);
        let v4 = SemVer::new(1, 0, 1);
        let v5 = SemVer::new(1, 0, 0);

        assert!(v1 < v2);
        assert!(v1 < v3);
        assert!(v1 < v4);
        assert!(v1 == v5);
        assert!(v3 < v2);
        assert!(v4 < v3);
    }

    #[test]
    fn test_semver_ordering() {
        let mut versions = vec![
            SemVer::new(2, 0, 0),
            SemVer::new(0, 1, 0),
            SemVer::new(1, 0, 0),
            SemVer::new(1, 1, 0),
        ];
        versions.sort();

        assert_eq!(versions[0], SemVer::new(0, 1, 0));
        assert_eq!(versions[1], SemVer::new(1, 0, 0));
        assert_eq!(versions[2], SemVer::new(1, 1, 0));
        assert_eq!(versions[3], SemVer::new(2, 0, 0));
    }
}
