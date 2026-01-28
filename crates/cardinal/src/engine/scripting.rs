use rhai::{Engine, AST, Scope, Dynamic};
use std::collections::HashMap;
use crate::error::CardinalError;

/// Wrapper around Rhai engine for executing card scripts
/// Configured for deterministic, safe execution
pub struct RhaiEngine {
    engine: Engine,
    /// Compiled scripts indexed by card ID
    scripts: HashMap<String, AST>,
}

impl RhaiEngine {
    /// Create a new RhaiEngine configured for Cardinal
    pub fn new() -> Self {
        let mut engine = Engine::new();
        
        // Configure for determinism and safety
        engine.set_max_operations(10_000); // Prevent infinite loops
        engine.set_max_expr_depths(32, 32); // Limit recursion
        
        // Register safe helper functions that scripts can call
        Self::register_helpers(&mut engine);
        
        RhaiEngine {
            engine,
            scripts: HashMap::new(),
        }
    }
    
    /// Register helper functions available to card scripts
    fn register_helpers(engine: &mut Engine) {
        // ==============================================
        // DAMAGE & LIFE HELPERS
        // ==============================================
        
        // Helper: deal_damage(target: i32, amount: i32) -> Dynamic
        engine.register_fn("deal_damage", |target: i32, amount: i32| {
            let mut map = rhai::Map::new();
            map.insert("type".into(), Dynamic::from("damage"));
            map.insert("target".into(), Dynamic::from(target));
            map.insert("amount".into(), Dynamic::from(amount));
            Dynamic::from(map)
        });
        
        // Helper: gain_life(player: i32, amount: i32) -> Dynamic
        engine.register_fn("gain_life", |player: i32, amount: i32| {
            let mut map = rhai::Map::new();
            map.insert("type".into(), Dynamic::from("gain_life"));
            map.insert("player".into(), Dynamic::from(player));
            map.insert("amount".into(), Dynamic::from(amount));
            Dynamic::from(map)
        });
        
        // Helper: lose_life(player: i32, amount: i32) -> Dynamic
        // Distinct from damage - loss of life doesn't trigger damage effects
        engine.register_fn("lose_life", |player: i32, amount: i32| {
            let mut map = rhai::Map::new();
            map.insert("type".into(), Dynamic::from("lose_life"));
            map.insert("player".into(), Dynamic::from(player));
            map.insert("amount".into(), Dynamic::from(amount));
            Dynamic::from(map)
        });
        
        // Helper: set_life(player: i32, amount: i32) -> Dynamic
        // Set a player's life to a specific value
        engine.register_fn("set_life", |player: i32, amount: i32| {
            let mut map = rhai::Map::new();
            map.insert("type".into(), Dynamic::from("set_life"));
            map.insert("player".into(), Dynamic::from(player));
            map.insert("amount".into(), Dynamic::from(amount));
            Dynamic::from(map)
        });
        
        // ==============================================
        // CARD DRAW & ZONE MOVEMENT HELPERS
        // ==============================================
        
        // Helper: draw_cards(player: i32, count: i32) -> Dynamic
        engine.register_fn("draw_cards", |player: i32, count: i32| {
            let mut map = rhai::Map::new();
            map.insert("type".into(), Dynamic::from("draw"));
            map.insert("player".into(), Dynamic::from(player));
            map.insert("count".into(), Dynamic::from(count));
            Dynamic::from(map)
        });
        
        // Helper: mill_cards(player: i32, count: i32) -> Dynamic
        // Move cards from top of deck to graveyard
        engine.register_fn("mill_cards", |player: i32, count: i32| {
            let mut map = rhai::Map::new();
            map.insert("type".into(), Dynamic::from("mill"));
            map.insert("player".into(), Dynamic::from(player));
            map.insert("count".into(), Dynamic::from(count));
            Dynamic::from(map)
        });
        
        // Helper: discard_cards(player: i32, count: i32) -> Dynamic
        // Move cards from hand to graveyard
        engine.register_fn("discard_cards", |player: i32, count: i32| {
            let mut map = rhai::Map::new();
            map.insert("type".into(), Dynamic::from("discard"));
            map.insert("player".into(), Dynamic::from(player));
            map.insert("count".into(), Dynamic::from(count));
            Dynamic::from(map)
        });
        
        // Helper: move_card(card: i32, from_zone: &str, to_zone: &str) -> Dynamic
        // General purpose card movement between zones
        engine.register_fn("move_card", |card: i32, from_zone: &str, to_zone: &str| {
            let mut map = rhai::Map::new();
            map.insert("type".into(), Dynamic::from("move_card"));
            map.insert("card".into(), Dynamic::from(card));
            map.insert("from_zone".into(), Dynamic::from(from_zone.to_string()));
            map.insert("to_zone".into(), Dynamic::from(to_zone.to_string()));
            Dynamic::from(map)
        });
        
        // Helper: shuffle_zone(player: i32, zone: &str) -> Dynamic
        // Shuffle a zone (typically deck)
        engine.register_fn("shuffle_zone", |player: i32, zone: &str| {
            let mut map = rhai::Map::new();
            map.insert("type".into(), Dynamic::from("shuffle_zone"));
            map.insert("player".into(), Dynamic::from(player));
            map.insert("zone".into(), Dynamic::from(zone.to_string()));
            Dynamic::from(map)
        });
        
        // ==============================================
        // CREATURE & STAT MODIFICATION HELPERS
        // ==============================================
        
        // Helper: pump_creature(card: i32, power: i32, toughness: i32) -> Dynamic
        engine.register_fn("pump_creature", |card: i32, power: i32, toughness: i32| {
            let mut map = rhai::Map::new();
            map.insert("type".into(), Dynamic::from("pump"));
            map.insert("card".into(), Dynamic::from(card));
            map.insert("power".into(), Dynamic::from(power));
            map.insert("toughness".into(), Dynamic::from(toughness));
            Dynamic::from(map)
        });
        
        // Helper: set_stats(card: i32, power: i32, toughness: i32) -> Dynamic
        // Set creature stats to specific values (not delta)
        engine.register_fn("set_stats", |card: i32, power: i32, toughness: i32| {
            let mut map = rhai::Map::new();
            map.insert("type".into(), Dynamic::from("set_stats"));
            map.insert("card".into(), Dynamic::from(card));
            map.insert("power".into(), Dynamic::from(power));
            map.insert("toughness".into(), Dynamic::from(toughness));
            Dynamic::from(map)
        });
        
        // Helper: modify_stat(card: i32, stat_name: &str, delta: i32) -> Dynamic
        // Modify any card stat by name (generic stat modification)
        engine.register_fn("modify_stat", |card: i32, stat_name: &str, delta: i32| {
            let mut map = rhai::Map::new();
            map.insert("type".into(), Dynamic::from("modify_stat"));
            map.insert("card".into(), Dynamic::from(card));
            map.insert("stat_name".into(), Dynamic::from(stat_name.to_string()));
            map.insert("delta".into(), Dynamic::from(delta));
            Dynamic::from(map)
        });
        
        // Helper: set_stat(card: i32, stat_name: &str, value: &str) -> Dynamic
        // Set any card stat to a specific value
        engine.register_fn("set_stat", |card: i32, stat_name: &str, value: &str| {
            let mut map = rhai::Map::new();
            map.insert("type".into(), Dynamic::from("set_stat"));
            map.insert("card".into(), Dynamic::from(card));
            map.insert("stat_name".into(), Dynamic::from(stat_name.to_string()));
            map.insert("value".into(), Dynamic::from(value.to_string()));
            Dynamic::from(map)
        });
        
        // ==============================================
        // KEYWORD MANIPULATION HELPERS
        // ==============================================
        
        // Helper: grant_keyword(card: i32, keyword: &str) -> Dynamic
        // Grant a keyword to a card (e.g., "flying", "quick")
        engine.register_fn("grant_keyword", |card: i32, keyword: &str| {
            let mut map = rhai::Map::new();
            map.insert("type".into(), Dynamic::from("grant_keyword"));
            map.insert("card".into(), Dynamic::from(card));
            map.insert("keyword".into(), Dynamic::from(keyword.to_string()));
            Dynamic::from(map)
        });
        
        // Helper: remove_keyword(card: i32, keyword: &str) -> Dynamic
        // Remove a keyword from a card
        engine.register_fn("remove_keyword", |card: i32, keyword: &str| {
            let mut map = rhai::Map::new();
            map.insert("type".into(), Dynamic::from("remove_keyword"));
            map.insert("card".into(), Dynamic::from(card));
            map.insert("keyword".into(), Dynamic::from(keyword.to_string()));
            Dynamic::from(map)
        });
        
        // ==============================================
        // RESOURCE MANIPULATION HELPERS
        // ==============================================
        
        // Helper: gain_resource(player: i32, resource: &str, amount: i32) -> Dynamic
        // Grant resources to a player (e.g., mana, action points)
        engine.register_fn("gain_resource", |player: i32, resource: &str, amount: i32| {
            let mut map = rhai::Map::new();
            map.insert("type".into(), Dynamic::from("gain_resource"));
            map.insert("player".into(), Dynamic::from(player));
            map.insert("resource".into(), Dynamic::from(resource.to_string()));
            map.insert("amount".into(), Dynamic::from(amount));
            Dynamic::from(map)
        });
        
        // Helper: spend_resource(player: i32, resource: &str, amount: i32) -> Dynamic
        // Spend/consume resources
        engine.register_fn("spend_resource", |player: i32, resource: &str, amount: i32| {
            let mut map = rhai::Map::new();
            map.insert("type".into(), Dynamic::from("spend_resource"));
            map.insert("player".into(), Dynamic::from(player));
            map.insert("resource".into(), Dynamic::from(resource.to_string()));
            map.insert("amount".into(), Dynamic::from(amount));
            Dynamic::from(map)
        });
        
        // Helper: set_resource(player: i32, resource: &str, amount: i32) -> Dynamic
        // Set resource to a specific value
        engine.register_fn("set_resource", |player: i32, resource: &str, amount: i32| {
            let mut map = rhai::Map::new();
            map.insert("type".into(), Dynamic::from("set_resource"));
            map.insert("player".into(), Dynamic::from(player));
            map.insert("resource".into(), Dynamic::from(resource.to_string()));
            map.insert("amount".into(), Dynamic::from(amount));
            Dynamic::from(map)
        });
        
        // ==============================================
        // TOKEN & CARD CREATION HELPERS
        // ==============================================
        
        // Helper: create_token(player: i32, token_type: &str, zone: &str) -> Dynamic
        // Create a token card in a specified zone
        engine.register_fn("create_token", |player: i32, token_type: &str, zone: &str| {
            let mut map = rhai::Map::new();
            map.insert("type".into(), Dynamic::from("create_token"));
            map.insert("player".into(), Dynamic::from(player));
            map.insert("token_type".into(), Dynamic::from(token_type.to_string()));
            map.insert("zone".into(), Dynamic::from(zone.to_string()));
            Dynamic::from(map)
        });
        
        // ==============================================
        // COUNTER & MARKER HELPERS
        // ==============================================
        
        // Helper: add_counter(card: i32, counter_type: &str, amount: i32) -> Dynamic
        // Add counters to a card (e.g., +1/+1 counters, charge counters)
        engine.register_fn("add_counter", |card: i32, counter_type: &str, amount: i32| {
            let mut map = rhai::Map::new();
            map.insert("type".into(), Dynamic::from("add_counter"));
            map.insert("card".into(), Dynamic::from(card));
            map.insert("counter_type".into(), Dynamic::from(counter_type.to_string()));
            map.insert("amount".into(), Dynamic::from(amount));
            Dynamic::from(map)
        });
        
        // Helper: remove_counter(card: i32, counter_type: &str, amount: i32) -> Dynamic
        // Remove counters from a card
        engine.register_fn("remove_counter", |card: i32, counter_type: &str, amount: i32| {
            let mut map = rhai::Map::new();
            map.insert("type".into(), Dynamic::from("remove_counter"));
            map.insert("card".into(), Dynamic::from(card));
            map.insert("counter_type".into(), Dynamic::from(counter_type.to_string()));
            map.insert("amount".into(), Dynamic::from(amount));
            Dynamic::from(map)
        });
        
        // ==============================================
        // TYPE HELPERS - Common Patterns
        // ==============================================
        
        // Helper: bolt(target: i32, damage: i32) -> Dynamic
        // Common pattern: simple direct damage
        engine.register_fn("bolt", |target: i32, damage: i32| {
            let mut map = rhai::Map::new();
            map.insert("type".into(), Dynamic::from("damage"));
            map.insert("target".into(), Dynamic::from(target));
            map.insert("amount".into(), Dynamic::from(damage));
            Dynamic::from(map)
        });
        
        // Helper: drain(target: i32, amount: i32, controller: i32) -> Dynamic array
        // Common pattern: damage opponent, gain life
        engine.register_fn("drain", |target: i32, amount: i32, controller: i32| {
            let mut damage_map = rhai::Map::new();
            damage_map.insert("type".into(), Dynamic::from("damage"));
            damage_map.insert("target".into(), Dynamic::from(target));
            damage_map.insert("amount".into(), Dynamic::from(amount));
            
            let mut heal_map = rhai::Map::new();
            heal_map.insert("type".into(), Dynamic::from("gain_life"));
            heal_map.insert("player".into(), Dynamic::from(controller));
            heal_map.insert("amount".into(), Dynamic::from(amount));
            
            vec![Dynamic::from(damage_map), Dynamic::from(heal_map)]
        });
        
        // Helper: cantrip(player: i32, effect: Dynamic) -> Dynamic array
        // Common pattern: effect + draw a card
        engine.register_fn("cantrip", |player: i32, effect: Dynamic| {
            let mut draw_map = rhai::Map::new();
            draw_map.insert("type".into(), Dynamic::from("draw"));
            draw_map.insert("player".into(), Dynamic::from(player));
            draw_map.insert("count".into(), Dynamic::from(1));
            
            vec![effect, Dynamic::from(draw_map)]
        });
    }
    
