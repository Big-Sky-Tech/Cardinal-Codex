use std::collections::HashMap;
use crate::{
    ids::CardId,
    rules::schema::CardDef,
    model::command::{Command, StackItem, EffectRef},
};

/// Maps card IDs to their definitions for O(1) lookup during gameplay
pub type CardRegistry = HashMap<u32, CardDef>;

/// Build a card registry from card definitions
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
