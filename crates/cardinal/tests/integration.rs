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
    
    // PassPriority should trigger phase advancement
    let result = engine.apply_action(PlayerId(0), Action::PassPriority)
        .expect("apply action");
    
    // Should have emitted at least one event
    assert!(!result.events.is_empty(), "PassPriority should emit events");
    
    // Check if phase or step advanced
    let phase_advanced = engine.state.turn.phase != initial_phase || engine.state.turn.step != initial_step;
    assert!(phase_advanced, "Phase or step should have advanced after pass");
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
    
    // Pass priority enough times to cycle through the entire turn structure
    // (each pass can advance one step if stack is empty and no pending choice)
    for _ in 0..total_steps + 5 {
        if engine.state.ended.is_some() {
            break; // Game ended, stop
        }
        let _ = engine.apply_action(PlayerId(engine.state.turn.active_player.0), Action::PassPriority);
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
fn test_pass_priority_always_allowed() {
    let rules = load_rules("../../rules.toml").expect("load rules");
    let mut engine = GameEngine::from_ruleset(rules, 42);
    
    // PassPriority should always be allowed, even for inactive players
    if engine.state.players.len() > 1 {
        let inactive_player = if engine.state.turn.active_player == PlayerId(0) {
            PlayerId(1)
        } else {
            PlayerId(0)
        };
        
        let result = engine.apply_action(inactive_player, Action::PassPriority);
        assert!(result.is_ok(), "PassPriority should always be allowed");
    }
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

