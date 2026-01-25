use serde::{Deserialize, Serialize};

/// Metadata for a card pack, stored in pack.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackMeta {
    /// Unique identifier for this pack (e.g., "core-set", "expansion-1")
    pub pack_id: String,
    
    /// Semantic version of this pack (e.g., "1.0.0")
    pub version: String,
    
    /// Optional list of pack dependencies (pack_id strings)
    #[serde(default)]
    pub dependencies: Vec<String>,
    
    /// Optional human-readable name
    #[serde(default)]
    pub name: Option<String>,
    
    /// Optional description
    #[serde(default)]
    pub description: Option<String>,
}

/// A single file entry in the manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// Relative path inside the pack (normalized with forward slashes)
    pub path: String,
    
    /// File size in bytes
    pub size: u64,
    
    /// SHA-256 hash of the file content (hex string)
    pub sha256: String,
}

/// The manifest.toml file generated and included in each pack
/// Lists all files with their metadata for verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Pack metadata (copy of pack.toml for convenience)
    pub pack: PackMeta,
    
    /// List of all files in the pack
    pub files: Vec<FileEntry>,
}
