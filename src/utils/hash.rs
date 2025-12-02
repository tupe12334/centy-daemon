use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::fs;

/// Compute SHA-256 hash of a string
pub fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

/// Compute SHA-256 hash of a file's contents
pub async fn compute_file_hash(path: &Path) -> Result<String, std::io::Error> {
    let content = fs::read_to_string(path).await?;
    Ok(compute_hash(&content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash() {
        let hash = compute_hash("hello world");
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }
}
