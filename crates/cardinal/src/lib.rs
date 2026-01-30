pub mod error;
pub mod ids;

pub mod model;
pub mod rules;
pub mod state;
pub mod engine;
pub mod util;
pub mod display;
pub mod pack;
pub mod validation;
pub mod compile;
pub mod testing;

pub use engine::core::{GameEngine, StepResult};
pub use engine::init::initialize_game;
pub use error::{EngineError, LegalityError};
pub use model::action::Action;
pub use model::command::Command;
pub use model::event::Event;
pub use rules::schema::Ruleset;
pub use rules::card_loader::CardSource;
pub use rules::RulesModule;
pub use state::gamestate::GameState;
pub use util::rng::GameRng;
pub use display::{GameDisplay, LogEntry};

use std::fs;
use std::path::Path;

use crate::error::CardinalError;
use crate::rules::schema::Ruleset as RulesetToml;
use crate::rules::card_loader::{load_cards_from_sources, validate_unique_card_ids};

/// Load a `Ruleset` from a TOML file. Returns a conservative `CardinalError` on failure.
pub fn load_rules<P: AsRef<Path>>(path: P) -> Result<RulesetToml, CardinalError> {
    let content = fs::read_to_string(path).map_err(|e| CardinalError(format!("Failed to read rules file: {}", e)))?;
    toml::from_str(&content).map_err(|e| CardinalError(format!("Failed to parse TOML: {}", e)))
}

/// Load a complete game configuration with rules and cards from separate sources
///
/// This function loads the rules from a TOML file and then loads cards from the specified sources.
/// The cards are validated for unique IDs and then added to the ruleset.
///
/// # Arguments
/// * `rules_path` - Path to the rules.toml file
/// * `card_sources` - Optional list of card sources (directories or packs). If None, uses default behavior:
///   - If `cards/` directory exists, loads from it (takes priority)
///   - Otherwise, if `cards.toml` file exists, loads from it
///   - Otherwise, no cards are loaded
///
/// # Note
/// When using default behavior (card_sources = None), the `cards/` directory takes priority.
/// If both `cards/` directory and `cards.toml` file exist, only the directory is used.
///
/// # Returns
/// A complete Ruleset with rules and cards loaded
pub fn load_game_config<P: AsRef<Path>>(
    rules_path: P,
    card_sources: Option<Vec<CardSource>>,
) -> Result<RulesetToml, CardinalError> {
    let rules_path = rules_path.as_ref();
    
    // Load base rules
    let mut ruleset = load_rules(rules_path)?;
    
    // Determine card sources
    let sources = if let Some(sources) = card_sources {
        sources
    } else {
        // Default: look for cards/ directory first, then cards.toml file
        // Priority: cards/ directory > cards.toml file
        let rules_dir = rules_path.parent().unwrap_or_else(|| Path::new("."));
        let cards_dir = rules_dir.join("cards");
        let cards_file = rules_dir.join("cards.toml");
        
        if cards_dir.exists() && cards_dir.is_dir() {
            // Priority 1: cards/ directory
            vec![CardSource::Directory(cards_dir)]
        } else if cards_file.exists() && cards_file.is_file() {
            // Priority 2: cards.toml file (only if cards/ directory doesn't exist)
            vec![CardSource::File(cards_file)]
        } else {
            Vec::new()
        }
    };
    
    // Load cards from sources
    let cards = load_cards_from_sources(&sources)
        .map_err(|e| CardinalError(format!("Failed to load cards: {}", e)))?;
    
    // Validate unique card IDs
    validate_unique_card_ids(&cards)
        .map_err(|e| CardinalError(format!("Card validation failed: {}", e)))?;
    
    // Add cards to ruleset
    ruleset.cards = cards;
    
    Ok(ruleset)
}
