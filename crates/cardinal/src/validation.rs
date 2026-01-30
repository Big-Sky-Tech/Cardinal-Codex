//! Validation module for Cardinal Codex
//!
//! This module provides validation functionality for:
//! - Rules TOML files
//! - Card TOML files
//! - Rhai scripts
//! - Pack directories
//!
//! All validation functions return detailed error messages to help users
//! identify and fix issues.

use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::Path;

use crate::rules::schema::{Ruleset, CardDef};
use crate::rules::card_loader::{load_cards_from_dir, load_cards_from_file, validate_unique_card_ids};
use crate::pack::metadata::PackMeta;

/// Validation result with detailed diagnostics
#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.is_valid = false;
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        if !other.is_valid {
            self.is_valid = false;
        }
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate a rules.toml file
pub fn validate_rules<P: AsRef<Path>>(path: P) -> Result<ValidationResult> {
    let path = path.as_ref();
    let mut result = ValidationResult::new();

    // Check file exists
    if !path.exists() {
        result.add_error(format!("Rules file not found: {}", path.display()));
        return Ok(result);
    }

    // Try to load and parse
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read rules file: {}", path.display()))?;

    let ruleset: Ruleset = match toml::from_str(&content) {
        Ok(r) => r,
        Err(e) => {
            result.add_error(format!("Failed to parse rules TOML: {}", e));
            return Ok(result);
        }
    };

    // Validate game metadata
    if ruleset.game.name.is_empty() {
        result.add_error("Game name cannot be empty".to_string());
    }

    // Validate phases
    if ruleset.turn.phases.is_empty() {
        result.add_error("At least one phase must be defined".to_string());
    } else {
        let mut phase_ids = HashSet::new();
        for phase in &ruleset.turn.phases {
            if phase.id.is_empty() {
                result.add_error("Phase ID cannot be empty".to_string());
            }
            if !phase_ids.insert(&phase.id) {
                result.add_error(format!("Duplicate phase ID: {}", phase.id));
            }
            if phase.steps.is_empty() {
                result.add_warning(format!("Phase '{}' has no steps", phase.id));
            }
        }
    }

    // Validate zones
    if ruleset.zones.is_empty() {
        result.add_error("At least one zone must be defined".to_string());
    } else {
        let mut zone_ids = HashSet::new();
        for zone in &ruleset.zones {
            if zone.id.is_empty() {
                result.add_error("Zone ID cannot be empty".to_string());
            }
            if !zone_ids.insert(&zone.id) {
                result.add_error(format!("Duplicate zone ID: {}", zone.id));
            }
        }
    }

    // Validate starting life
    if ruleset.players.starting_life == 0 {
        result.add_warning("Starting life is 0".to_string());
    }

    Ok(result)
}

/// Validate a card TOML file
pub fn validate_card<P: AsRef<Path>>(path: P) -> Result<ValidationResult> {
    let path = path.as_ref();
    let mut result = ValidationResult::new();

    // Check file exists
    if !path.exists() {
        result.add_error(format!("Card file not found: {}", path.display()));
        return Ok(result);
    }

    // Try to load and parse
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read card file: {}", path.display()))?;

    let card: CardDef = match toml::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            result.add_error(format!("Failed to parse card TOML: {}", e));
            return Ok(result);
        }
    };

    // Validate card fields
    if card.id.is_empty() {
        result.add_error("Card ID cannot be empty".to_string());
    }

    if card.name.is_empty() {
        result.add_error("Card name cannot be empty".to_string());
    }

    if card.card_type.is_empty() {
        result.add_error("Card type cannot be empty".to_string());
    }

    // Note: Card types are config-driven in Cardinal, so we don't validate against
    // a hardcoded list. The game designer defines what card types are valid.

    // Check for script file if script_path is specified
    if let Some(script_path) = &card.script_path {
        if !script_path.is_empty() {
            // Check if script file exists relative to card file
            if let Some(card_dir) = path.parent() {
                let full_script_path = card_dir.join(script_path);
                if !full_script_path.exists() {
                    // Also try relative to card_dir parent (for cards/ subdirectory)
                    let alt_script_path = card_dir.parent()
                        .map(|p| p.join(script_path))
                        .unwrap_or_else(|| full_script_path.clone());
                    
                    if !alt_script_path.exists() {
                        result.add_warning(format!(
                            "Script file not found: {} (checked {} and {})",
                            script_path,
                            full_script_path.display(),
                            alt_script_path.display()
                        ));
                    }
                }
            }
        }
    }

    Ok(result)
}

