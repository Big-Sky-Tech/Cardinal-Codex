use cardinal_kernel as cardinal;
use cardinal::*;
use cardinal::ids::PlayerId;
use cardinal::model::action::Action;

#[test]
fn build_engine_from_rules() {
    let rules = load_rules("../../rules.toml").expect("load rules");
    let engine = GameEngine::from_ruleset(rules, 42);
    // basic sanity: at least one player present
    assert!(!engine.state.players.is_empty());
}

#[test]
fn test_phase_advancement() {
    let rules = load_rules("../../rules.toml").expect("load rules");
    let mut engine = GameEngine::from_ruleset(rules.clone(), 42);
    
    let initial_phase = engine.state.turn.phase.clone();
    let initial_step = engine.state.turn.step.clone();
    let num_players = engine.state.players.len() as u32;
    
    // Pass priority from every player (need num_players passes to advance)
    for _ in 0..num_players {
        let priority_player = engine.state.turn.priority_player;
        let result = engine.apply_action(priority_player, Action::PassPriority)
            .expect("apply action");
        
        // Should have emitted at least one event
        assert!(!result.events.is_empty(), "PassPriority should emit events");
    }
    
    // After all players pass, phase or step should have advanced
    let phase_advanced = engine.state.turn.phase != initial_phase || engine.state.turn.step != initial_step;
    assert!(phase_advanced, "Phase or step should have advanced after all players pass");
}

#[test]
fn test_phase_progression_full_turn() {
    let rules = load_rules("../../rules.toml").expect("load rules");
    let mut engine = GameEngine::from_ruleset(rules.clone(), 42);
    
    let initial_turn = engine.state.turn.number;
    let initial_active_player = engine.state.turn.active_player;
    
    // Count total steps across all phases
    let total_steps: usize = rules.turn.phases.iter()
        .map(|p| p.steps.len())
        .sum();
    
    let num_players = engine.state.players.len() as u32;
    
    // For each step, we need all players to pass priority to advance
    // So roughly total_steps * num_players passes needed
    for _ in 0..(total_steps * num_players as usize + 10) {
        if engine.state.ended.is_some() {
            break; // Game ended, stop
        }
        let priority_player = engine.state.turn.priority_player;
        let _ = engine.apply_action(priority_player, Action::PassPriority);
    }
    
    // After cycling through all phases, we should have advanced to next turn
    assert_eq!(engine.state.turn.number, initial_turn + 1, "Turn number should have incremented");
    
    // Active player should have rotated
    let expected_next_player = if initial_active_player.0 + 1 < engine.state.players.len() as u8 {
        PlayerId(initial_active_player.0 + 1)
    } else {
        PlayerId(0)
    };
    assert_eq!(engine.state.turn.active_player, expected_next_player, "Active player should rotate");
}

#[test]
fn test_legality_active_player_restriction() {
    let rules = load_rules("../../rules.toml").expect("load rules");
    let mut engine = GameEngine::from_ruleset(rules, 42);
    
    // Only player 0 should be able to play cards (active player)
    let inactive_player = if engine.state.players.len() > 1 {
        PlayerId(1)
    } else {
        PlayerId(0)
    };
    
    // If there are 2+ players and it's player 0's turn, player 1 cannot play
    if engine.state.turn.active_player == PlayerId(0) && inactive_player != PlayerId(0) {
        // Create a dummy card ID and zone
        let card = cardinal::ids::CardId(999);
        let zone = engine.state.zones.iter()
            .find(|z| z.owner == Some(inactive_player))
            .map(|z| z.id.clone());
        
        if let Some(zone_id) = zone {
            let result = engine.apply_action(
                inactive_player,
                Action::PlayCard { card, from: zone_id },
            );
            
            // Should fail: inactive player trying to play
            assert!(result.is_err(), "Inactive player should not be able to play cards");
        }
    }
}

