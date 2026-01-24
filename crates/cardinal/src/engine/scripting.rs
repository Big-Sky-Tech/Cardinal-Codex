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
        // Helper: deal_damage(target: i32, amount: i32) -> Dynamic
        engine.register_fn("deal_damage", |target: i32, amount: i32| {
            let mut map = rhai::Map::new();
            map.insert("type".into(), Dynamic::from("damage"));
            map.insert("target".into(), Dynamic::from(target));
            map.insert("amount".into(), Dynamic::from(amount));
            Dynamic::from(map)
        });
        
        // Helper: draw_cards(player: i32, count: i32) -> Dynamic
        engine.register_fn("draw_cards", |player: i32, count: i32| {
            let mut map = rhai::Map::new();
            map.insert("type".into(), Dynamic::from("draw"));
            map.insert("player".into(), Dynamic::from(player));
            map.insert("count".into(), Dynamic::from(count));
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
        
        // Helper: pump_creature(card: i32, power: i32, toughness: i32) -> Dynamic
        engine.register_fn("pump_creature", |card: i32, power: i32, toughness: i32| {
            let mut map = rhai::Map::new();
            map.insert("type".into(), Dynamic::from("pump"));
            map.insert("card".into(), Dynamic::from(card));
            map.insert("power".into(), Dynamic::from(power));
            map.insert("toughness".into(), Dynamic::from(toughness));
            Dynamic::from(map)
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
#[derive(Debug, Clone)]
pub struct ScriptContext {
    pub controller: u8,
    pub source_card: u32,
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
        };
        
        let result = engine.execute_ability("test_card", context);
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 2);
    }
}
