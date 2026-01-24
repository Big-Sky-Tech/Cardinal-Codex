use std::collections::{HashMap, HashSet};
use crate::{
    ids::CardId,
    rules::schema::{CardDef, Ruleset},
    model::command::{Command, StackItem, EffectRef},
};

/// Maps card IDs to their definitions for O(1) lookup during gameplay
pub type CardRegistry = HashMap<u32, CardDef>;

/// Build a card registry from card definitions with validation
pub fn build_registry(cards: &[CardDef]) -> CardRegistry {
    let mut registry = HashMap::new();
    
    for card_def in cards {
        // Parse card ID as u32 if it's numeric, otherwise skip
        if let Ok(card_id) = card_def.id.parse::<u32>() {
            registry.insert(card_id, card_def.clone());
        }
    }
    
    registry
}

/// Build a card registry from card definitions with ruleset validation
/// This validates that cards only reference keywords defined in the ruleset
pub fn build_validated_registry(cards: &[CardDef], ruleset: &Ruleset) -> Result<CardRegistry, String> {
    let mut registry = HashMap::new();
    
    // Build set of valid keyword IDs from ruleset
    let valid_keywords: HashSet<String> = ruleset.keywords.iter()
        .map(|k| k.id.clone())
        .collect();
    
    for card_def in cards {
        // Validate keywords - each keyword must exist in ruleset
        for keyword in &card_def.keywords {
            if !valid_keywords.contains(keyword) {
                return Err(format!(
                    "Card '{}' (ID: {}) references undefined keyword '{}'. Valid keywords: {:?}",
                    card_def.name,
                    card_def.id,
                    keyword,
                    valid_keywords
                ));
            }
        }
        
        // Parse card ID as u32 if it's numeric, otherwise skip
        if let Ok(card_id) = card_def.id.parse::<u32>() {
            registry.insert(card_id, card_def.clone());
        }
    }
    
    Ok(registry)
}

/// Get a card definition by ID
pub fn get_card(registry: &CardRegistry, card_id: CardId) -> Option<&CardDef> {
    registry.get(&card_id.0)
}

/// Generate commands from a card's abilities when an event matches a trigger
pub fn generate_ability_commands(
    card_id: CardId,
    event_trigger: &str,
    controller: crate::ids::PlayerId,
    registry: &CardRegistry,
    next_stack_id: &mut u32,
) -> Vec<Command> {
    let mut commands = Vec::new();
    
    if let Some(card_def) = get_card(registry, card_id) {
        for ability in &card_def.abilities {
            // Only fire if the trigger matches
            if ability.trigger == event_trigger {
                // Generate a command for this ability effect
                if let Some(cmd) = effect_to_command(
                    card_id,
                    &ability.effect,
                    &ability.params,
                    controller,
                    next_stack_id,
                ) {
                    commands.push(cmd);
                }
            }
        }
    }
    
    commands
}

/// Convert a card ability effect into an engine Command
fn effect_to_command(
    source: CardId,
    effect_kind: &str,
    params: &std::collections::HashMap<String, String>,
    controller: crate::ids::PlayerId,
    stack_id: &mut u32,
) -> Option<Command> {
    let id = *stack_id;
    *stack_id += 1;

    // Check if this is a scripted effect (indicated by "script:" prefix)
    if effect_kind.starts_with("script:") {
        let script_name = effect_kind.strip_prefix("script:").unwrap_or(effect_kind);
        
        return Some(Command::PushStack {
            item: StackItem {
                id,
                source: Some(source),
                controller,
                effect: EffectRef::Scripted(script_name.to_string()),
            },
        });
    }

    match effect_kind {
        "damage" => {
            let amount = params.get("amount")
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(1);
            
            let effect_str = Box::leak(format!("damage_{}", amount).into_boxed_str());
            
            Some(Command::PushStack {
                item: StackItem {
                    id,
                    source: Some(source),
                    controller,
                    effect: EffectRef::Builtin(effect_str),
                },
            })
        }
        "draw" => {
            let amount = params.get("amount")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(1);
            
            let effect_str = Box::leak(format!("draw_{}", amount).into_boxed_str());
            
            Some(Command::PushStack {
                item: StackItem {
                    id,
                    source: Some(source),
                    controller,
                    effect: EffectRef::Builtin(effect_str),
                },
            })
        }
        "gain_life" => {
            let amount = params.get("amount")
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(1);
            
            let effect_str = Box::leak(format!("gain_life_{}", amount).into_boxed_str());
            
            Some(Command::PushStack {
                item: StackItem {
                    id,
                    source: Some(source),
                    controller,
                    effect: EffectRef::Builtin(effect_str),
                },
            })
        }
        "pump" => {
            let power = params.get("power")
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(1);
            let toughness = params.get("toughness")
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(1);
            
            let effect_str = Box::leak(format!("pump_{}_{}", power, toughness).into_boxed_str());
            
            Some(Command::PushStack {
                item: StackItem {
                    id,
                    source: Some(source),
                    controller,
                    effect: EffectRef::Builtin(effect_str),
                },
            })
        }
        _ => {
            // Unknown effect type - skip
            None
        }
    }
}

