//! Card loading utilities
//!
//! This module provides functions for loading card definitions from various sources:
//! - Individual `.toml` files in a `cards/` directory
//! - `.ccpack` files
//! - Merging cards from multiple sources

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::rules::schema::CardDef;
use crate::pack::load_pack;

/// Load all card definitions from a directory
///
/// Recursively scans the directory for `.toml` files and attempts to parse each as a CardDef.
///
/// # Arguments
/// * `cards_dir` - Path to the directory containing card definition files
///
/// # Returns
/// A vector of CardDef structs
pub fn load_cards_from_dir<P: AsRef<Path>>(cards_dir: P) -> Result<Vec<CardDef>> {
    let cards_dir = cards_dir.as_ref();
    
    if !cards_dir.exists() {
        return Ok(Vec::new());
    }

    let mut cards = Vec::new();

    for entry in WalkDir::new(cards_dir)
        .into_iter()
        .filter_entry(|e| {
            // Skip hidden directories and files
            e.file_name()
                .to_str()
                .map(|s| !s.starts_with('.'))
                .unwrap_or(false)
        })
    {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Warning: Failed to read directory entry: {}", e);
                continue;
            }
        };
        let path = entry.path();
        
        // Only process .toml files
        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("toml") {
            continue;
        }

        // Read and parse the card file
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read card file: {}", path.display()))?;
        
        let card: CardDef = toml::from_str(&content)
            .with_context(|| format!("Failed to parse card file: {}", path.display()))?;
        
        cards.push(card);
    }

    Ok(cards)
}

/// Load card definitions from a .ccpack file
///
/// Extracts all `.toml` files from the `cards/` directory within the pack
/// and parses them as CardDef structs.
///
/// # Arguments
/// * `ccpack_path` - Path to the .ccpack file
///
/// # Returns
/// A vector of CardDef structs
pub fn load_cards_from_pack<P: AsRef<Path>>(ccpack_path: P) -> Result<Vec<CardDef>> {
    let ccpack_path = ccpack_path.as_ref();
    
    let (_manifest, files) = load_pack(ccpack_path)
        .with_context(|| format!("Failed to load pack: {}", ccpack_path.display()))?;

    let mut cards = Vec::new();

    for (path, content) in files {
        // Only process files in the cards/ directory
        if !path.starts_with("cards/") || !path.ends_with(".toml") {
            continue;
        }

        let content_str = String::from_utf8(content)
            .with_context(|| format!("Card file is not valid UTF-8: {}", path))?;
        
        let card: CardDef = toml::from_str(&content_str)
            .with_context(|| format!("Failed to parse card from pack: {}", path))?;
        
        cards.push(card);
    }

    Ok(cards)
}

/// Load cards from multiple sources and merge them
///
/// # Arguments
/// * `sources` - A slice of CardSource enums specifying where to load cards from
///
/// # Returns
/// A vector of all loaded CardDef structs
pub fn load_cards_from_sources(sources: &[CardSource]) -> Result<Vec<CardDef>> {
    let mut all_cards = Vec::new();

    for source in sources {
        let cards = match source {
            CardSource::Directory(path) => load_cards_from_dir(path)?,
            CardSource::Pack(path) => load_cards_from_pack(path)?,
        };
        all_cards.extend(cards);
    }

    Ok(all_cards)
}

/// Enum representing different sources of card definitions
#[derive(Debug, Clone)]
pub enum CardSource {
    /// Load cards from a directory containing .toml files
    Directory(PathBuf),
    /// Load cards from a .ccpack file
    Pack(PathBuf),
}

/// Validate that card IDs are unique
///
/// # Arguments
/// * `cards` - A slice of CardDef structs to validate
///
/// # Returns
/// Ok(()) if all card IDs are unique, Err otherwise
pub fn validate_unique_card_ids(cards: &[CardDef]) -> Result<()> {
    let mut seen_ids = HashMap::new();
    
    for card in cards {
        if let Some(first_occurrence) = seen_ids.insert(&card.id, &card.name) {
            return Err(anyhow::anyhow!(
                "Duplicate card ID '{}' found: '{}' and '{}'",
                card.id,
                first_occurrence,
                card.name
            ));
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_load_cards_from_dir() {
        // Create a temporary test directory
        let temp_dir = std::env::temp_dir().join("test_cards_dir");
        let _ = fs::remove_dir_all(&temp_dir); // Clean up if exists
        fs::create_dir_all(&temp_dir).unwrap();

        // Create a test card file
        let card_toml = r#"
id = "test_card_1"
name = "Test Card"
card_type = "creature"
cost = "1R"
description = "A test card"
"#;
        fs::write(temp_dir.join("test_card.toml"), card_toml).unwrap();

        // Load cards
        let cards = load_cards_from_dir(&temp_dir).unwrap();
        
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].id, "test_card_1");
        assert_eq!(cards[0].name, "Test Card");

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_validate_unique_card_ids() {
        let card1 = CardDef {
            id: "1".to_string(),
            name: "Card One".to_string(),
            card_type: "creature".to_string(),
            cost: None,
            description: None,
            abilities: vec![],
            script_path: None,
            keywords: vec![],
            stats: HashMap::new(),
        };

        let card2 = CardDef {
            id: "2".to_string(),
            name: "Card Two".to_string(),
            card_type: "spell".to_string(),
            cost: None,
            description: None,
            abilities: vec![],
            script_path: None,
            keywords: vec![],
            stats: HashMap::new(),
        };

        // Test valid cards
        assert!(validate_unique_card_ids(&[card1.clone(), card2.clone()]).is_ok());

        // Test duplicate IDs
        let card_duplicate = CardDef {
            id: "1".to_string(),
            name: "Card One Duplicate".to_string(),
            card_type: "creature".to_string(),
            cost: None,
            description: None,
            abilities: vec![],
            script_path: None,
            keywords: vec![],
            stats: HashMap::new(),
        };

        assert!(validate_unique_card_ids(&[card1, card_duplicate]).is_err());
    }
}
