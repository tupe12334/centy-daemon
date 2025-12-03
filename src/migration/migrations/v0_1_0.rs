//! Initial migration - sets version field for existing projects.
//!
//! This migration handles the transition from unversioned projects (0.0.0)
//! to the first versioned state (0.1.0).

use crate::migration::types::{Migration, MigrationError};
use crate::version::SemVer;
use async_trait::async_trait;
use once_cell::sync::Lazy;
use std::path::Path;

/// Lazy static for the "from" version (0.0.0).
static FROM_VERSION: Lazy<SemVer> = Lazy::new(|| SemVer::new(0, 0, 0));

/// Lazy static for the "to" version (0.1.0).
static TO_VERSION: Lazy<SemVer> = Lazy::new(|| SemVer::new(0, 1, 0));

/// Initial migration that establishes version tracking for existing projects.
///
/// This migration doesn't transform any data - it simply establishes the
/// version tracking system. The version field in config.json is set by
/// the executor after successful migration.
pub struct InitialVersionMigration;

impl InitialVersionMigration {
    /// Create a new initial version migration.
    pub fn new() -> Self {
        Self
    }
}

impl Default for InitialVersionMigration {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Migration for InitialVersionMigration {
    fn from_version(&self) -> &SemVer {
        &FROM_VERSION
    }

    fn to_version(&self) -> &SemVer {
        &TO_VERSION
    }

    fn description(&self) -> &str {
        "Initialize version tracking for existing projects"
    }

    async fn up(&self, _project_path: &Path) -> Result<(), MigrationError> {
        // No data transformation needed for initial version.
        // The version field in config.json is set by the executor.
        Ok(())
    }

    async fn down(&self, _project_path: &Path) -> Result<(), MigrationError> {
        // Downgrade removes version field - handled by executor.
        // No data transformation needed.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_versions() {
        let migration = InitialVersionMigration::new();
        assert_eq!(migration.from_version(), &SemVer::new(0, 0, 0));
        assert_eq!(migration.to_version(), &SemVer::new(0, 1, 0));
    }

    #[test]
    fn test_migration_description() {
        let migration = InitialVersionMigration::new();
        assert!(!migration.description().is_empty());
    }

    #[tokio::test]
    async fn test_migration_up() {
        let migration = InitialVersionMigration::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let result = migration.up(temp_dir.path()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_migration_down() {
        let migration = InitialVersionMigration::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let result = migration.down(temp_dir.path()).await;
        assert!(result.is_ok());
    }
}