/// Check if a card has a specific keyword
pub fn card_has_keyword(card_def: &CardDef, keyword_id: &str) -> bool {
    card_def.keywords.iter().any(|k| k == keyword_id)
}

/// Get a card's stat value by key
pub fn get_card_stat<'a>(card_def: &'a CardDef, stat_key: &str) -> Option<&'a String> {
    card_def.stats.get(stat_key)
}

/// Get a card's stat as an integer
pub fn get_card_stat_i32(card_def: &CardDef, stat_key: &str) -> Option<i32> {
    card_def.stats.get(stat_key)
        .and_then(|s| s.parse::<i32>().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::schema::{Keyword, Ruleset, GameInfo, PlayerRules, TurnStructure};
    
    fn minimal_ruleset() -> Ruleset {
        Ruleset {
            game: GameInfo {
                id: "test".to_string(),
                name: "Test Game".to_string(),
                version: "1.0".to_string(),
                description: "Test".to_string(),
            },
            players: PlayerRules {
                min_players: 2,
                max_players: 2,
                starting_life: 20,
                max_life: 100,
                starting_hand_size: 5,
                max_hand_size: 10,
                min_deck_size: 40,
                max_deck_size: 60,
                mulligan_rule: "none".to_string(),
                first_player_rule: "random".to_string(),
            },
            zones: vec![],
            resources: vec![],
            turn: TurnStructure {
                priority_system: true,
                skip_first_turn_draw_for_first_player: false,
                phases: vec![],
            },
            actions: vec![],
            stack: crate::rules::schema::StackRules {
                enabled: true,
                resolve_order: "lifo".to_string(),
                auto_resolve_on_pass: true,
            },
            trigger_kinds: vec![],
            keywords: vec![
                Keyword {
                    id: "flying".to_string(),
                    name: "Flying".to_string(),
                    description: "Can only be blocked by flying creatures".to_string(),
                },
                Keyword {
                    id: "quick".to_string(),
                    name: "Quick".to_string(),
                    description: "Can be played at instant speed".to_string(),
                },
            ],
            win_conditions: vec![],
            loss_conditions: vec![],
            cards: vec![],
        }
    }
    
    #[test]
    fn test_validate_valid_keywords() {
        let ruleset = minimal_ruleset();
        let mut card = CardDef {
            id: "1".to_string(),
            name: "Test Card".to_string(),
            card_type: "creature".to_string(),
            cost: None,
            description: None,
            abilities: vec![],
            script_path: None,
            keywords: vec!["flying".to_string()],
            stats: std::collections::HashMap::new(),
        };
        
        let result = build_validated_registry(&[card.clone()], &ruleset);
        assert!(result.is_ok());
        
        // Test with multiple valid keywords
        card.keywords = vec!["flying".to_string(), "quick".to_string()];
        let result = build_validated_registry(&[card], &ruleset);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_invalid_keyword() {
        let ruleset = minimal_ruleset();
        let card = CardDef {
            id: "1".to_string(),
            name: "Test Card".to_string(),
            card_type: "creature".to_string(),
            cost: None,
            description: None,
            abilities: vec![],
            script_path: None,
            keywords: vec!["invalid_keyword".to_string()],
            stats: std::collections::HashMap::new(),
        };
        
        let result = build_validated_registry(&[card], &ruleset);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("undefined keyword"));
    }
    
    #[test]
    fn test_card_has_keyword() {
        let card = CardDef {
            id: "1".to_string(),
            name: "Test Card".to_string(),
            card_type: "creature".to_string(),
            cost: None,
            description: None,
            abilities: vec![],
            script_path: None,
            keywords: vec!["flying".to_string(), "quick".to_string()],
            stats: std::collections::HashMap::new(),
        };
        
        assert!(card_has_keyword(&card, "flying"));
        assert!(card_has_keyword(&card, "quick"));
        assert!(!card_has_keyword(&card, "haste"));
    }
    
    #[test]
    fn test_get_card_stats() {
        let mut stats = std::collections::HashMap::new();
        stats.insert("power".to_string(), "3".to_string());
        stats.insert("toughness".to_string(), "4".to_string());
        
        let card = CardDef {
            id: "1".to_string(),
            name: "Test Creature".to_string(),
            card_type: "creature".to_string(),
            cost: None,
            description: None,
            abilities: vec![],
            script_path: None,
            keywords: vec![],
            stats,
        };
        
        assert_eq!(get_card_stat(&card, "power"), Some(&"3".to_string()));
        assert_eq!(get_card_stat_i32(&card, "power"), Some(3));
        assert_eq!(get_card_stat_i32(&card, "toughness"), Some(4));
        assert_eq!(get_card_stat(&card, "missing"), None);
    }
}