    /// Register a card script from source code
    pub fn register_script(&mut self, card_id: String, script: &str) -> Result<(), CardinalError> {
        match self.engine.compile(script) {
            Ok(ast) => {
                self.scripts.insert(card_id, ast);
                Ok(())
            }
            Err(err) => {
                Err(CardinalError(format!("Failed to compile script for card {}: {}", card_id, err)))
            }
        }
    }
    
    /// Execute a card script's ability
    /// Returns a list of effect descriptions as Dynamic values
    pub fn execute_ability(&self, card_id: &str, context: ScriptContext) -> Result<Vec<Dynamic>, CardinalError> {
        let ast = self.scripts.get(card_id)
            .ok_or_else(|| CardinalError(format!("No script registered for card {}", card_id)))?;
        
        let mut scope = Scope::new();
        
        // Pass context to script
        scope.push("controller", context.controller as i32);
        scope.push("source_card", context.source_card as i32);
        
        // Pass optional context fields if available
        if let Some(active) = context.active_player {
            scope.push("active_player", active as i32);
        }
        if let Some(turn) = context.turn_number {
            scope.push("turn_number", turn as i32);
        }
        if let Some(ref phase) = context.phase {
            scope.push("phase", phase.clone());
        }
        
        // Call the execute_ability function in the script
        match self.engine.call_fn::<Dynamic>(&mut scope, ast, "execute_ability", ()) {
            Ok(result) => {
                // Convert result to Vec<Dynamic>
                // Script should return an array of command maps
                if let Some(arr) = result.clone().try_cast::<rhai::Array>() {
                    Ok(arr)
                } else {
                    // Single command, wrap in array
                    Ok(vec![result])
                }
            }
            Err(err) => {
                Err(CardinalError(format!("Script execution failed for card {}: {}", card_id, err)))
            }
        }
    }
}