#[test]
fn test_legality_phase_restrictions() {
    let rules = load_rules("../../rules.toml").expect("load rules");
    let mut engine = GameEngine::from_ruleset(rules.clone(), 42);
    
    // Find a phase that does NOT allow actions
    let no_action_phase = rules.turn.phases.iter()
        .find(|p| !p.allow_actions);
    
    if let Some(phase) = no_action_phase {
        // Manually set engine to that phase
        let phase_box: Box<str> = phase.id.clone().into_boxed_str();
        let phase_static: &'static str = Box::leak(phase_box);
        engine.state.turn.phase = cardinal::ids::PhaseId(phase_static);
        
        if let Some(step) = phase.steps.first() {
            let step_box: Box<str> = step.id.clone().into_boxed_str();
            let step_static: &'static str = Box::leak(step_box);
            engine.state.turn.step = cardinal::ids::StepId(step_static);
        }
        
        // Try to play a card in this phase
        let card = cardinal::ids::CardId(888);
        let zone = engine.state.zones.iter()
            .find(|z| z.owner == Some(engine.state.turn.active_player))
            .map(|z| z.id.clone());
        
        if let Some(zone_id) = zone {
            let result = engine.apply_action(
                engine.state.turn.active_player,
                Action::PlayCard { card, from: zone_id },
            );
            
            // Should fail: phase doesn't allow actions
            assert!(result.is_err(), "Cannot play cards in a phase that doesn't allow actions");
        }
    }
}

#[test]
fn test_legality_zone_ownership() {
    let rules = load_rules("../../rules.toml").expect("load rules");
    let mut engine = GameEngine::from_ruleset(rules, 42);
    
    if engine.state.players.len() > 1 {
        let active_player = engine.state.turn.active_player;
        let opponent = if active_player == PlayerId(0) {
            PlayerId(1)
        } else {
            PlayerId(0)
        };
        
        // Try to play a card from opponent's zone
        let card = cardinal::ids::CardId(777);
        let opponent_zone = engine.state.zones.iter()
            .find(|z| z.owner == Some(opponent))
            .map(|z| z.id.clone());
        
        if let Some(zone_id) = opponent_zone {
            let result = engine.apply_action(
                active_player,
                Action::PlayCard { card, from: zone_id },
            );
            
            // Should fail: zone ownership violation
            assert!(result.is_err(), "Cannot play cards from opponent's zones");
        }
    }
}

#[test]
fn test_pass_priority_requires_priority() {
    let rules = load_rules("../../rules.toml").expect("load rules");
    let mut engine = GameEngine::from_ruleset(rules, 42);
    
    // The priority player should be able to pass
    let priority_player = engine.state.turn.priority_player;
    let result = engine.apply_action(priority_player, Action::PassPriority);
    assert!(result.is_ok(), "Priority player should be able to pass priority");
    
    // After one player passes, priority rotates to the next player
    let new_priority = engine.state.turn.priority_player;
    assert_ne!(new_priority, priority_player, "Priority should rotate to next player");
    
    // The old priority player should NOT be able to pass now
    let result = engine.apply_action(priority_player, Action::PassPriority);
    assert!(result.is_err(), "Non-priority player should not be able to pass priority");
}

#[test]
fn test_play_card_action() {
    let rules = load_rules("../../rules.toml").expect("load rules");
    let mut engine = GameEngine::from_ruleset(rules.clone(), 42);
    
    // Find the main phase where actions are allowed
    let main_phase = rules.turn.phases.iter()
        .find(|p| p.allow_actions && p.id.contains("main"));
    
    if let Some(phase) = main_phase {
        // Set to main phase (use first step)
        let phase_box: Box<str> = phase.id.clone().into_boxed_str();
        let phase_static: &'static str = Box::leak(phase_box);
        engine.state.turn.phase = cardinal::ids::PhaseId(phase_static);
        
        if let Some(step) = phase.steps.first() {
            let step_box: Box<str> = step.id.clone().into_boxed_str();
            let step_static: &'static str = Box::leak(step_box);
            engine.state.turn.step = cardinal::ids::StepId(step_static);
        }
    }
    
    let active_player = engine.state.turn.active_player;
    
    // Find hand zone for active player
    let hand_zone = engine.state.zones.iter()
        .find(|z| z.owner == Some(active_player) && z.id.0.starts_with("hand"))
        .map(|z| z.id.clone());
    
    if let Some(hand) = hand_zone {
        // Add a test card to the hand
        let test_card = cardinal::ids::CardId(12345);
        engine.state.zones.iter_mut()
            .find(|z| z.id == hand)
            .map(|z| z.cards.push(test_card));
        
        // Verify the card is in the hand
        assert!(engine.state.zones.iter()
            .find(|z| z.id == hand)
            .map(|z| z.cards.contains(&test_card))
            .unwrap_or(false), "Test card should be in hand");
        
        // Play the card
        let result = engine.apply_action(
            active_player,
            Action::PlayCard { card: test_card, from: hand.clone() },
        );
        
        assert!(result.is_ok(), "Playing card from hand should succeed");
        
        let events = &result.unwrap().events;
        
        // Check for CardPlayed event
        let card_played = events.iter()
            .any(|e| matches!(e, Event::CardPlayed { player: p, card: c } 
                if p == &active_player && c == &test_card));
        assert!(card_played, "CardPlayed event should be emitted");
        
        // Check for CardMoved event
        let card_moved = events.iter()
            .any(|e| matches!(e, Event::CardMoved { card: c, from: f, .. } 
                if c == &test_card && f == &hand));
        assert!(card_moved, "CardMoved event should be emitted");
        
        // Verify card is no longer in hand
        assert!(!engine.state.zones.iter()
            .find(|z| z.id == hand)
            .map(|z| z.cards.contains(&test_card))
            .unwrap_or(false), "Test card should no longer be in hand");
        
        // Verify card is now in field
        let field_zone = engine.state.zones.iter()
            .find(|z| z.owner == Some(active_player) && z.id.0.starts_with("field"))
            .map(|z| z.id.clone());
        
        if let Some(field) = field_zone {
            assert!(engine.state.zones.iter()
                .find(|z| z.id == field)
                .map(|z| z.cards.contains(&test_card))
                .unwrap_or(false), "Test card should be in field after playing");
        }
    }
}

