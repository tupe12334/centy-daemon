//! Version management for the Centy daemon.
//!
//! This module provides semantic versioning support, version comparison,
//! and utilities for checking project version compatibility.

mod types;

pub use types::{SemVer, VersionComparison, VersionError};

use crate::config::read_config;
use crate::utils::CENTY_VERSION;
use std::path::Path;
use tracing::warn;

/// Get the current daemon version as a SemVer.
pub fn daemon_version() -> SemVer {
    SemVer::parse(CENTY_VERSION).expect("CENTY_VERSION should be valid semver")
}

/// Compare project version against daemon version.
pub fn compare_versions(project_version: &SemVer, daemon_version: &SemVer) -> VersionComparison {
    match project_version.cmp(daemon_version) {
        std::cmp::Ordering::Equal => VersionComparison::Equal,
        std::cmp::Ordering::Less => VersionComparison::ProjectBehind,
        std::cmp::Ordering::Greater => VersionComparison::ProjectAhead,
    }
}

/// Check version compatibility and log warning if in degraded mode.
///
/// This function reads the project's config, compares versions, and logs
/// a warning if the project version is newer than the daemon version.
///
/// Returns the version comparison result.
pub async fn check_version_for_operation(project_path: &Path) -> VersionComparison {
    let daemon_ver = daemon_version();

    if let Ok(Some(config)) = read_config(project_path).await {
        if let Some(version_str) = &config.version {
            if let Ok(project_ver) = SemVer::parse(version_str) {
                let comparison = compare_versions(&project_ver, &daemon_ver);

                if comparison == VersionComparison::ProjectAhead {
                    warn!(
                        project_version = %project_ver,
                        daemon_version = %daemon_ver,
                        "Project version is newer than daemon. Operating in degraded mode."
                    );
                }

                return comparison;
            }
        }
    }

    // No version in config means it defaults to daemon version
    VersionComparison::Equal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_version() {
        let v = daemon_version();
        // Should parse without panicking
        assert!(v.major >= 0);
    }

    #[test]
    fn test_compare_versions_equal() {
        let v = SemVer::new(1, 0, 0);
        assert_eq!(compare_versions(&v, &v), VersionComparison::Equal);
    }

    #[test]
    fn test_compare_versions_project_behind() {
        let project = SemVer::new(0, 9, 0);
        let daemon = SemVer::new(1, 0, 0);
        assert_eq!(
            compare_versions(&project, &daemon),
            VersionComparison::ProjectBehind
        );
    }

    #[test]
    fn test_compare_versions_project_ahead() {
        let project = SemVer::new(2, 0, 0);
        let daemon = SemVer::new(1, 0, 0);
        assert_eq!(
            compare_versions(&project, &daemon),
            VersionComparison::ProjectAhead
        );
    }
}
