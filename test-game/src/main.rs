use cardinal::*;
use std::io;

fn main() {
    println!("====================================");
    println!("Welcome to Cardinal Test Game!");
    println!("====================================\n");

    // Load game configuration
    let rules_path = "test-game/rules.toml";
    println!("Loading game rules from: {}", rules_path);
    
    let rules = match cardinal::load_game_config(rules_path, None) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("ERROR: Failed to load game config: {:?}", e);
            eprintln!("\nMake sure you're running from the repository root:");
            eprintln!("  cargo run --bin test-game");
            return;
        }
    };

    println!("✓ Rules loaded: {}", rules.game.name);
    println!("✓ Cards loaded: {}\n", rules.cards.len());

    // Create initial game state
    let initial_state = GameState::from_ruleset(&rules);
    
    // Create test decks with some cards
    let mut state = initial_state.clone();
    populate_test_decks(&mut state, 3);
    println!("✓ Test decks populated with 3 cards each\n");

    // Initialize the game with a fixed seed for consistency
    let seed = 42;
    let state = cardinal::initialize_game(state, &rules, seed);
    
    // Create game engine
    let engine = GameEngine::new(rules, seed, state);
    
    println!("====================================");
    println!("Game Ready!");
    println!("====================================\n");
    
    // Display game state
    display_game_state(&engine);
    
    println!("\nThis is a minimal test game demonstrating Cardinal engine integration.");
    println!("The game has been initialized with two players and starting hands.\n");
    
    println!("Next steps to make this interactive:");
    println!("  1. Add a game loop to handle player actions");
    println!("  2. Implement action handlers (play card, pass turn, etc.)");
    println!("  3. Add event processing for game events");
    println!("  4. Create a UI rendering system\n");
    
    println!("Press Enter to exit...");
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
}

fn populate_test_decks(state: &mut GameState, num_cards: usize) {
    let num_players = state.players.len() as u8;
    for player_idx in 0..num_players {
        let deck_zone_id = format!("deck@{}", player_idx);

        if let Some(deck) = state.zones.iter_mut().find(|z| z.id.0 == deck_zone_id) {
            for i in 0..num_cards {
                let card_id = cardinal::ids::CardId((player_idx as u32 * 100) + i as u32);
                deck.cards.push(card_id);
            }
        }
    }
}

fn display_game_state(engine: &GameEngine) {
    println!("Current Game State:");
    println!("  Turn: {}", engine.state.turn.number);
    println!("  Phase: {}", engine.state.turn.phase.0);
    println!("  Step: {}", engine.state.turn.step.0);
    println!("  Active Player: Player {}", engine.state.turn.active_player.0);
    println!("  Priority Player: Player {}", engine.state.turn.priority_player.0);
    println!();
    
    for player in &engine.state.players {
        println!("Player {}:", player.id.0);
        println!("  Life: {}", player.life);
        println!("  Mana: {}", player.resources.get("mana").unwrap_or(&0));
        
        // Show hand size
        let hand_zone_id = format!("hand@{}", player.id.0);
        if let Some(hand) = engine.state.zones.iter().find(|z| z.id.0 == hand_zone_id) {
            println!("  Hand: {} cards", hand.cards.len());
        }
        
        // Show deck size
        let deck_zone_id = format!("deck@{}", player.id.0);
        if let Some(deck) = engine.state.zones.iter().find(|z| z.id.0 == deck_zone_id) {
            println!("  Deck: {} cards", deck.cards.len());
        }
        println!();
    }
}
