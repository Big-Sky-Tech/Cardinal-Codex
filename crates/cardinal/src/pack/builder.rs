use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::metadata::{FileEntry, Manifest, PackMeta};

/// Build a .ccpack file from a directory
///
/// # Arguments
/// * `input_dir` - Path to the directory containing pack.toml, cards/, scripts/, etc.
/// * `output_file` - Path where the .ccpack file will be written
///
/// # Returns
/// Result indicating success or detailed error
///
/// # Process
/// 1. Validate that pack.toml exists and parse it
/// 2. Walk directory and collect all valid files (sorted)
/// 3. Compute SHA-256 for each file
/// 4. Generate manifest.toml
/// 5. Create tar archive with all files + manifest
/// 6. Compress with zstd
pub fn build_pack<P: AsRef<Path>, Q: AsRef<Path>>(input_dir: P, output_file: Q) -> Result<()> {
    let input_dir = input_dir.as_ref();
    let output_file = output_file.as_ref();

    // Step 1: Load and validate pack.toml
    let pack_toml_path = input_dir.join("pack.toml");
    if !pack_toml_path.exists() {
        anyhow::bail!("pack.toml not found in {}", input_dir.display());
    }

    let pack_toml_content = std::fs::read_to_string(&pack_toml_path)
        .with_context(|| format!("Failed to read pack.toml at {}", pack_toml_path.display()))?;

    let pack_meta: PackMeta = toml::from_str(&pack_toml_content)
        .with_context(|| format!("Failed to parse pack.toml at {}", pack_toml_path.display()))?;

    // Step 2: Collect all files, excluding unwanted ones
    let mut file_paths = collect_files(input_dir)?;

    // Sort for deterministic builds
    file_paths.sort();

    // Step 3: Generate file entries with hashes
    let mut file_entries = Vec::new();
    for file_path in &file_paths {
        let full_path = input_dir.join(file_path);
        let metadata = std::fs::metadata(&full_path)
            .with_context(|| format!("Failed to read metadata for {}", full_path.display()))?;

        let size = metadata.len();
        let hash = compute_sha256(&full_path)
            .with_context(|| format!("Failed to compute hash for {}", full_path.display()))?;

        // Normalize path to use forward slashes
        let normalized_path = file_path.to_string_lossy().replace('\\', "/");

        file_entries.push(FileEntry {
            path: normalized_path,
            size,
            sha256: hash,
        });
    }

    // Step 4: Create manifest
    let manifest = Manifest {
        pack: pack_meta.clone(),
        files: file_entries,
    };

    let manifest_toml = toml::to_string_pretty(&manifest)
        .context("Failed to serialize manifest to TOML")?;

    // Step 5: Create tar archive
    let tar_data = create_tar_archive(input_dir, &file_paths, &manifest_toml)
        .context("Failed to create tar archive")?;

    // Step 6: Compress with zstd
    let compressed = zstd::encode_all(&tar_data[..], 3)
        .context("Failed to compress archive with zstd")?;

    // Write to output file
    std::fs::write(output_file, compressed)
        .with_context(|| format!("Failed to write output file {}", output_file.display()))?;

    println!("✓ Pack built successfully: {}", output_file.display());
    println!("  Pack ID: {}", pack_meta.pack_id);
    println!("  Version: {}", pack_meta.version);
    println!("  Files: {}", file_paths.len());

    Ok(())
}

/// Collect all files from the input directory, excluding unwanted files
fn collect_files(input_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(input_dir)
        .into_iter()
        .filter_entry(|e| !is_excluded(e))
    {
        let entry = entry.context("Failed to read directory entry")?;

        // Skip directories
        if entry.file_type().is_dir() {
            continue;
        }

        // Get relative path
        let relative_path = entry
            .path()
            .strip_prefix(input_dir)
            .context("Failed to compute relative path")?
            .to_path_buf();

        files.push(relative_path);
    }

    Ok(files)
}

/// Check if a directory entry should be excluded
fn is_excluded(entry: &walkdir::DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();

    // Exclude hidden files and directories
    if name.starts_with('.') {
        return true;
    }

    // Exclude common editor/build artifacts
    let excluded_names = [
        "node_modules",
        "target",
        "dist",
        "build",
        "__pycache__",
        ".DS_Store",
        "Thumbs.db",
    ];

    if excluded_names.contains(&name.as_ref()) {
        return true;
    }

    // Exclude backup files
    if name.ends_with('~') || name.ends_with(".bak") || name.ends_with(".swp") {
        return true;
    }

    false
}

/// Compute SHA-256 hash of a file
fn compute_sha256(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Create a tar archive containing all files plus the generated manifest
fn create_tar_archive(
    input_dir: &Path,
    file_paths: &[PathBuf],
    manifest_toml: &str,
) -> Result<Vec<u8>> {
    let mut tar_data = Vec::new();
    {
        let mut tar = tar::Builder::new(&mut tar_data);

        // Add all collected files
        for file_path in file_paths {
            let full_path = input_dir.join(file_path);
            let mut file = File::open(&full_path)
                .with_context(|| format!("Failed to open file {}", full_path.display()))?;

            // Normalize path to use forward slashes
            let normalized_path = file_path.to_string_lossy().replace('\\', "/");

            tar.append_file(&normalized_path, &mut file)
                .with_context(|| format!("Failed to add {} to archive", normalized_path))?;
        }

        // Add manifest.toml
        let manifest_bytes = manifest_toml.as_bytes();
        let mut header = tar::Header::new_gnu();
        header.set_path("manifest.toml")?;
        header.set_size(manifest_bytes.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();

        tar.append(&header, manifest_bytes)
            .context("Failed to add manifest.toml to archive")?;

        tar.finish().context("Failed to finalize tar archive")?;
    }

    Ok(tar_data)
}