/// Validate a cards directory
pub fn validate_cards_dir<P: AsRef<Path>>(path: P) -> Result<ValidationResult> {
    let path = path.as_ref();
    let mut result = ValidationResult::new();

    // Check directory exists
    if !path.exists() {
        result.add_error(format!("Cards directory not found: {}", path.display()));
        return Ok(result);
    }

    if !path.is_dir() {
        result.add_error(format!("Path is not a directory: {}", path.display()));
        return Ok(result);
    }

    // Load cards from directory
    let cards = match load_cards_from_dir(path) {
        Ok(c) => c,
        Err(e) => {
            result.add_error(format!("Failed to load cards: {}", e));
            return Ok(result);
        }
    };

    if cards.is_empty() {
        result.add_warning("No card files found in directory".to_string());
        return Ok(result);
    }

    // Validate unique IDs
    if let Err(e) = validate_unique_card_ids(&cards) {
        result.add_error(format!("Card ID validation failed: {}", e));
    }

    // Validate each card
    for card in &cards {
        if card.id.is_empty() {
            result.add_error(format!("Card '{}' has empty ID", card.name));
        }
        if card.name.is_empty() {
            result.add_error(format!("Card with ID '{}' has empty name", card.id));
        }
    }

    Ok(result)
}

/// Validate a cards.toml file
pub fn validate_cards_file<P: AsRef<Path>>(path: P) -> Result<ValidationResult> {
    let path = path.as_ref();
    let mut result = ValidationResult::new();

    // Check file exists
    if !path.exists() {
        result.add_error(format!("Cards file not found: {}", path.display()));
        return Ok(result);
    }

    // Load cards from file
    let cards = match load_cards_from_file(path) {
        Ok(c) => c,
        Err(e) => {
            result.add_error(format!("Failed to load cards: {}", e));
            return Ok(result);
        }
    };

    if cards.is_empty() {
        result.add_warning("No cards defined in file".to_string());
        return Ok(result);
    }

    // Validate unique IDs
    if let Err(e) = validate_unique_card_ids(&cards) {
        result.add_error(format!("Card ID validation failed: {}", e));
    }

    Ok(result)
}

/// Validate a Rhai script file
pub fn validate_script<P: AsRef<Path>>(path: P) -> Result<ValidationResult> {
    let path = path.as_ref();
    let mut result = ValidationResult::new();

    // Check file exists
    if !path.exists() {
        result.add_error(format!("Script file not found: {}", path.display()));
        return Ok(result);
    }

    // Read script content
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read script file: {}", path.display()))?;

    // Try to compile the script
    let engine = rhai::Engine::new();
    if let Err(e) = engine.compile(&content) {
        result.add_error(format!("Script compilation failed: {}", e));
    }

    // Check for empty script
    if content.trim().is_empty() {
        result.add_warning("Script file is empty".to_string());
    }

    Ok(result)
}

