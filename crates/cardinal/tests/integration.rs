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

