use crate::{
    ids::{CardId, PlayerId},
    model::command::{Command, EffectRef},
    state::gamestate::GameState,
    error::CardinalError,
};

/// Execute an effect and return commands to apply its results
/// This handles three types of effects:
/// 1. Builtin effects (damage, draw, gain_life, pump) - parsed from effect string
/// 2. Data-driven effects - future: loaded from TOML params
/// 3. Scripted effects - future: executed via Rhai
pub fn execute_effect(
    effect: &EffectRef,
    _source: Option<CardId>,
    controller: PlayerId,
    _state: &GameState,
) -> Result<Vec<Command>, CardinalError> {
    match effect {
        EffectRef::Builtin(effect_str) => execute_builtin_effect(effect_str, controller),
        EffectRef::Scripted(_script_name) => {
            // TODO: Execute scripted effect via RhaiEngine
            // For now, return empty command list
            Ok(vec![])
        }
    }
}

/// Execute a builtin effect parsed from its string representation
/// Format: "{effect_type}_{param1}_{param2}..."
/// Examples: "damage_2", "draw_1", "gain_life_3", "pump_1_1"
fn execute_builtin_effect(effect_str: &str, controller: PlayerId) -> Result<Vec<Command>, CardinalError> {
    // Handle different effect patterns
    if effect_str.starts_with("damage_") {
        let amount = effect_str.strip_prefix("damage_")
            .and_then(|s| s.parse::<i32>().ok())
            .ok_or_else(|| CardinalError(format!("Invalid damage amount in: {}", effect_str)))?;
        
        // For now, damage goes to the controller (later: target selection)
        Ok(vec![Command::ChangeLife {
            player: controller,
            delta: -amount,
        }])
    } else if effect_str.starts_with("draw_") {
        let _count = effect_str.strip_prefix("draw_")
            .and_then(|s| s.parse::<u32>().ok())
            .ok_or_else(|| CardinalError(format!("Invalid draw count in: {}", effect_str)))?;
        
        // TODO: Implement card drawing
        // For now, return empty (no MoveCard commands yet)
        Ok(vec![])
    } else if effect_str.starts_with("gain_life_") {
        let amount = effect_str.strip_prefix("gain_life_")
            .and_then(|s| s.parse::<i32>().ok())
            .ok_or_else(|| CardinalError(format!("Invalid life amount in: {}", effect_str)))?;
        
        Ok(vec![Command::ChangeLife {
            player: controller,
            delta: amount,
        }])
    } else if effect_str.starts_with("pump_") {
        let parts: Vec<&str> = effect_str.strip_prefix("pump_")
            .unwrap_or("")
            .split('_')
            .collect();
        
        let _power = parts.get(0)
            .and_then(|s| s.parse::<i32>().ok())
            .ok_or_else(|| CardinalError(format!("Invalid power in: {}", effect_str)))?;
        let _toughness = parts.get(1)
            .and_then(|s| s.parse::<i32>().ok())
            .ok_or_else(|| CardinalError(format!("Invalid toughness in: {}", effect_str)))?;
        
        // TODO: Implement creature stat modification
        // For now, return empty (no creature tracking yet)
        Ok(vec![])
    } else {
        Err(CardinalError(format!("Unknown builtin effect type: {}", effect_str)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::gamestate::{GameState, TurnState, PlayerState};
    use crate::ids::{PhaseId, StepId};
    
    fn minimal_game_state() -> GameState {
        GameState {
            turn: TurnState {
                number: 1,
                active_player: PlayerId(0),
                priority_player: PlayerId(0),
                phase: PhaseId("main"),
                step: StepId("main"),
                priority_passes: 0,
            },
            players: vec![
                PlayerState { id: PlayerId(0), life: 20 },
                PlayerState { id: PlayerId(1), life: 20 },
            ],
            zones: vec![],
            stack: vec![],
            pending_choice: None,
            ended: None,
        }
    }
    
    #[test]
    fn test_execute_damage_effect() {
        let effect = EffectRef::Builtin("damage_2");
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state);
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::ChangeLife { player, delta } => {
                assert_eq!(*player, controller);
                assert_eq!(*delta, -2);
            }
            _ => panic!("Expected ChangeLife command"),
        }
    }
    
    #[test]
    fn test_execute_gain_life_effect() {
        let effect = EffectRef::Builtin("gain_life_5");
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state);
        if result.is_err() {
            println!("Error: {:?}", result.as_ref().err());
        }
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::ChangeLife { player, delta } => {
                assert_eq!(*player, controller);
                assert_eq!(*delta, 5);
            }
            _ => panic!("Expected ChangeLife command"),
        }
    }
    
    #[test]
    fn test_execute_draw_effect() {
        let effect = EffectRef::Builtin("draw_1");
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state);
        assert!(result.is_ok());
        
        // Draw not yet implemented, should return empty
        let commands = result.unwrap();
        assert_eq!(commands.len(), 0);
    }
    
    #[test]
    fn test_execute_pump_effect() {
        let effect = EffectRef::Builtin("pump_1_1");
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state);
        assert!(result.is_ok());
        
        // Pump not yet implemented, should return empty
        let commands = result.unwrap();
        assert_eq!(commands.len(), 0);
    }
    
    #[test]
    fn test_invalid_effect_string() {
        let effect = EffectRef::Builtin("invalid");
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_invalid_damage_amount() {
        let effect = EffectRef::Builtin("damage_abc");
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state);
        assert!(result.is_err());
    }
}
