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
        .follow_links(false)  // Don't follow symlinks to prevent cycles
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

/// Load card definitions from a single TOML file containing a [[cards]] array
///
/// This function loads multiple cards from a single TOML file that uses the [[cards]] array format.
/// This is useful for loading a cards.toml file or similar.
///
/// # Arguments
/// * `file_path` - Path to the TOML file containing card definitions
///
/// # Returns
/// A vector of CardDef structs
///
/// # Example TOML format
/// ```toml
/// [[cards]]
/// id = "1"
/// name = "Goblin Scout"
/// card_type = "creature"
/// cost = "1R"
///
/// [[cards]]
/// id = "2"
/// name = "Fireball"
/// card_type = "spell"
/// cost = "2R"
/// ```
pub fn load_cards_from_file<P: AsRef<Path>>(file_path: P) -> Result<Vec<CardDef>> {
    let file_path = file_path.as_ref();
    
    if !file_path.exists() {
        return Ok(Vec::new());
    }

    #[derive(serde::Deserialize)]
    struct CardsFile {
        cards: Vec<CardDef>,
    }

    let content = std::fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read cards file: {}", file_path.display()))?;
    
    let cards_file: CardsFile = toml::from_str(&content)
        .with_context(|| format!("Failed to parse cards file: {}", file_path.display()))?;
    
    Ok(cards_file.cards)
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
            CardSource::File(path) => load_cards_from_file(path)?,
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
    /// Load cards from a single TOML file containing a [[cards]] array
    File(PathBuf),
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

    #[test]
    fn test_load_cards_from_file() {
        // Create a temporary test file
        let temp_dir = std::env::temp_dir().join("test_cards_file");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let cards_file = temp_dir.join("cards.toml");
        let cards_toml = r#"
[[cards]]
id = "file_card_1"
name = "File Card One"
card_type = "creature"
cost = "1R"
description = "First card from file"

[[cards]]
id = "file_card_2"
name = "File Card Two"
card_type = "spell"
cost = "2U"
description = "Second card from file"

[[cards]]
id = "file_card_3"
name = "File Card Three"
card_type = "enchantment"
cost = "3G"
description = "Third card from file"
"#;
        fs::write(&cards_file, cards_toml).unwrap();

        // Load cards from file
        let cards = load_cards_from_file(&cards_file).unwrap();
        
        assert_eq!(cards.len(), 3);
        assert_eq!(cards[0].id, "file_card_1");
        assert_eq!(cards[0].name, "File Card One");
        assert_eq!(cards[1].id, "file_card_2");
        assert_eq!(cards[1].name, "File Card Two");
        assert_eq!(cards[2].id, "file_card_3");
        assert_eq!(cards[2].name, "File Card Three");

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_load_cards_from_file_nonexistent() {
        // Test loading from a file that doesn't exist
        let nonexistent = PathBuf::from("/tmp/nonexistent_cards_file_12345.toml");
        let cards = load_cards_from_file(&nonexistent).unwrap();
        
        // Should return empty vector for non-existent file
        assert_eq!(cards.len(), 0);
    }

    #[test]
    fn test_load_cards_from_pack() {
        // This test uses the existing example pack
        let pack_path = PathBuf::from("examples/example-pack.ccpack");
        
        // Skip test if pack doesn't exist
        if !pack_path.exists() {
            println!("Skipping test_load_cards_from_pack: example pack not found");
            return;
        }

        let cards = load_cards_from_pack(&pack_path).unwrap();
        
        // The example pack should have cards
        assert!(cards.len() > 0, "Pack should contain at least one card");
        
        // Verify cards have valid structure
        for card in &cards {
            assert!(!card.id.is_empty(), "Card ID should not be empty");
            assert!(!card.name.is_empty(), "Card name should not be empty");
        }
    }

    #[test]
    fn test_load_cards_from_sources_single() {
        // Create a temporary test directory
        let temp_dir = std::env::temp_dir().join("test_sources_single");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create a test card file
        let card_toml = r#"
id = "source_test_1"
name = "Source Test Card"
card_type = "creature"
cost = "1R"
"#;
        fs::write(temp_dir.join("test.toml"), card_toml).unwrap();

        // Load from single source
        let sources = vec![CardSource::Directory(temp_dir.clone())];
        let cards = load_cards_from_sources(&sources).unwrap();
        
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].id, "source_test_1");

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_load_cards_from_sources_multiple() {
        // Create two temporary test directories
        let temp_dir1 = std::env::temp_dir().join("test_sources_multi_1");
        let temp_dir2 = std::env::temp_dir().join("test_sources_multi_2");
        let _ = fs::remove_dir_all(&temp_dir1);
        let _ = fs::remove_dir_all(&temp_dir2);
        fs::create_dir_all(&temp_dir1).unwrap();
        fs::create_dir_all(&temp_dir2).unwrap();

        // Create test cards in first directory
        fs::write(temp_dir1.join("card1.toml"), r#"
id = "multi_1"
name = "Multi Card 1"
card_type = "creature"
"#).unwrap();

        // Create test cards in second directory
        fs::write(temp_dir2.join("card2.toml"), r#"
id = "multi_2"
name = "Multi Card 2"
card_type = "spell"
"#).unwrap();

        // Create a cards.toml file
        let cards_file = std::env::temp_dir().join("test_multi_cards.toml");
        fs::write(&cards_file, r#"
[[cards]]
id = "multi_3"
name = "Multi Card 3"
card_type = "enchantment"
"#).unwrap();

        // Load from multiple sources
        let sources = vec![
            CardSource::Directory(temp_dir1.clone()),
            CardSource::Directory(temp_dir2.clone()),
            CardSource::File(cards_file.clone()),
        ];
        let cards = load_cards_from_sources(&sources).unwrap();
        
        assert_eq!(cards.len(), 3);
        
        // Verify all cards are loaded
        let ids: Vec<_> = cards.iter().map(|c| c.id.as_str()).collect();
        assert!(ids.contains(&"multi_1"));
        assert!(ids.contains(&"multi_2"));
        assert!(ids.contains(&"multi_3"));

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir1);
        let _ = fs::remove_dir_all(&temp_dir2);
        let _ = fs::remove_file(&cards_file);
    }

    #[test]
    fn test_load_cards_from_sources_empty() {
        // Test with empty sources list
        let sources: Vec<CardSource> = vec![];
        let cards = load_cards_from_sources(&sources).unwrap();
        
        assert_eq!(cards.len(), 0);
    }

    #[test]
    fn test_load_cards_from_dir_nested() {
        // Create a temporary test directory with nested structure
        let temp_dir = std::env::temp_dir().join("test_cards_nested");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        fs::create_dir_all(temp_dir.join("subdir")).unwrap();

        // Create cards in root
        fs::write(temp_dir.join("root_card.toml"), r#"
id = "root_1"
name = "Root Card"
card_type = "creature"
"#).unwrap();

        // Create cards in subdirectory
        fs::write(temp_dir.join("subdir/nested_card.toml"), r#"
id = "nested_1"
name = "Nested Card"
card_type = "spell"
"#).unwrap();

        // Load cards (should be recursive)
        let cards = load_cards_from_dir(&temp_dir).unwrap();
        
        assert_eq!(cards.len(), 2);
        
        let ids: Vec<_> = cards.iter().map(|c| c.id.as_str()).collect();
        assert!(ids.contains(&"root_1"));
        assert!(ids.contains(&"nested_1"));

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