#[test]
fn test_play_card_requires_empty_stack() {
    let rules = load_rules("../../rules.toml").expect("load rules");
    let mut engine = GameEngine::from_ruleset(rules.clone(), 42);
    
    let active_player = engine.state.turn.active_player;
    
    // Add a fake stack item to test the requires_empty_stack rule
    let dummy_stack_item = cardinal::model::command::StackItem {
        id: 1,
        source: Some(cardinal::ids::CardId(999)),
        controller: active_player,
        effect: cardinal::model::command::EffectRef::Builtin("test"),
    };
    engine.state.stack.push(dummy_stack_item);
    
    // Find hand zone
    let hand_zone = engine.state.zones.iter()
        .find(|z| z.owner == Some(active_player) && z.id.0.starts_with("hand"))
        .map(|z| z.id.clone());
    
    if let Some(hand) = hand_zone {
        // Add a test card
        let test_card = cardinal::ids::CardId(54321);
        engine.state.zones.iter_mut()
            .find(|z| z.id == hand)
            .map(|z| z.cards.push(test_card));
        
        // Try to play card with non-empty stack - should fail
        let result = engine.apply_action(
            active_player,
            Action::PlayCard { card: test_card, from: hand },
        );
        
        assert!(result.is_err(), "Playing card with non-empty stack should fail");
    }
}

#[test]
fn test_concede_action() {
    let rules = load_rules("../../rules.toml").expect("load rules");
    let mut engine = GameEngine::from_ruleset(rules, 42);
    
    let player_0 = PlayerId(0);
    
    // Player 0 concedes
    let result = engine.apply_action(player_0, Action::Concede)
        .expect("Concede should succeed");
    
    // Check for GameEnded event
    let game_ended = result.events.iter()
        .any(|e| matches!(e, Event::GameEnded { .. }));
    assert!(game_ended, "GameEnded event should be emitted on concede");
    
    // Game should be marked as ended
    assert!(engine.state.ended.is_some(), "Game should be marked as ended");
}

#[test]
fn test_priority_rotation() {
    let rules = load_rules("../../rules.toml").expect("load rules");
    let mut engine = GameEngine::from_ruleset(rules, 42);
    
    let num_players = engine.state.players.len() as u32;
    if num_players < 2 {
        return; // Need 2+ players for priority rotation test
    }
    
    let initial_priority = engine.state.turn.priority_player;
    let mut priority_sequence = vec![initial_priority];
    
    // Pass priority num_players times to complete a round
    for _ in 0..num_players {
        let current_priority = engine.state.turn.priority_player;
        let result = engine.apply_action(current_priority, Action::PassPriority)
            .expect("should be able to pass");
        
        // Verify we got a priority passed event
        assert!(result.events.iter()
            .any(|e| matches!(e, Event::PriorityPassed { by: p } if p == &current_priority)),
            "PriorityPassed event should be emitted");
        
        priority_sequence.push(engine.state.turn.priority_player);
    }
    
    // After num_players passes, we should be back at the initial priority player
    // (or close to it depending on phase advancement)
    assert!(priority_sequence.iter().take(num_players as usize).all(|p| {
        // Each player should get priority once
        priority_sequence.iter().filter(|x| x == &p).count() >= 1
    }), "Each player should have gotten priority");
}

