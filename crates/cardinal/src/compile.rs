//! Compilation module for Cardinal Codex
//!
//! This module provides functionality for compiling game assets into
//! optimized, ready-to-use artifacts for frontends.

use anyhow::{Context, Result};
use std::path::Path;

use crate::rules::schema::Ruleset;
use crate::rules::card_loader::CardSource;
use crate::pack::build_pack;
use crate::validation::{validate_rules, validate_pack};
use crate::error::CardinalError;

/// Compilation options
pub struct CompileOptions {
    /// Validate all inputs before compiling
    pub validate: bool,
    /// Output verbose compilation information
    pub verbose: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            validate: true,
            verbose: false,
        }
    }
}

/// Compile a game from rules and card sources into a .ccpack artifact
///
/// This is a high-level function that:
/// 1. Validates rules and cards (if validate option is enabled)
/// 2. Loads and merges all game configuration
/// 3. Creates a distributable .ccpack file
///
/// # Arguments
/// * `rules_path` - Path to rules.toml file
/// * `card_sources` - Optional list of card sources (directories or packs)
/// * `output_path` - Path where the compiled .ccpack will be written
/// * `options` - Compilation options
///
/// # Returns
/// Result with the compiled ruleset
pub fn compile_game<P: AsRef<Path>, Q: AsRef<Path>>(
    rules_path: P,
    card_sources: Option<Vec<CardSource>>,
    output_path: Q,
    options: CompileOptions,
) -> Result<Ruleset> {
    let rules_path = rules_path.as_ref();
    let output_path = output_path.as_ref();

    if options.verbose {
        println!("Compiling game artifact...");
        println!("  Rules: {}", rules_path.display());
    }

    // Step 1: Validate if requested
    if options.validate {
        if options.verbose {
            println!("\nValidating rules...");
        }

        let validation_result = validate_rules(rules_path)
            .context("Failed to validate rules")?;

        if !validation_result.is_valid {
            anyhow::bail!(
                "Rules validation failed with {} error(s)",
                validation_result.errors.len()
            );
        }

        if options.verbose && !validation_result.warnings.is_empty() {
            println!("  ⚠️  {} warning(s) found", validation_result.warnings.len());
        }
    }

    // Step 2: Load complete game configuration
    if options.verbose {
        println!("\nLoading game configuration...");
    }

    let ruleset = crate::load_game_config(rules_path, card_sources)
        .map_err(|e: CardinalError| anyhow::anyhow!(e.0))
        .context("Failed to load game configuration")?;

    if options.verbose {
        println!("  ✓ Loaded {} cards", ruleset.cards.len());
        println!("  ✓ Loaded {} zones", ruleset.zones.len());
        println!("  ✓ Loaded {} phases", ruleset.turn.phases.len());
    }

    if options.verbose {
        println!("\n✓ Compilation successful!");
        println!("  Game: {} v{}", ruleset.game.name, ruleset.game.version);
        println!("  Cards: {}", ruleset.cards.len());
    }

    Ok(ruleset)
}

/// Compile a pack directory into a .ccpack file with validation
///
/// This is a wrapper around build_pack that adds validation
///
/// # Arguments
/// * `pack_dir` - Path to pack directory containing pack.toml
/// * `output_path` - Path where the .ccpack will be written
/// * `options` - Compilation options
pub fn compile_pack<P: AsRef<Path>, Q: AsRef<Path>>(
    pack_dir: P,
    output_path: Q,
    options: CompileOptions,
) -> Result<()> {
    let pack_dir = pack_dir.as_ref();
    let output_path = output_path.as_ref();

    if options.verbose {
        println!("Compiling pack...");
        println!("  Input: {}", pack_dir.display());
        println!("  Output: {}", output_path.display());
    }

    // Validate pack if requested
    if options.validate {
        if options.verbose {
            println!("\nValidating pack...");
        }

        let validation_result = validate_pack(pack_dir)
            .context("Failed to validate pack")?;

        if !validation_result.is_valid {
            anyhow::bail!(
                "Pack validation failed with {} error(s)",
                validation_result.errors.len()
            );
        }

        if options.verbose {
            println!("  ✓ Validation passed");
            if !validation_result.warnings.is_empty() {
                println!("  ⚠️  {} warning(s) found", validation_result.warnings.len());
            }
        }
    }

    // Build the pack
    if options.verbose {
        println!("\nBuilding pack...");
    }

    build_pack(pack_dir, output_path)
        .context("Failed to build pack")?;

    // Note: build_pack already prints success message

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_options_default() {
        let options = CompileOptions::default();
        assert!(options.validate);
        assert!(!options.verbose);
    }
}