/// Validate a pack directory before building
pub fn validate_pack<P: AsRef<Path>>(path: P) -> Result<ValidationResult> {
    let path = path.as_ref();
    let mut result = ValidationResult::new();

    // Check directory exists
    if !path.exists() {
        result.add_error(format!("Pack directory not found: {}", path.display()));
        return Ok(result);
    }

    if !path.is_dir() {
        result.add_error(format!("Path is not a directory: {}", path.display()));
        return Ok(result);
    }

    // Check pack.toml exists
    let pack_toml_path = path.join("pack.toml");
    if !pack_toml_path.exists() {
        result.add_error("pack.toml not found in pack directory".to_string());
        return Ok(result);
    }

    // Validate pack.toml
    let pack_content = std::fs::read_to_string(&pack_toml_path)
        .with_context(|| format!("Failed to read pack.toml: {}", pack_toml_path.display()))?;

    let pack_meta: PackMeta = match toml::from_str(&pack_content) {
        Ok(p) => p,
        Err(e) => {
            result.add_error(format!("Failed to parse pack.toml: {}", e));
            return Ok(result);
        }
    };

    // Validate pack metadata
    if pack_meta.pack_id.is_empty() {
        result.add_error("pack_id cannot be empty".to_string());
    }

    if pack_meta.version.is_empty() {
        result.add_error("version cannot be empty".to_string());
    }

    // Check for cards directory
    let cards_dir = path.join("cards");
    if cards_dir.exists() && cards_dir.is_dir() {
        let cards_result = validate_cards_dir(&cards_dir)?;
        result.merge(cards_result);
    } else {
        result.add_warning("No cards/ directory found in pack".to_string());
    }

    // Check for scripts directory
    let scripts_dir = path.join("scripts");
    if scripts_dir.exists() && scripts_dir.is_dir() {
        // Validate all .rhai files in scripts/
        let script_files = std::fs::read_dir(&scripts_dir)
            .with_context(|| format!("Failed to read scripts directory: {}", scripts_dir.display()))?;

        for entry in script_files {
            let entry = entry?;
            let script_path = entry.path();
            
            if script_path.extension().and_then(|s| s.to_str()) == Some("rhai") {
                let script_result = validate_script(&script_path)?;
                if !script_result.is_valid {
                    let filename = script_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("<unknown>");
                    
                    result.add_error(format!(
                        "Script validation failed for {}: {}",
                        filename,
                        script_result.errors.join(", ")
                    ));
                }
            }
        }
    }

    Ok(result)
}

/// Print validation result to stdout
pub fn print_validation_result(result: &ValidationResult, context: &str) {
    if result.is_valid && result.errors.is_empty() && result.warnings.is_empty() {
        println!("✓ {} validation passed", context);
        return;
    }

    println!("\n{} Validation Report", context);
    println!("{}", "=".repeat(60));

    if !result.errors.is_empty() {
        println!("\n❌ Errors ({}):", result.errors.len());
        for (i, error) in result.errors.iter().enumerate() {
            println!("  {}. {}", i + 1, error);
        }
    }

    if !result.warnings.is_empty() {
        println!("\n⚠️  Warnings ({}):", result.warnings.len());
        for (i, warning) in result.warnings.iter().enumerate() {
            println!("  {}. {}", i + 1, warning);
        }
    }

    println!("\n{}", "=".repeat(60));
    if result.is_valid {
        println!("✓ Validation passed with {} warning(s)", result.warnings.len());
    } else {
        println!("❌ Validation failed with {} error(s)", result.errors.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_validate_rules_missing_file() {
        let result = validate_rules("/nonexistent/path/rules.toml").unwrap();
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validate_card_missing_file() {
        let result = validate_card("/nonexistent/path/card.toml").unwrap();
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validate_script_syntax() {
        // Create a temporary test script with syntax error
        let temp_dir = std::env::temp_dir().join("test_script_validation");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let bad_script = "let x = ";  // Incomplete statement
        let script_path = temp_dir.join("bad.rhai");
        fs::write(&script_path, bad_script).unwrap();

        let result = validate_script(&script_path).unwrap();
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_validate_empty_script() {
        // Create a temporary empty script
        let temp_dir = std::env::temp_dir().join("test_empty_script");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let script_path = temp_dir.join("empty.rhai");
        fs::write(&script_path, "").unwrap();

        let result = validate_script(&script_path).unwrap();
        assert!(result.is_valid);  // Empty is valid, just a warning
        assert!(!result.warnings.is_empty());

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
