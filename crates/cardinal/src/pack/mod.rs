//! Cardinal Codex Pack System
//!
//! This module provides functionality for creating, loading, and managing
//! .ccpack files - single-file distributable bundles of card definitions
//! and Rhai scripts.
//!
//! # Overview
//!
//! A `.ccpack` file is a deterministic, compressed archive containing:
//! - `pack.toml`: Pack metadata (ID, version, dependencies)
//! - `cards/*.toml`: Card definition files
//! - `scripts/*.rhai`: Rhai script files
//! - `manifest.toml`: Auto-generated file list with hashes
//!
//! # Format
//!
//! The pack format is:
//! - Deterministic: Same input → same output
//! - Portable: Single file, no extraction needed
//! - Inspectable: Can list contents without extraction
//! - Verifiable: Includes SHA-256 hashes for all files
//!
//! # Example Usage
//!
//! ```no_run
//! use cardinal::pack::{build_pack, load_pack, list_pack};
//!
//! // Build a pack from a directory
//! build_pack("./my-pack", "./output/my-pack.ccpack").unwrap();
//!
//! // List pack contents
//! list_pack("./output/my-pack.ccpack").unwrap();
//!
//! // Load pack into memory
//! let (manifest, files) = load_pack("./output/my-pack.ccpack").unwrap();
//! ```

pub mod metadata;
pub mod builder;
pub mod loader;

// Re-export main API
pub use metadata::{PackMeta, FileEntry, Manifest};
pub use builder::build_pack;
pub use loader::{load_pack, list_pack, unpack_pack};
