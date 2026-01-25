# Cardinal Codex Pack System

The Cardinal Codex pack system provides a single-file format (`.ccpack`) for distributing card definitions and Rhai scripts. This format is deterministic, portable, and can be loaded directly into memory without extraction.

## Overview

A `.ccpack` file is a compressed archive containing:
- `pack.toml` - Pack metadata (required)
- `cards/*.toml` - Card definition files
- `scripts/*.rhai` - Rhai script files
- `manifest.toml` - Auto-generated file list with SHA-256 hashes (added during build)

## Pack Format

The pack format uses:
- **TAR** for archiving files with relative paths
- **ZSTD** for compression (level 3)
- **SHA-256** hashing for file integrity verification
- **Deterministic builds** - same input produces identical output

## Creating a Pack

### Directory Structure

```
my-pack/
├── pack.toml              # Required: pack metadata
├── cards/                 # Card definitions
│   ├── fireball.toml
│   └── lightning.toml
└── scripts/               # Rhai scripts
    ├── fireball.rhai
    └── lightning.rhai
```

### pack.toml Format

```toml
pack_id = "my-pack"         # Required: unique identifier
version = "1.0.0"           # Required: semantic version
name = "My Card Pack"       # Optional: human-readable name
description = "..."         # Optional: pack description
dependencies = []           # Optional: list of pack IDs this depends on
```

### Building a Pack

Using the CLI:

```bash
cardinal-cli build-pack ./my-pack ./output/my-pack.ccpack
```

Using the Rust API:

```rust
use cardinal::pack::build_pack;

build_pack("./my-pack", "./output/my-pack.ccpack")?;
```

## Loading a Pack

### In-Memory Loading

```rust
use cardinal::pack::load_pack;

// Load pack into memory (no disk extraction)
let (manifest, files) = load_pack("./my-pack.ccpack")?;

// Access files
if let Some(card_data) = files.get("cards/fireball.toml") {
    let card_toml = String::from_utf8(card_data.clone())?;
    // Parse and use...
}
```

### Inspecting a Pack

Using the CLI:

```bash
cardinal-cli list-pack ./my-pack.ccpack
```

Output:
```
Pack: my-pack
Version: 1.0.0
Name: My Card Pack
Description: ...

Files (5):
  cards/fireball.toml (118 bytes, sha256: 71757bc8...)
  cards/lightning.toml (124 bytes, sha256: 793e8c08...)
  pack.toml (142 bytes, sha256: 32fb5626...)
  scripts/fireball.rhai (122 bytes, sha256: 1b4b914b...)
  scripts/lightning.rhai (123 bytes, sha256: 3ec9c14f...)
```

### Unpacking a Pack

For debugging or inspection:

```bash
cardinal-cli unpack-pack ./my-pack.ccpack ./output-dir
```

## File Filtering

The pack builder automatically excludes:
- Hidden files and directories (starting with `.`)
- Version control (`.git/`, `.svn/`, etc.)
- Editor artifacts (`.swp`, `.bak`, `~` suffix)
- Build directories (`node_modules/`, `target/`, `dist/`, `build/`)
- System files (`.DS_Store`, `Thumbs.db`)
- Python cache (`__pycache__/`)

## Deterministic Builds

The pack system ensures deterministic builds by:
1. Sorting all file paths before archiving
2. Using normalized paths (forward slashes)
3. Using consistent tar header metadata
4. Excluding timestamp-dependent data

You can verify this:

```bash
# Build the same pack twice
cardinal-cli build-pack ./my-pack ./pack1.ccpack
cardinal-cli build-pack ./my-pack ./pack2.ccpack

# Hashes should match
sha256sum ./pack1.ccpack ./pack2.ccpack
```

## Manifest Format

The `manifest.toml` file is auto-generated and includes:

```toml
[pack]
pack_id = "my-pack"
version = "1.0.0"
# ... other metadata

[[files]]
path = "cards/fireball.toml"
size = 118
sha256 = "71757bc87f0b4870861a0a3275dfb39c1cf41876ea8d83bd246f82348c5d936b"

[[files]]
path = "cards/lightning.toml"
size = 124
sha256 = "793e8c08e2ed61d293cad999ce59d6c7da4de28c9b587e59b636d0205f18596a"
```

## Error Handling

All pack operations use `anyhow::Result` for detailed error messages:

```rust
use cardinal::pack::build_pack;

match build_pack("./my-pack", "./output.ccpack") {
    Ok(_) => println!("Pack built successfully"),
    Err(e) => {
        eprintln!("Error building pack: {}", e);
        // Error includes context about which file/operation failed
    }
}
```

## CLI Commands

### build-pack

Build a `.ccpack` file from a directory.

```bash
cardinal-cli build-pack <INPUT_DIR> <OUTPUT_FILE>
```

### list-pack

List contents of a `.ccpack` file.

```bash
cardinal-cli list-pack <PACK_FILE>
```

### unpack-pack

Extract a `.ccpack` file to a directory.

```bash
cardinal-cli unpack-pack <PACK_FILE> <OUTPUT_DIR>
```

## Dependencies

The pack system uses:
- `walkdir` - Directory traversal
- `tar` - Archive creation/extraction
- `zstd` - Compression/decompression
- `sha2` - File hashing
- `serde` + `toml` - Metadata serialization
- `anyhow` - Error handling with context

## Testing

Run the pack system tests:

```bash
cargo test pack
```

Example test pack creation:

```rust
use cardinal::pack::{build_pack, load_pack};

// Create test pack
build_pack("./test-pack", "./test.ccpack")?;

// Load and verify
let (manifest, files) = load_pack("./test.ccpack")?;
assert_eq!(manifest.pack.pack_id, "test-pack");
assert!(files.contains_key("pack.toml"));
```