/// Context passed to script execution
/// Contains runtime information scripts need to make decisions
#[derive(Debug, Clone)]
pub struct ScriptContext {
    /// The player who controls the card/effect
    pub controller: u8,
    /// The card that is the source of this effect
    pub source_card: u32,
    /// Optional: the active player's ID
    pub active_player: Option<u8>,
    /// Optional: current turn number
    pub turn_number: Option<u32>,
    /// Optional: current phase ID
    pub phase: Option<String>,
}

impl Default for RhaiEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rhai_engine_creation() {
        let engine = RhaiEngine::new();
        assert_eq!(engine.scripts.len(), 0);
    }
    
    #[test]
    fn test_register_simple_script() {
        let mut engine = RhaiEngine::new();
        let script = r#"
            fn execute_ability() {
                deal_damage(0, 2)
            }
        "#;
        
        let result = engine.register_script("test_card".to_string(), script);
        assert!(result.is_ok());
        assert_eq!(engine.scripts.len(), 1);
    }
    
    #[test]
    fn test_execute_simple_script() {
        let mut engine = RhaiEngine::new();
        let script = r#"
            fn execute_ability() {
                deal_damage(0, 2)
            }
        "#;
        
        engine.register_script("test_card".to_string(), script).unwrap();
        
        let context = ScriptContext {
            controller: 0,
            source_card: 1,
            active_player: None,
            turn_number: None,
            phase: None,
        };
        
        let result = engine.execute_ability("test_card", context);
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
    }
    
    #[test]
    fn test_execute_multi_command_script() {
        let mut engine = RhaiEngine::new();
        let script = r#"
            fn execute_ability() {
                [
                    deal_damage(0, 2),
                    draw_cards(0, 1)
                ]
            }
        "#;
        
        engine.register_script("test_card".to_string(), script).unwrap();
        
        let context = ScriptContext {
            controller: 0,
            source_card: 1,
            active_player: None,
            turn_number: None,
            phase: None,
        };
        
        let result = engine.execute_ability("test_card", context);
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 2);
    }
    
    #[test]
    fn test_new_helper_functions() {
        let mut engine = RhaiEngine::new();
        
        // Test various new helper functions
        let script = r#"
            fn execute_ability() {
                [
                    lose_life(1, 2),
                    mill_cards(1, 3),
                    discard_cards(0, 1),
                    grant_keyword(source_card, "flying"),
                    gain_resource(controller, "mana", 5),
                    create_token(controller, "soldier", "field"),
                    add_counter(source_card, "+1/+1", 2),
                    modify_stat(source_card, "range", 1)
                ]
            }
        "#;
        
        engine.register_script("advanced_card".to_string(), script).unwrap();
        
        let context = ScriptContext {
            controller: 0,
            source_card: 1,
            active_player: Some(0),
            turn_number: Some(3),
            phase: Some("main1".to_string()),
        };
        
        let result = engine.execute_ability("advanced_card", context);
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 8);
        
        // Verify each command has the correct type
        for cmd in &commands {
            if let Some(map) = cmd.clone().try_cast::<rhai::Map>() {
                assert!(map.contains_key("type"));
            } else {
                panic!("Expected map");
            }
        }
    }
    
    #[test]
    fn test_type_helper_drain() {
        let mut engine = RhaiEngine::new();
        let script = r#"
            fn execute_ability() {
                drain(1, 3, controller)
            }
        "#;
        
        engine.register_script("drain_card".to_string(), script).unwrap();
        
        let context = ScriptContext {
            controller: 0,
            source_card: 1,
            active_player: None,
            turn_number: None,
            phase: None,
        };
        
        let result = engine.execute_ability("drain_card", context);
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 2); // drain returns 2 effects: damage + heal
    }
    
    #[test]
    fn test_type_helper_cantrip() {
        let mut engine = RhaiEngine::new();
        let script = r#"
            fn execute_ability() {
                cantrip(controller, bolt(1, 2))
            }
        "#;
        
        engine.register_script("cantrip_card".to_string(), script).unwrap();
        
        let context = ScriptContext {
            controller: 0,
            source_card: 1,
            active_player: None,
            turn_number: None,
            phase: None,
        };
        
        let result = engine.execute_ability("cantrip_card", context);
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 2); // cantrip returns effect + draw
    }
    
    #[test]
    fn test_context_variables() {
        let mut engine = RhaiEngine::new();
        let script = r#"
            fn execute_ability() {
                // Use optional context variables
                if active_player == 0 {
                    gain_life(controller, 5)
                } else {
                    gain_life(controller, 3)
                }
            }
        "#;
        
        engine.register_script("context_card".to_string(), script).unwrap();
        
        let context = ScriptContext {
            controller: 0,
            source_card: 1,
            active_player: Some(0),
            turn_number: Some(5),
            phase: Some("main1".to_string()),
        };
        
        let result = engine.execute_ability("context_card", context);
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
    }
}
