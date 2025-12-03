//! Migration executor for running migrations.

use super::registry::MigrationRegistry;
use super::types::{Migration, MigrationDirection, MigrationError, MigrationResult};
use crate::config::{read_config, write_config};
use crate::version::SemVer;
use std::path::Path;
use std::sync::Arc;
use tracing::{error, info};

/// Executor for running migrations.
///
/// The executor takes a registry of migrations and provides methods
/// to migrate projects between versions.
pub struct MigrationExecutor {
    registry: Arc<MigrationRegistry>,
}

impl MigrationExecutor {
    /// Create a new executor with the given registry.
    pub fn new(registry: Arc<MigrationRegistry>) -> Self {
        Self { registry }
    }

    /// Execute migrations to reach the target version.
    ///
    /// This method:
    /// 1. Reads the current project version from config
    /// 2. Finds the migration path to the target version
    /// 3. Executes each migration in order
    /// 4. Rolls back on failure
    /// 5. Updates the config with the new version on success
    pub async fn migrate(
        &self,
        project_path: &Path,
        target_version: &SemVer,
    ) -> Result<MigrationResult, MigrationError> {
        // Read current config to get project version
        let config = read_config(project_path)
            .await
            .map_err(|e| MigrationError::ConfigError(e.to_string()))?;

        let current_version = config
            .as_ref()
            .and_then(|c| c.version.as_ref())
            .map(|v| SemVer::parse(v))
            .transpose()?
            .unwrap_or_else(|| SemVer::new(0, 0, 0)); // Unversioned projects start at 0.0.0

        info!(
            from = %current_version,
            to = %target_version,
            "Starting migration"
        );

        // Get migration path
        let (migrations, direction) = self
            .registry
            .get_migration_path(&current_version, target_version)?;

        if migrations.is_empty() {
            info!("No migrations needed, already at target version");
            return Ok(MigrationResult {
                success: true,
                from_version: current_version.to_string(),
                to_version: target_version.to_string(),
                migrations_applied: vec![],
                error: None,
            });
        }

        let mut applied: Vec<Arc<dyn Migration>> = Vec::new();

        // Execute migrations
        for migration in &migrations {
            let migration_name = format!(
                "{} -> {}: {}",
                migration.from_version(),
                migration.to_version(),
                migration.description()
            );

            info!(migration = %migration_name, "Applying migration");

            let result = match direction {
                MigrationDirection::Up => migration.up(project_path).await,
                MigrationDirection::Down => migration.down(project_path).await,
            };

            if let Err(e) = result {
                error!(migration = %migration_name, error = %e, "Migration failed");

                // Rollback applied migrations
                for applied_migration in applied.iter().rev() {
                    let rollback_name = format!(
                        "{} -> {}",
                        applied_migration.from_version(),
                        applied_migration.to_version()
                    );
                    info!(migration = %rollback_name, "Rolling back migration");

                    if let Err(rollback_err) =
                        self.rollback_migration(project_path, applied_migration, direction).await
                    {
                        error!(
                            migration = %rollback_name,
                            error = %rollback_err,
                            "Rollback failed"
                        );
                    }
                }

                return Ok(MigrationResult {
                    success: false,
                    from_version: current_version.to_string(),
                    to_version: target_version.to_string(),
                    migrations_applied: vec![],
                    error: Some(format!("Migration {} failed: {}", migration_name, e)),
                });
            }

            applied.push(Arc::clone(migration));
        }

        // Update config with new version
        let mut config = config.unwrap_or_default();
        config.version = Some(target_version.to_string());
        write_config(project_path, &config)
            .await
            .map_err(|e| MigrationError::ConfigError(e.to_string()))?;

        info!(
            from = %current_version,
            to = %target_version,
            count = applied.len(),
            "Migration completed successfully"
        );

        Ok(MigrationResult {
            success: true,
            from_version: current_version.to_string(),
            to_version: target_version.to_string(),
            migrations_applied: applied
                .iter()
                .map(|m| {
                    format!(
                        "{} -> {}: {}",
                        m.from_version(),
                        m.to_version(),
                        m.description()
                    )
                })
                .collect(),
            error: None,
        })
    }

    /// Rollback a single migration.
    async fn rollback_migration(
        &self,
        project_path: &Path,
        migration: &Arc<dyn Migration>,
        direction: MigrationDirection,
    ) -> Result<(), MigrationError> {
        // To rollback, we do the opposite operation
        match direction {
            MigrationDirection::Up => migration.down(project_path).await,
            MigrationDirection::Down => migration.up(project_path).await,
        }
    }
}