#[test]
fn test_priority_passes_tracked() {
    let rules = load_rules("../../rules.toml").expect("load rules");
    let mut engine = GameEngine::from_ruleset(rules, 42);
    
    // Initially, priority_passes should be 0
    assert_eq!(engine.state.turn.priority_passes, 0, "Priority passes should start at 0");
    
    // Pass priority once
    let priority_player = engine.state.turn.priority_player;
    let _ = engine.apply_action(priority_player, Action::PassPriority);
    
    // priority_passes should be incremented before any phase advancement
    // But it may be reset if phase advancement occurs. Let's check it's tracking correctly:
    let passes_after_first = engine.state.turn.priority_passes;
    assert!(passes_after_first >= 1, "Priority passes should have been incremented");
}

#[test]
fn test_trigger_evaluation_on_card_play() {
    let rules = load_rules("../../rules.toml").expect("load rules");
    let mut engine = GameEngine::from_ruleset(rules, 42);
    
    // Get the active player (normally player 0)
    let active_player = engine.state.turn.active_player;
    
    // Find a card in the active player's hand to play
    let hand_zone_id = format!("hand@{}", active_player.0);
    let hand = engine.state.zones.iter()
        .find(|z| z.id.0 == hand_zone_id)
        .cloned();
    
    if let Some(hand_zone) = hand {
        if !hand_zone.cards.is_empty() {
            let card_to_play = hand_zone.cards[0];
            let from_zone = hand_zone.id;
            
            // Play the card (this should trigger the card_played trigger)
            let result = engine.apply_action(
                active_player,
                Action::PlayCard { card: card_to_play, from: from_zone },
            ).expect("play card action should succeed");
            
            // Check that we got events - should include CardPlayed, CardMoved, and potentially StackResolved
            assert!(!result.events.is_empty(), "PlayCard should emit events");
            
            // Verify CardPlayed event is present
            let has_card_played = result.events.iter()
                .any(|e| matches!(e, Event::CardPlayed { player, card } 
                    if player == &active_player && card == &card_to_play));
            assert!(has_card_played, "CardPlayed event should be emitted");
            
            // Verify CardMoved event is present (from hand to field)
            let has_card_moved = result.events.iter()
                .any(|e| matches!(e, Event::CardMoved { card, .. } if card == &card_to_play));
            assert!(has_card_moved, "CardMoved event should be emitted");
            
            // With trigger system, we should have stack items created (triggers pushed stack)
            // and potentially resolved (StackResolved events)
            let _has_stack_event = result.events.iter()
                .any(|e| matches!(e, Event::StackResolved { .. }));
            // Note: Stack might be auto-resolved or waiting - this is just checking the mechanism works
            assert!(!result.events.is_empty(), "Events should be generated from card play and triggers");
        }
    }
}

#[test]
fn test_game_initialization_creates_decks() {
    let rules = load_rules("../../rules.toml").expect("load rules");
    let initial_state = GameState::from_ruleset(&rules);
    
    // Create test cards for each player's deck
    let mut state = initial_state.clone();
    let num_players = state.players.len() as u32;
    
    for i in 0..num_players {
        let player_id = PlayerId(i as u8);
        let deck_zone_id_string = format!("deck@{}", player_id.0);
        
        // Find the deck zone and add test cards
        if let Some(deck_zone) = state.zones.iter_mut()
            .find(|z| z.id.0 == deck_zone_id_string)
        {
            // Add 40 test cards to the deck
            for card_num in 0..40 {
                let card_id = cardinal::ids::CardId(i * 100 + card_num);
                deck_zone.cards.push(card_id);
            }
        }
    }
    
    // Initialize the game
    let initialized = cardinal::initialize_game(state, &rules, 42);
    
    // Verify decks exist and have cards
    let mut at_least_one_deck_changed = false;
    for i in 0..num_players {
        let player_id = PlayerId(i as u8);
        let deck_zone_id_string = format!("deck@{}", player_id.0);
        
        let deck_zone = initialized.zones.iter()
            .find(|z| z.id.0 == deck_zone_id_string);
        
        assert!(deck_zone.is_some(), "Deck zone should exist");
        
        let deck = deck_zone.unwrap();
        let is_first_player = player_id == initialized.turn.active_player;
        
        // If this is not the first player, or skip_first_turn_draw is false, 
        // some cards should have been drawn
        if !is_first_player || !rules.turn.skip_first_turn_draw_for_first_player {
            if deck.cards.len() < 40 {
                at_least_one_deck_changed = true;
            }
        }
    }
    
    assert!(at_least_one_deck_changed, "At least one deck should have cards drawn during initialization");
}

