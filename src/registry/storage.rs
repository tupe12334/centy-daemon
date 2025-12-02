use super::types::ProjectRegistry;
use super::RegistryError;
use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::fs;
use tokio::sync::Mutex;

/// Global mutex for registry file access
static REGISTRY_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub fn get_lock() -> &'static Mutex<()> {
    REGISTRY_LOCK.get_or_init(|| Mutex::new(()))
}

/// Get the path to the global centy config directory (~/.centy)
pub fn get_centy_config_dir() -> Result<PathBuf, RegistryError> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| RegistryError::HomeDirNotFound)?;

    Ok(PathBuf::from(home).join(".centy"))
}

/// Get the path to the global registry file (~/.centy/projects.json)
pub fn get_registry_path() -> Result<PathBuf, RegistryError> {
    Ok(get_centy_config_dir()?.join("projects.json"))
}

/// Read the registry from disk
pub async fn read_registry() -> Result<ProjectRegistry, RegistryError> {
    let path = get_registry_path()?;

    if !path.exists() {
        return Ok(ProjectRegistry::new());
    }

    let content = fs::read_to_string(&path).await?;
    let registry: ProjectRegistry = serde_json::from_str(&content)?;
    Ok(registry)
}

/// Write the registry to disk with locking and atomic write
pub async fn write_registry(registry: &ProjectRegistry) -> Result<(), RegistryError> {
    let _guard = get_lock().lock().await;
    write_registry_unlocked(registry).await
}

/// Write the registry to disk without acquiring the lock (caller must hold lock)
pub async fn write_registry_unlocked(registry: &ProjectRegistry) -> Result<(), RegistryError> {
    let path = get_registry_path()?;

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }

    // Write atomically using temp file + rename
    let temp_path = path.with_extension("json.tmp");
    let content = serde_json::to_string_pretty(registry)?;
    fs::write(&temp_path, &content).await?;
    fs::rename(&temp_path, &path).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_registry_path() {
        // This test will work if HOME or USERPROFILE is set
        let result = get_registry_path();
        if std::env::var("HOME").is_ok() || std::env::var("USERPROFILE").is_ok() {
            assert!(result.is_ok());
            let path = result.unwrap();
            assert!(path.ends_with("projects.json"));
            assert!(path.to_string_lossy().contains(".centy"));
        }
    }

    #[test]
    fn test_project_registry_new() {
        let registry = ProjectRegistry::new();
        assert_eq!(registry.schema_version, 1);
        assert!(registry.projects.is_empty());
        assert!(!registry.updated_at.is_empty());
    }
}
