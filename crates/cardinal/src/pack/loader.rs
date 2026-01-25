use anyhow::{Context, Result};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use super::metadata::Manifest;

/// Load a .ccpack file into memory and return the manifest and file contents
///
/// # Arguments
/// * `ccpack_path` - Path to the .ccpack file
///
/// # Returns
/// A tuple of (Manifest, HashMap<path, content_bytes>)
pub fn load_pack<P: AsRef<Path>>(ccpack_path: P) -> Result<(Manifest, HashMap<String, Vec<u8>>)> {
    let ccpack_path = ccpack_path.as_ref();

    // Read and decompress the pack file
    let compressed_data = std::fs::read(ccpack_path)
        .with_context(|| format!("Failed to read pack file {}", ccpack_path.display()))?;

    let tar_data = zstd::decode_all(&compressed_data[..])
        .context("Failed to decompress pack file with zstd")?;

    // Extract tar archive
    let mut archive = tar::Archive::new(&tar_data[..]);
    let mut files = HashMap::new();
    let mut manifest_content = None;

    for entry in archive.entries().context("Failed to read tar entries")? {
        let mut entry = entry.context("Failed to read tar entry")?;
        let path = entry
            .path()
            .context("Failed to get entry path")?
            .to_string_lossy()
            .to_string();

        let mut content = Vec::new();
        entry
            .read_to_end(&mut content)
            .with_context(|| format!("Failed to read content of {}", path))?;

        if path == "manifest.toml" {
            manifest_content = Some(content.clone());
        }

        files.insert(path, content);
    }

    // Parse manifest
    let manifest_bytes = manifest_content
        .ok_or_else(|| anyhow::anyhow!("manifest.toml not found in pack"))?;

    let manifest_str = String::from_utf8(manifest_bytes)
        .context("manifest.toml is not valid UTF-8")?;

    let manifest: Manifest = toml::from_str(&manifest_str)
        .context("Failed to parse manifest.toml")?;

    Ok((manifest, files))
}

/// List the contents of a .ccpack file
///
/// # Arguments
/// * `ccpack_path` - Path to the .ccpack file
///
/// Prints information about the pack to stdout
pub fn list_pack<P: AsRef<Path>>(ccpack_path: P) -> Result<()> {
    let ccpack_path = ccpack_path.as_ref();

    let (manifest, _files) = load_pack(ccpack_path)
        .with_context(|| format!("Failed to load pack {}", ccpack_path.display()))?;

    println!("Pack: {}", manifest.pack.pack_id);
    println!("Version: {}", manifest.pack.version);

    if let Some(name) = &manifest.pack.name {
        println!("Name: {}", name);
    }

    if let Some(desc) = &manifest.pack.description {
        println!("Description: {}", desc);
    }

    if !manifest.pack.dependencies.is_empty() {
        println!("Dependencies:");
        for dep in &manifest.pack.dependencies {
            println!("  - {}", dep);
        }
    }

    println!();
    println!("Files ({}):", manifest.files.len());

    for file_entry in &manifest.files {
        println!(
            "  {} ({} bytes, sha256: {})",
            file_entry.path, file_entry.size, file_entry.sha256
        );
    }

    Ok(())
}

/// Unpack a .ccpack file to a directory
///
/// # Arguments
/// * `ccpack_path` - Path to the .ccpack file
/// * `output_dir` - Directory where files will be extracted
///
/// Extracts all files from the pack to the output directory
pub fn unpack_pack<P: AsRef<Path>, Q: AsRef<Path>>(ccpack_path: P, output_dir: Q) -> Result<()> {
    let ccpack_path = ccpack_path.as_ref();
    let output_dir = output_dir.as_ref();

    let (_manifest, files) = load_pack(ccpack_path)
        .with_context(|| format!("Failed to load pack {}", ccpack_path.display()))?;

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output directory {}", output_dir.display()))?;

    // Extract all files
    for (path, content) in &files {
        let output_path = output_dir.join(path);

        // Create parent directories if needed
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }

        std::fs::write(&output_path, content)
            .with_context(|| format!("Failed to write file {}", output_path.display()))?;

        println!("  Extracted: {}", path);
    }

    println!("✓ Pack unpacked to: {}", output_dir.display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pack::builder::build_pack;
    use crate::pack::metadata::PackMeta;
    use std::fs;

    #[test]
    fn test_pack_roundtrip() {
        // Create a temporary test pack directory
        let temp_dir = std::env::temp_dir().join("test_pack");
        let _ = fs::remove_dir_all(&temp_dir); // Clean up if exists
        fs::create_dir_all(&temp_dir).unwrap();

        // Create pack.toml
        let pack_meta = PackMeta {
            pack_id: "test-pack".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec![],
            name: Some("Test Pack".to_string()),
            description: Some("A test pack".to_string()),
        };

        let pack_toml = toml::to_string(&pack_meta).unwrap();
        fs::write(temp_dir.join("pack.toml"), pack_toml).unwrap();

        // Create some test files
        fs::create_dir_all(temp_dir.join("cards")).unwrap();
        fs::write(temp_dir.join("cards/test_card.toml"), "name = \"Test Card\"\n").unwrap();

        fs::create_dir_all(temp_dir.join("scripts")).unwrap();
        fs::write(temp_dir.join("scripts/test.rhai"), "// Test script\n").unwrap();

        // Build pack
        let pack_path = temp_dir.join("test.ccpack");
        build_pack(&temp_dir, &pack_path).unwrap();

        // Load pack
        let (manifest, files) = load_pack(&pack_path).unwrap();

        assert_eq!(manifest.pack.pack_id, "test-pack");
        assert_eq!(manifest.pack.version, "1.0.0");
        assert!(files.contains_key("pack.toml"));
        assert!(files.contains_key("cards/test_card.toml"));
        assert!(files.contains_key("scripts/test.rhai"));
        assert!(files.contains_key("manifest.toml"));

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