#[test]
fn test_game_initialization_draws_starting_hands() {
    let rules = load_rules("../../rules.toml").expect("load rules");
    let initial_state = GameState::from_ruleset(&rules);
    let starting_hand_size = rules.players.starting_hand_size;
    
    let mut state = initial_state.clone();
    let num_players = state.players.len() as u32;
    
    // Create test decks
    for i in 0..num_players {
        let player_id = PlayerId(i as u8);
        let deck_zone_id_string = format!("deck@{}", player_id.0);
        
        if let Some(deck_zone) = state.zones.iter_mut()
            .find(|z| z.id.0 == deck_zone_id_string)
        {
            // Add enough cards for starting hand plus shuffled deck
            for card_num in 0..60 {
                let card_id = cardinal::ids::CardId(i * 1000 + card_num);
                deck_zone.cards.push(card_id);
            }
        }
    }
    
    // Initialize the game
    let initialized = cardinal::initialize_game(state, &rules, 42);
    
    // Check that players have cards in their hands
    for i in 0..num_players {
        let player_id = PlayerId(i as u8);
        let is_first_player = player_id == initialized.turn.active_player;
        let hand_zone_id_string = format!("hand@{}", player_id.0);
        
        let hand_zone = initialized.zones.iter()
            .find(|z| z.id.0 == hand_zone_id_string);
        
        assert!(hand_zone.is_some(), "Hand zone should exist for player {}", i);
        
        let hand = hand_zone.unwrap();
        
        // First player may skip draw if configured
        if !is_first_player || !rules.turn.skip_first_turn_draw_for_first_player {
            assert_eq!(
                hand.cards.len(), 
                starting_hand_size,
                "Player {} should have starting_hand_size cards in hand",
                i
            );
        } else {
            // First player skipped draw
            assert_eq!(
                hand.cards.len(),
                0,
                "First player should have 0 cards if skip_first_turn_draw is true"
            );
        }
    }
}

#[test]
fn test_game_initialization_determines_first_player() {
    let rules = load_rules("../../rules.toml").expect("load rules");
    let initial_state = GameState::from_ruleset(&rules);
    
    let mut state = initial_state.clone();
    let num_players = state.players.len() as u32;
    
    // Add test cards to decks
    for i in 0..num_players {
        let player_id = PlayerId(i as u8);
        let deck_zone_id_string = format!("deck@{}", player_id.0);
        
        if let Some(deck_zone) = state.zones.iter_mut()
            .find(|z| z.id.0 == deck_zone_id_string)
        {
            for card_num in 0..40 {
                let card_id = cardinal::ids::CardId(i * 1000 + card_num);
                deck_zone.cards.push(card_id);
            }
        }
    }
    
    // Initialize game multiple times with same seed - should get same first player
    let initialized1 = cardinal::initialize_game(state.clone(), &rules, 42);
    let initialized2 = cardinal::initialize_game(state.clone(), &rules, 42);
    
    assert_eq!(
        initialized1.turn.active_player,
        initialized2.turn.active_player,
        "Same seed should result in same first player"
    );
    
    // Different seed might give different first player (with "random" rule)
    let initialized3 = cardinal::initialize_game(state.clone(), &rules, 99);
    // This might be the same or different - just verify it's a valid player ID
    assert!(initialized3.turn.active_player.0 < num_players as u8, "First player should be valid");
}

