//! Migration registry for managing and querying available migrations.

use super::types::{Migration, MigrationDirection, MigrationError};
use crate::version::SemVer;
use std::sync::Arc;

/// Registry of all available migrations.
///
/// The registry maintains an ordered list of migrations and provides
/// utilities to find migration paths between versions.
pub struct MigrationRegistry {
    migrations: Vec<Arc<dyn Migration>>,
}

impl MigrationRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            migrations: Vec::new(),
        }
    }

    /// Register a migration.
    ///
    /// Migrations are automatically sorted by from_version after registration.
    pub fn register(&mut self, migration: Arc<dyn Migration>) {
        self.migrations.push(migration);
        // Keep sorted by from_version for efficient path finding
        self.migrations
            .sort_by(|a, b| a.from_version().cmp(b.from_version()));
    }

    /// Get all available versions that can be migrated to.
    pub fn available_versions(&self) -> Vec<String> {
        let mut versions: Vec<SemVer> = self
            .migrations
            .iter()
            .flat_map(|m| vec![m.from_version().clone(), m.to_version().clone()])
            .collect();

        versions.sort();
        versions.dedup();
        versions.iter().map(|v| v.to_string()).collect()
    }

    /// Get the migrations needed to go from one version to another.
    ///
    /// Returns a tuple of (migrations, direction) where migrations is the
    /// ordered list of migrations to apply and direction indicates whether
    /// we're upgrading or downgrading.
    pub fn get_migration_path(
        &self,
        from: &SemVer,
        to: &SemVer,
    ) -> Result<(Vec<Arc<dyn Migration>>, MigrationDirection), MigrationError> {
        if from == to {
            return Ok((Vec::new(), MigrationDirection::Up));
        }

        let direction = if from < to {
            MigrationDirection::Up
        } else {
            MigrationDirection::Down
        };

        let mut path = Vec::new();
        let mut current = from.clone();

        match direction {
            MigrationDirection::Up => {
                while &current < to {
                    let next_migration = self
                        .migrations
                        .iter()
                        .find(|m| m.from_version() == &current)
                        .ok_or_else(|| {
                            MigrationError::NoMigrationPath(current.to_string(), to.to_string())
                        })?;

                    current = next_migration.to_version().clone();
                    path.push(Arc::clone(next_migration));
                }
            }
            MigrationDirection::Down => {
                while &current > to {
                    let prev_migration = self
                        .migrations
                        .iter()
                        .find(|m| m.to_version() == &current)
                        .ok_or_else(|| {
                            MigrationError::NoMigrationPath(current.to_string(), to.to_string())
                        })?;

                    current = prev_migration.from_version().clone();
                    path.push(Arc::clone(prev_migration));
                }
            }
        }

        Ok((path, direction))
    }
}

impl Default for MigrationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock migration for testing
    struct MockMigration {
        from: SemVer,
        to: SemVer,
    }

    #[async_trait::async_trait]
    impl Migration for MockMigration {
        fn from_version(&self) -> &SemVer {
            &self.from
        }

        fn to_version(&self) -> &SemVer {
            &self.to
        }

        fn description(&self) -> &str {
            "Mock migration"
        }

        async fn up(&self, _project_path: &std::path::Path) -> Result<(), MigrationError> {
            Ok(())
        }

        async fn down(&self, _project_path: &std::path::Path) -> Result<(), MigrationError> {
            Ok(())
        }
    }

    #[test]
    fn test_empty_registry() {
        let registry = MigrationRegistry::new();
        let from = SemVer::new(0, 0, 0);
        let to = SemVer::new(0, 0, 0);

        let (path, _) = registry.get_migration_path(&from, &to).unwrap();
        assert!(path.is_empty());
    }

    #[test]
    fn test_single_upgrade() {
        let mut registry = MigrationRegistry::new();
        registry.register(Arc::new(MockMigration {
            from: SemVer::new(0, 0, 0),
            to: SemVer::new(0, 1, 0),
        }));

        let from = SemVer::new(0, 0, 0);
        let to = SemVer::new(0, 1, 0);

        let (path, direction) = registry.get_migration_path(&from, &to).unwrap();
        assert_eq!(path.len(), 1);
        assert_eq!(direction, MigrationDirection::Up);
    }

    #[test]
    fn test_single_downgrade() {
        let mut registry = MigrationRegistry::new();
        registry.register(Arc::new(MockMigration {
            from: SemVer::new(0, 0, 0),
            to: SemVer::new(0, 1, 0),
        }));

        let from = SemVer::new(0, 1, 0);
        let to = SemVer::new(0, 0, 0);

        let (path, direction) = registry.get_migration_path(&from, &to).unwrap();
        assert_eq!(path.len(), 1);
        assert_eq!(direction, MigrationDirection::Down);
    }

    #[test]
    fn test_multi_step_upgrade() {
        let mut registry = MigrationRegistry::new();
        registry.register(Arc::new(MockMigration {
            from: SemVer::new(0, 0, 0),
            to: SemVer::new(0, 1, 0),
        }));
        registry.register(Arc::new(MockMigration {
            from: SemVer::new(0, 1, 0),
            to: SemVer::new(0, 2, 0),
        }));

        let from = SemVer::new(0, 0, 0);
        let to = SemVer::new(0, 2, 0);

        let (path, direction) = registry.get_migration_path(&from, &to).unwrap();
        assert_eq!(path.len(), 2);
        assert_eq!(direction, MigrationDirection::Up);
    }

    #[test]
    fn test_no_path_error() {
        let registry = MigrationRegistry::new();
        let from = SemVer::new(0, 0, 0);
        let to = SemVer::new(0, 1, 0);

        let result = registry.get_migration_path(&from, &to);
        assert!(result.is_err());
    }

    #[test]
    fn test_available_versions() {
        let mut registry = MigrationRegistry::new();
        registry.register(Arc::new(MockMigration {
            from: SemVer::new(0, 0, 0),
            to: SemVer::new(0, 1, 0),
        }));
        registry.register(Arc::new(MockMigration {
            from: SemVer::new(0, 1, 0),
            to: SemVer::new(0, 2, 0),
        }));

        let versions = registry.available_versions();
        assert_eq!(versions.len(), 3);
        assert!(versions.contains(&"0.0.0".to_string()));
        assert!(versions.contains(&"0.1.0".to_string()));
        assert!(versions.contains(&"0.2.0".to_string()));
    }
}
