use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CentyManifest {
    pub schema_version: u32,
    pub centy_version: String,
    pub created_at: String,
    pub updated_at: String,
    pub managed_files: Vec<ManagedFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ManagedFile {
    pub path: String,
    pub hash: String,
    pub version: String,
    pub created_at: String,
    #[serde(rename = "type")]
    pub file_type: ManagedFileType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ManagedFileType {
    File,
    Directory,
}
