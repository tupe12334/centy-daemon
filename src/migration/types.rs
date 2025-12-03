//! Types for the migration system.

use crate::version::SemVer;
use async_trait::async_trait;
use std::path::Path;
use thiserror::Error;

/// Error types for migration operations.
#[derive(Error, Debug)]
pub enum MigrationError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Migration {0} failed: {1}")]
    MigrationFailed(String, String),

    #[error("Rollback failed for migration {0}: {1}")]
    RollbackFailed(String, String),

    #[error("Version error: {0}")]
    VersionError(#[from] crate::version::VersionError),

    #[error("No migration path from {0} to {1}")]
    NoMigrationPath(String, String),

    #[error("Config error: {0}")]
    ConfigError(String),
}

/// Trait for a single migration.
///
/// Each migration represents a transformation from one version to another.
/// Migrations must be reversible (implement both up and down).
#[async_trait]
pub trait Migration: Send + Sync {
    /// The version this migration upgrades FROM.
    fn from_version(&self) -> &SemVer;

    /// The version this migration upgrades TO.
    fn to_version(&self) -> &SemVer;

    /// Human-readable description of what this migration does.
    fn description(&self) -> &str;

    /// Apply the migration (upgrade).
    async fn up(&self, project_path: &Path) -> Result<(), MigrationError>;

    /// Revert the migration (downgrade).
    async fn down(&self, project_path: &Path) -> Result<(), MigrationError>;
}

/// Result of migration execution.
#[derive(Debug, Clone)]
pub struct MigrationResult {
    /// Whether the migration was successful.
    pub success: bool,
    /// The version we migrated from.
    pub from_version: String,
    /// The version we migrated to.
    pub to_version: String,
    /// List of migrations that were applied (descriptions).
    pub migrations_applied: Vec<String>,
    /// Error message if migration failed.
    pub error: Option<String>,
}

/// Direction of migration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationDirection {
    /// Upgrading to a newer version.
    Up,
    /// Downgrading to an older version.
    Down,
}
