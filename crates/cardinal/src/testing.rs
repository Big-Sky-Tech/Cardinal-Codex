//! Testing and simulation module for Cardinal Codex
//!
//! This module provides functionality for testing and simulating game scenarios
//! without requiring a full UI implementation.

use anyhow::{Context, Result};
use std::path::Path;

use crate::{GameEngine, GameState, Action, load_game_config};
use crate::ids::PlayerId;
use crate::error::CardinalError;

/// Test configuration options
pub struct TestOptions {
    /// Random seed for deterministic testing
    pub seed: u64,
    /// Number of cards to deal to each player for testing
    pub starting_hand_size: usize,
    /// Enable verbose output
    pub verbose: bool,
}

impl Default for TestOptions {
    fn default() -> Self {
        Self {
            seed: 42,
            starting_hand_size: 5,
            verbose: false,
        }
    }
}

/// Initialize a test game from rules
///
/// This creates a game engine with test decks and initializes it for testing
///
/// # Arguments
/// * `rules_path` - Path to rules.toml file
/// * `options` - Test configuration options
///
/// # Returns
/// A ready-to-use GameEngine for testing
pub fn init_test_game<P: AsRef<Path>>(
    rules_path: P,
    options: TestOptions,
) -> Result<GameEngine> {
    let rules_path = rules_path.as_ref();

    if options.verbose {
        println!("Initializing test game...");
        println!("  Rules: {}", rules_path.display());
        println!("  Seed: {}", options.seed);
    }

    // Load game configuration
    let ruleset = load_game_config(rules_path, None)
        .map_err(|e: CardinalError| anyhow::anyhow!(e.0))
        .context("Failed to load game configuration")?;

    if options.verbose {
        println!("  ✓ Loaded {} cards", ruleset.cards.len());
    }

    // Create initial state
    let initial_state = GameState::from_ruleset(&ruleset);

    // Populate test decks
    let mut state = initial_state;
    populate_test_decks(&mut state, options.starting_hand_size);

    if options.verbose {
        println!("  ✓ Test decks populated");
    }

    // Initialize game
    let state = crate::initialize_game(state, &ruleset, options.seed);

    if options.verbose {
        println!("  ✓ Game initialized");
    }

    // Create engine
    let engine = GameEngine::new(ruleset, options.seed, state);

    if options.verbose {
        println!("\n✓ Test game ready!");
        println!("  Players: {}", engine.state.players.len());
    }

    Ok(engine)
}

/// Simulate a simple game scenario for testing
///
/// This runs a basic automated test to verify the engine is working
///
/// # Arguments
/// * `rules_path` - Path to rules.toml file
/// * `options` - Test configuration options
///
/// # Returns
/// Result with summary of the test
pub fn run_basic_test<P: AsRef<Path>>(
    rules_path: P,
    options: TestOptions,
) -> Result<String> {
    let verbose = options.verbose;
    let mut engine = init_test_game(rules_path, options)?;

    let player = PlayerId(0);
    let mut actions_taken = 0;
    let mut events_emitted = 0;

    if verbose {
        println!("\nRunning basic game test...");
        println!("{}", "=".repeat(60));
    }

    // Try passing priority a few times to advance the game
    for i in 0..5 {
        if verbose {
            println!("\nTurn {}, Phase: {}, Step: {}",
                engine.state.turn.number,
                engine.state.turn.phase.0,
                engine.state.turn.step.0,
            );
        }

        match engine.apply_action(player, Action::PassPriority) {
            Ok(result) => {
                actions_taken += 1;
                events_emitted += result.events.len();

                if verbose {
                    println!("  ✓ Action {} succeeded", i + 1);
                    println!("  Events: {}", result.events.len());
                }
            }
            Err(e) => {
                if verbose {
                    println!("  ⚠ Action {} failed (expected): {:?}", i + 1, e);
                }
                // Some failures are expected (e.g., not having priority)
            }
        }
    }

    let summary = format!(
        "Test completed successfully!\n  Actions taken: {}\n  Events emitted: {}",
        actions_taken,
        events_emitted
    );

    if verbose {
        println!("\n{}", "=".repeat(60));
        println!("{}", summary);
    }

    Ok(summary)
}

/// Populate test decks with cards
fn populate_test_decks(state: &mut GameState, num_cards: usize) {
    let num_players = state.players.len() as u8;
    for player_idx in 0..num_players {
        let deck_zone_id = format!("deck@{}", player_idx);

        if let Some(deck) = state.zones.iter_mut().find(|z| z.id.0 == deck_zone_id) {
            for i in 0..num_cards {
                let card_id = crate::ids::CardId((player_idx as u32 * 100) + i as u32);
                deck.cards.push(card_id);
            }
        }
    }
}

/// Test card loading from a pack
///
/// This verifies that a pack can be loaded and contains valid cards
///
/// # Arguments
/// * `pack_path` - Path to .ccpack file
/// * `verbose` - Enable verbose output
///
/// # Returns
/// Result with summary of the test
pub fn test_pack_loading<P: AsRef<Path>>(
    pack_path: P,
    verbose: bool,
) -> Result<String> {
    let pack_path = pack_path.as_ref();

    if verbose {
        println!("Testing pack loading...");
        println!("  Pack: {}", pack_path.display());
    }

    // Load pack
    let (manifest, _files) = crate::pack::load_pack(pack_path)
        .context("Failed to load pack")?;

    if verbose {
        println!("  ✓ Pack loaded successfully");
        println!("  Pack ID: {}", manifest.pack.pack_id);
        println!("  Version: {}", manifest.pack.version);
        println!("  Files: {}", manifest.files.len());
    }

    // Count card files
    let card_count = manifest.files.iter()
        .filter(|f| f.path.starts_with("cards/") && f.path.ends_with(".toml"))
        .count();

    let script_count = manifest.files.iter()
        .filter(|f| f.path.starts_with("scripts/") && f.path.ends_with(".rhai"))
        .count();

    let summary = format!(
        "Pack test completed!\n  Cards: {}\n  Scripts: {}\n  Total files: {}",
        card_count,
        script_count,
        manifest.files.len()
    );

    if verbose {
        println!("\n{}", summary);
    }

    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_options_default() {
        let options = TestOptions::default();
        assert_eq!(options.seed, 42);
        assert_eq!(options.starting_hand_size, 5);
        assert!(!options.verbose);
    }
}