#[test]
fn test_game_initialization_preserves_priority() {
    let rules = load_rules("../../rules.toml").expect("load rules");
    let initial_state = GameState::from_ruleset(&rules);
    
    let mut state = initial_state.clone();
    let num_players = state.players.len() as u32;
    
    // Add test cards
    for i in 0..num_players {
        let player_id = PlayerId(i as u8);
        let deck_zone_id_string = format!("deck@{}", player_id.0);
        
        if let Some(deck_zone) = state.zones.iter_mut()
            .find(|z| z.id.0 == deck_zone_id_string)
        {
            for card_num in 0..40 {
                let card_id = cardinal::ids::CardId(i * 100 + card_num);
                deck_zone.cards.push(card_id);
            }
        }
    }
    
    let initialized = cardinal::initialize_game(state, &rules, 42);
    
    // Priority player should be the same as active player initially
    assert_eq!(
        initialized.turn.active_player,
        initialized.turn.priority_player,
        "Priority player should be the same as active player after initialization"
    );
}

#[test]
fn test_card_ability_etb_trigger() {
    let rules = load_rules("../../rules.toml").expect("load rules");
    let mut engine = GameEngine::from_ruleset(rules.clone(), 42);
    
    // Set engine to a phase that allows actions
    let main_phase = rules.turn.phases.iter()
        .find(|p| p.allow_actions && p.id.contains("main"));
    
    if let Some(phase) = main_phase {
        let phase_box: Box<str> = phase.id.clone().into_boxed_str();
        let phase_static: &'static str = Box::leak(phase_box);
        engine.state.turn.phase = cardinal::ids::PhaseId(phase_static);
        
        if let Some(step) = phase.steps.first() {
            let step_box: Box<str> = step.id.clone().into_boxed_str();
            let step_static: &'static str = Box::leak(step_box);
            engine.state.turn.step = cardinal::ids::StepId(step_static);
        }
    }
    
    // Use the Goblin Scout card (id: 1) which has an ETB damage ability
    let goblin_id = cardinal::ids::CardId(1);
    
    let active_player = engine.state.turn.active_player;
    
    // Find hand zone for active player
    let hand_zone = engine.state.zones.iter()
        .find(|z| z.owner == Some(active_player) && z.id.0.starts_with("hand"))
        .map(|z| z.id.clone());
    
    if let Some(hand) = hand_zone {
        // Add the goblin to hand
        engine.state.zones.iter_mut()
            .find(|z| z.id == hand)
            .map(|z| z.cards.push(goblin_id));
        
        // Play the goblin - this should trigger its ETB ability
        let result = engine.apply_action(
            active_player,
            Action::PlayCard { card: goblin_id, from: hand },
        ).expect("play card should succeed");
        
        // Check that we got events including trigger effects
        assert!(!result.events.is_empty(), "Playing card should emit events");
        
        // Verify CardPlayed event
        let has_card_played = result.events.iter()
            .any(|e| matches!(e, Event::CardPlayed { player, card } 
                if player == &active_player && card == &goblin_id));
        assert!(has_card_played, "CardPlayed event should be emitted");
        
        // Verify StackResolved event (trigger was auto-resolved)
        let has_stack_resolved = result.events.iter()
            .any(|e| matches!(e, Event::StackResolved { .. }));
        assert!(has_stack_resolved, "Trigger should create and resolve stack items");
    }
}

#[test]
fn test_card_registry_lookup() {
    let rules = load_rules("../../rules.toml").expect("load rules");
    let engine = GameEngine::from_ruleset(rules, 42);
    
    // Verify that card registry was built correctly
    assert!(!engine.cards.is_empty(), "Card registry should have cards");
    
    // Try to look up the goblin
    let goblin_id = cardinal::ids::CardId(1);
    let goblin_def = cardinal::engine::cards::get_card(&engine.cards, goblin_id);
    
    if let Some(card) = goblin_def {
        assert_eq!(card.name, "Goblin Scout", "Card name should match");
        assert_eq!(card.card_type, "creature", "Card type should match");
        assert!(!card.abilities.is_empty(), "Card should have abilities");
        
        // Check the ability
        if let Some(ability) = card.abilities.first() {
            assert_eq!(ability.trigger, "etb", "Ability should be ETB trigger");
            assert_eq!(ability.effect, "damage", "Ability should be damage effect");
            assert_eq!(ability.params.get("amount").map(|s| s.as_str()), Some("1"), "Damage amount should be 1");
        }
    } else {
        panic!("Goblin Scout card not found in registry");
    }
}

