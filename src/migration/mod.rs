//! Migration system for the Centy daemon.
//!
//! This module provides a versioning and migration system that allows
//! projects to be upgraded or downgraded between daemon versions.
//!
//! # Overview
//!
//! - Migrations are registered in a `MigrationRegistry`
//! - The `MigrationExecutor` finds the path between versions and executes migrations
//! - Each migration implements the `Migration` trait with `up()` and `down()` methods
//! - Migrations are reversible for downgrade support
//! - On failure, applied migrations are automatically rolled back
//!
//! # Usage
//!
//! ```ignore
//! let registry = create_registry();
//! let executor = MigrationExecutor::new(registry);
//! let result = executor.migrate(project_path, &target_version).await?;
//! ```

mod executor;
pub mod migrations;
mod registry;
mod types;

pub use executor::MigrationExecutor;
pub use registry::MigrationRegistry;
pub use types::{Migration, MigrationDirection, MigrationError, MigrationResult};

use migrations::v0_1_0::InitialVersionMigration;
use std::sync::Arc;

/// Create the default migration registry with all available migrations.
///
/// This function creates a new registry and registers all migrations
/// in the correct order. Call this to get the standard registry for
/// migration operations.
pub fn create_registry() -> Arc<MigrationRegistry> {
    let mut registry = MigrationRegistry::new();

    // Register all migrations in order
    registry.register(Arc::new(InitialVersionMigration::new()));

    // Future migrations will be added here:
    // registry.register(Arc::new(migrations::v0_2_0::SomeMigration::new()));

    Arc::new(registry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::SemVer;

    #[test]
    fn test_create_registry() {
        let registry = create_registry();
        let versions = registry.available_versions();

        // Should have at least the initial versions
        assert!(versions.contains(&"0.0.0".to_string()));
        assert!(versions.contains(&"0.1.0".to_string()));
    }

    #[test]
    fn test_migration_path_exists() {
        let registry = create_registry();
        let from = SemVer::new(0, 0, 0);
        let to = SemVer::new(0, 1, 0);

        let result = registry.get_migration_path(&from, &to);
        assert!(result.is_ok());

        let (path, direction) = result.unwrap();
        assert_eq!(path.len(), 1);
        assert_eq!(direction, MigrationDirection::Up);
    }
}
