use crate::{
    ids::{CardId, PlayerId, ZoneId},
    model::command::{Command, EffectRef},
    state::gamestate::GameState,
    engine::scripting::{RhaiEngine, ScriptContext},
    error::CardinalError,
};

/// Execute an effect and return commands to apply its results
/// This handles three types of effects:
/// 1. Builtin effects (damage, draw, gain_life, pump) - parsed from effect string
/// 2. Data-driven effects - future: loaded from TOML params
/// 3. Scripted effects - executed via Rhai
pub fn execute_effect(
    effect: &EffectRef,
    source: Option<CardId>,
    controller: PlayerId,
    _state: &GameState,
    scripting: Option<&RhaiEngine>,
) -> Result<Vec<Command>, CardinalError> {
    match effect {
        EffectRef::Builtin(effect_str) => execute_builtin_effect(effect_str, controller),
        EffectRef::Scripted(script_name) => {
            if let Some(engine) = scripting {
                execute_scripted_effect(script_name, source, controller, engine)
            } else {
                Err(CardinalError(format!("Cannot execute scripted effect '{}': RhaiEngine not available", script_name)))
            }
        }
    }
}

/// Execute a scripted effect via RhaiEngine
fn execute_scripted_effect(
    script_name: &str,
    source: Option<CardId>,
    controller: PlayerId,
    engine: &RhaiEngine,
) -> Result<Vec<Command>, CardinalError> {
    let context = ScriptContext {
        controller: controller.0,
        source_card: source.map(|c| c.0).unwrap_or(0),
        active_player: None,
        turn_number: None,
        phase: None,
    };
    
    let results = engine.execute_ability(script_name, context)?;
    
    // Convert Rhai Dynamic results into Commands
    let mut commands = Vec::new();
    
    for (index, result) in results.into_iter().enumerate() {
        // Each result must be a map with a "type" field
        let map = result.try_cast::<rhai::Map>()
            .ok_or_else(|| CardinalError(format!(
                "Script '{}' returned non-map value at index {}", 
                script_name, index
            )))?;
        
        let effect_type = map.get("type")
            .ok_or_else(|| CardinalError(format!(
                "Script '{}' result at index {} missing 'type' field",
                script_name, index
            )))?
            .clone()
            .try_cast::<String>()
            .ok_or_else(|| CardinalError(format!(
                "Script '{}' result at index {} has non-string 'type' field",
                script_name, index
            )))?;
        
        match effect_type.as_str() {
            "damage" => {
                let target = extract_i32(&map, "target", script_name)?;
                let amount = extract_i32(&map, "amount", script_name)?;
                
                validate_non_negative(target, "target", script_name)?;
                validate_non_negative(amount, "amount", script_name)?;
                validate_u8_range(target, "target", script_name)?;
                
                commands.push(Command::ChangeLife {
                    player: PlayerId(target as u8),
                    delta: -amount,
                });
            }
            "draw" => {
                let player = extract_i32(&map, "player", script_name)?;
                let count = extract_i32(&map, "count", script_name)?;
                
                validate_non_negative(player, "player", script_name)?;
                validate_u8_range(player, "player", script_name)?;
                validate_positive(count, "count", script_name)?;
                
                // Draw cards: move from deck to hand
                // For now, we don't have deck/hand tracking, so this is a placeholder
                // In a full implementation, this would generate MoveCard commands
            }
            "gain_life" => {
                let player = extract_i32(&map, "player", script_name)?;
                let amount = extract_i32(&map, "amount", script_name)?;
                
                validate_non_negative(player, "player", script_name)?;
                validate_non_negative(amount, "amount", script_name)?;
                validate_u8_range(player, "player", script_name)?;
                
                commands.push(Command::ChangeLife {
                    player: PlayerId(player as u8),
                    delta: amount,
                });
            }
            "lose_life" => {
                let player = extract_i32(&map, "player", script_name)?;
                let amount = extract_i32(&map, "amount", script_name)?;
                
                validate_non_negative(player, "player", script_name)?;
                validate_non_negative(amount, "amount", script_name)?;
                validate_u8_range(player, "player", script_name)?;
                
                commands.push(Command::ChangeLife {
                    player: PlayerId(player as u8),
                    delta: -amount,
                });
            }
            "set_life" => {
                let player = extract_i32(&map, "player", script_name)?;
                let amount = extract_i32(&map, "amount", script_name)?;
                
                validate_non_negative(player, "player", script_name)?;
                validate_non_negative(amount, "amount", script_name)?;
                validate_u8_range(player, "player", script_name)?;
                
                commands.push(Command::SetLife {
                    player: PlayerId(player as u8),
                    amount,
                });
            }
            "mill" => {
                let player = extract_i32(&map, "player", script_name)?;
                let count = extract_i32(&map, "count", script_name)?;
                
                validate_non_negative(player, "player", script_name)?;
                validate_non_negative(count, "count", script_name)?;
                validate_u8_range(player, "player", script_name)?;
                
                // TODO: Implement milling (deck to graveyard)
            }
            "discard" => {
                let player = extract_i32(&map, "player", script_name)?;
                let count = extract_i32(&map, "count", script_name)?;
                
                validate_non_negative(player, "player", script_name)?;
                validate_non_negative(count, "count", script_name)?;
                validate_u8_range(player, "player", script_name)?;
                
                // TODO: Implement discarding (hand to graveyard)
            }
            "move_card" => {
                let card = extract_i32(&map, "card", script_name)?;
                let from_zone = extract_string(&map, "from_zone", script_name)?;
                let to_zone = extract_string(&map, "to_zone", script_name)?;
                
                validate_non_negative(card, "card", script_name)?;
                
                // Convert zone strings to ZoneId
                let from_zone_id = string_to_zone_id(&from_zone);
                let to_zone_id = string_to_zone_id(&to_zone);
                
                commands.push(Command::MoveCard {
                    card: CardId(card as u32),
                    from: from_zone_id,
                    to: to_zone_id,
                });
            }
            "shuffle_zone" => {
                let _player = extract_i32(&map, "player", script_name)?;
                let _zone = extract_string(&map, "zone", script_name)?;
                
                // NOTE: ShuffleZone is intentionally left unimplemented.
                // A correct implementation must use the engine-owned RNG to deterministically
                // reorder cards in the target zone within GameState. Until proper shuffling
                // is wired up, this effect must not be used in live rules/effects.
                return Err(CardinalError(
                    "shuffle_zone effect is not yet implemented: it must update GameState and use the engine RNG to shuffle the zone".to_string()
                ));
            }
            "pump" => {
                let card = extract_i32(&map, "card", script_name)?;
                let power = extract_i32(&map, "power", script_name)?;
                let toughness = extract_i32(&map, "toughness", script_name)?;
                
                validate_non_negative(card, "card", script_name)?;
                
                commands.push(Command::ModifyStats {
                    card: CardId(card as u32),
                    power,
                    toughness,
                });
            }
            "set_stats" => {
                let card = extract_i32(&map, "card", script_name)?;
                let power = extract_i32(&map, "power", script_name)?;
                let toughness = extract_i32(&map, "toughness", script_name)?;
                
                validate_non_negative(card, "card", script_name)?;
                
                commands.push(Command::SetStats {
                    card: CardId(card as u32),
                    power,
                    toughness,
                });
            }
            "modify_stat" => {
                let card = extract_i32(&map, "card", script_name)?;
                let stat_name = extract_string(&map, "stat_name", script_name)?;
                let delta = extract_i32(&map, "delta", script_name)?;
                
                validate_non_negative(card, "card", script_name)?;
                
                commands.push(Command::ModifyStat {
                    card: CardId(card as u32),
                    stat_name,
                    delta,
                });
            }
            "set_stat" => {
                let card = extract_i32(&map, "card", script_name)?;
                let stat_name = extract_string(&map, "stat_name", script_name)?;
                let value = extract_string(&map, "value", script_name)?;
                
                validate_non_negative(card, "card", script_name)?;
                
                commands.push(Command::SetStat {
                    card: CardId(card as u32),
                    stat_name,
                    value,
                });
            }
            "grant_keyword" => {
                let card = extract_i32(&map, "card", script_name)?;
                let keyword = extract_string(&map, "keyword", script_name)?;
                
                validate_non_negative(card, "card", script_name)?;
                
                commands.push(Command::GrantKeyword {
                    card: CardId(card as u32),
                    keyword,
                });
            }
            "remove_keyword" => {
                let card = extract_i32(&map, "card", script_name)?;
                let keyword = extract_string(&map, "keyword", script_name)?;
                
                validate_non_negative(card, "card", script_name)?;
                
                commands.push(Command::RemoveKeyword {
                    card: CardId(card as u32),
                    keyword,
                });
            }
            "gain_resource" => {
                let player = extract_i32(&map, "player", script_name)?;
                let resource = extract_string(&map, "resource", script_name)?;
                let amount = extract_i32(&map, "amount", script_name)?;
                
                validate_non_negative(player, "player", script_name)?;
                validate_non_negative(amount, "amount", script_name)?;
                validate_u8_range(player, "player", script_name)?;
                
                commands.push(Command::GainResource {
                    player: PlayerId(player as u8),
                    resource,
                    amount,
                });
            }
            "spend_resource" => {
                let player = extract_i32(&map, "player", script_name)?;
                let resource = extract_string(&map, "resource", script_name)?;
                let amount = extract_i32(&map, "amount", script_name)?;
                
                validate_non_negative(player, "player", script_name)?;
                validate_non_negative(amount, "amount", script_name)?;
                validate_u8_range(player, "player", script_name)?;
                
                commands.push(Command::SpendResource {
                    player: PlayerId(player as u8),
                    resource,
                    amount,
                });
            }
            "set_resource" => {
                let player = extract_i32(&map, "player", script_name)?;
                let resource = extract_string(&map, "resource", script_name)?;
                let amount = extract_i32(&map, "amount", script_name)?;
                
                validate_non_negative(player, "player", script_name)?;
                validate_non_negative(amount, "amount", script_name)?;
                validate_u8_range(player, "player", script_name)?;
                
                commands.push(Command::SetResource {
                    player: PlayerId(player as u8),
                    resource,
                    amount,
                });
            }
            "create_token" => {
                let player = extract_i32(&map, "player", script_name)?;
                let token_type = extract_string(&map, "token_type", script_name)?;
                let zone = extract_string(&map, "zone", script_name)?;
                
                validate_non_negative(player, "player", script_name)?;
                validate_u8_range(player, "player", script_name)?;
                
                let zone_id = string_to_zone_id(&zone);
                
                commands.push(Command::CreateToken {
                    player: PlayerId(player as u8),
                    token_type,
                    zone: zone_id,
                });
            }
            "add_counter" => {
                let card = extract_i32(&map, "card", script_name)?;
                let counter_type = extract_string(&map, "counter_type", script_name)?;
                let amount = extract_i32(&map, "amount", script_name)?;
                
                validate_non_negative(card, "card", script_name)?;
                validate_non_negative(amount, "amount", script_name)?;
                
                commands.push(Command::AddCounter {
                    card: CardId(card as u32),
                    counter_type,
                    amount,
                });
            }
            "remove_counter" => {
                let card = extract_i32(&map, "card", script_name)?;
                let counter_type = extract_string(&map, "counter_type", script_name)?;
                let amount = extract_i32(&map, "amount", script_name)?;
                
                validate_non_negative(card, "card", script_name)?;
                validate_non_negative(amount, "amount", script_name)?;
                
                commands.push(Command::RemoveCounter {
                    card: CardId(card as u32),
                    counter_type,
                    amount,
                });
            }
            _ => {
                return Err(CardinalError(format!(
                    "Script '{}' has unknown effect type: '{}'",
                    script_name, effect_type
                )));
            }
        }
    }
    
    Ok(commands)
}

// Helper functions to extract and validate values from Rhai maps
fn extract_i32(map: &rhai::Map, key: &str, script_name: &str) -> Result<i32, CardinalError> {
    map.get(key)
        .ok_or_else(|| CardinalError(format!(
            "Script '{}' effect missing '{}' field",
            script_name, key
        )))?
        .clone()
        .try_cast::<i32>()
        .ok_or_else(|| CardinalError(format!(
            "Script '{}' effect has non-integer '{}'",
            script_name, key
        )))
}

fn extract_string(map: &rhai::Map, key: &str, script_name: &str) -> Result<String, CardinalError> {
    map.get(key)
        .ok_or_else(|| CardinalError(format!(
            "Script '{}' effect missing '{}' field",
            script_name, key
        )))?
        .clone()
        .try_cast::<String>()
        .ok_or_else(|| CardinalError(format!(
            "Script '{}' effect has non-string '{}'",
            script_name, key
        )))
}

fn validate_non_negative(value: i32, field: &str, script_name: &str) -> Result<(), CardinalError> {
    if value < 0 {
        return Err(CardinalError(format!(
            "Script '{}' effect has negative {}: {}",
            script_name, field, value
        )));
    }
    Ok(())
}

fn validate_positive(value: i32, field: &str, script_name: &str) -> Result<(), CardinalError> {
    if value <= 0 {
        return Err(CardinalError(format!(
            "Script '{}' effect has non-positive {}: {}",
            script_name, field, value
        )));
    }
    Ok(())
}

fn validate_u8_range(value: i32, field: &str, script_name: &str) -> Result<(), CardinalError> {
    if value > u8::MAX as i32 {
        return Err(CardinalError(format!(
            "Script '{}' effect has {} out of range: {}",
            script_name, field, value
        )));
    }
    Ok(())
}

fn string_to_zone_id(zone_str: &str) -> ZoneId {
    // Convert string to static ZoneId by leaking the string
    // Note: This intentionally leaks memory but zone IDs are expected to be
    // a small, finite set (hand, deck, graveyard, field, etc.) in practice.
    // A more robust solution would store zone IDs in GameState/GameEngine
    // or redesign ZoneId to own its String, but this is acceptable for now
    // given the limited set of zone names used in typical games.
    let boxed = zone_str.to_string().into_boxed_str();
    let static_str: &'static str = Box::leak(boxed);
    ZoneId(static_str)
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
        
        // Validate amount is non-negative to prevent healing via damage
        if amount < 0 {
            return Err(CardinalError(format!(
                "Builtin damage effect has negative amount: {} (effect: {})",
                amount, effect_str
            )));
        }
        
        // TODO: Add proper target selection
        // For now, damage affects the controller as a placeholder
        // Future: request target via PendingChoice, then apply to selected target
        Ok(vec![Command::ChangeLife {
            player: controller,
            delta: -amount,
        }])
    } else if effect_str.starts_with("draw_") {
        let count = effect_str.strip_prefix("draw_")
            .and_then(|s| s.parse::<u32>().ok())
            .ok_or_else(|| CardinalError(format!("Invalid draw count in: {}", effect_str)))?;
        
        // Validate count is reasonable (prevent excessive draws)
        if count == 0 {
            return Err(CardinalError(format!(
                "Builtin draw effect has zero count (effect: {})",
                effect_str
            )));
        }
        
        // TODO: Implement card drawing
        // For now, return empty (no MoveCard commands yet)
        Ok(vec![])
    } else if effect_str.starts_with("gain_life_") {
        let amount = effect_str.strip_prefix("gain_life_")
            .and_then(|s| s.parse::<i32>().ok())
            .ok_or_else(|| CardinalError(format!("Invalid life amount in: {}", effect_str)))?;
        
        // Validate amount is non-negative to prevent damage via life gain
        if amount < 0 {
            return Err(CardinalError(format!(
                "Builtin gain_life effect has negative amount: {} (effect: {})",
                amount, effect_str
            )));
        }
        
        Ok(vec![Command::ChangeLife {
            player: controller,
            delta: amount,
        }])
    } else if effect_str.starts_with("lose_life_") {
        // Format: lose_life_{amount}_player_{player_id}
        let parts: Vec<&str> = effect_str.strip_prefix("lose_life_")
            .unwrap_or("")
            .split("_player_")
            .collect();
        
        let amount = parts.get(0)
            .and_then(|s| s.parse::<i32>().ok())
            .ok_or_else(|| CardinalError(format!("Invalid life amount in: {}", effect_str)))?;
        let player = parts.get(1)
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(controller.0);
        
        if amount < 0 {
            return Err(CardinalError(format!(
                "Builtin lose_life effect has negative amount: {} (effect: {})",
                amount, effect_str
            )));
        }
        
        Ok(vec![Command::ChangeLife {
            player: PlayerId(player),
            delta: -amount,
        }])
    } else if effect_str.starts_with("set_life_") {
        // Format: set_life_{amount}_player_{player_id}
        let parts: Vec<&str> = effect_str.strip_prefix("set_life_")
            .unwrap_or("")
            .split("_player_")
            .collect();
        
        let amount = parts.get(0)
            .and_then(|s| s.parse::<i32>().ok())
            .ok_or_else(|| CardinalError(format!("Invalid life amount in: {}", effect_str)))?;
        let player = parts.get(1)
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(controller.0);
        
        if amount < 0 {
            return Err(CardinalError(format!(
                "Builtin set_life effect has negative amount: {} (effect: {})",
                amount, effect_str
            )));
        }
        
        Ok(vec![Command::SetLife {
            player: PlayerId(player),
            amount,
        }])
    } else if effect_str.starts_with("mill_") {
        // Format: mill_{count}_player_{player_id}
        let parts: Vec<&str> = effect_str.strip_prefix("mill_")
            .unwrap_or("")
            .split("_player_")
            .collect();
        
        let _count = parts.get(0)
            .and_then(|s| s.parse::<i32>().ok())
            .ok_or_else(|| CardinalError(format!("Invalid mill count in: {}", effect_str)))?;
        let _player = parts.get(1)
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(controller.0);
        
        // Placeholder: milling (deck to graveyard) is not implemented yet for builtin effects.
        // Fail explicitly so game designers are not misled by a silent no-op.
        Err(CardinalError(format!(
            "Builtin effect '{}' is not implemented yet (milling is currently unsupported)",
            effect_str
        )))
    } else if effect_str.starts_with("discard_") {
        // Format: discard_{count}_player_{player_id}
        let parts: Vec<&str> = effect_str.strip_prefix("discard_")
            .unwrap_or("")
            .split("_player_")
            .collect();
        
        let _count = parts.get(0)
            .and_then(|s| s.parse::<i32>().ok())
            .ok_or_else(|| CardinalError(format!("Invalid discard count in: {}", effect_str)))?;
        let _player = parts.get(1)
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(controller.0);
        
        // Placeholder: discarding (hand to graveyard) is not implemented yet for builtin effects.
        // Fail explicitly so game designers are not misled by a silent no-op.
        Err(CardinalError(format!(
            "Builtin effect '{}' is not implemented yet (discarding is currently unsupported)",
            effect_str
        )))
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
        
        // Note: pump can have negative values to reduce stats, so no validation here
        
        // Placeholder: creature stat modification is not implemented yet for builtin effects.
        // Fail explicitly so game designers are not misled by a silent no-op.
        Err(CardinalError(format!(
            "Builtin effect '{}' is not implemented yet (pump is currently unsupported)",
            effect_str
        )))
    } else if effect_str.starts_with("set_stats_") {
        // Format: set_stats_{card_id}_{power}_{toughness}
        let parts: Vec<&str> = effect_str.strip_prefix("set_stats_")
            .unwrap_or("")
            .split('_')
            .collect();
        
        let card = parts.get(0)
            .and_then(|s| s.parse::<u32>().ok())
            .ok_or_else(|| CardinalError(format!("Invalid card id in: {}", effect_str)))?;
        let power = parts.get(1)
            .and_then(|s| s.parse::<i32>().ok())
            .ok_or_else(|| CardinalError(format!("Invalid power in: {}", effect_str)))?;
        let toughness = parts.get(2)
            .and_then(|s| s.parse::<i32>().ok())
            .ok_or_else(|| CardinalError(format!("Invalid toughness in: {}", effect_str)))?;
        
        Ok(vec![Command::SetStats {
            card: CardId(card),
            power,
            toughness,
        }])
    } else if effect_str.starts_with("grant_keyword_") {
        // Format: grant_keyword_{card_id}_{keyword}
        let parts: Vec<&str> = effect_str.strip_prefix("grant_keyword_")
            .unwrap_or("")
            .splitn(2, '_')
            .collect();
        
        let card = parts.get(0)
            .and_then(|s| s.parse::<u32>().ok())
            .ok_or_else(|| CardinalError(format!("Invalid card id in: {}", effect_str)))?;
        let keyword = parts.get(1)
            .ok_or_else(|| CardinalError(format!("Missing keyword in: {}", effect_str)))?
            .to_string();
        
        Ok(vec![Command::GrantKeyword {
            card: CardId(card),
            keyword,
        }])
    } else if effect_str.starts_with("remove_keyword_") {
        // Format: remove_keyword_{card_id}_{keyword}
        let parts: Vec<&str> = effect_str.strip_prefix("remove_keyword_")
            .unwrap_or("")
            .splitn(2, '_')
            .collect();
        
        let card = parts.get(0)
            .and_then(|s| s.parse::<u32>().ok())
            .ok_or_else(|| CardinalError(format!("Invalid card id in: {}", effect_str)))?;
        let keyword = parts.get(1)
            .ok_or_else(|| CardinalError(format!("Missing keyword in: {}", effect_str)))?
            .to_string();
        
        Ok(vec![Command::RemoveKeyword {
            card: CardId(card),
            keyword,
        }])
    } else if effect_str.starts_with("gain_resource_") {
        // Format: gain_resource_{player_id}_{resource_name}_{amount}
        let parts: Vec<&str> = effect_str.strip_prefix("gain_resource_")
            .unwrap_or("")
            .splitn(3, '_')
            .collect();
        
        let player = parts.get(0)
            .and_then(|s| s.parse::<u8>().ok())
            .ok_or_else(|| CardinalError(format!("Invalid player id in: {}", effect_str)))?;
        let resource = parts.get(1)
            .ok_or_else(|| CardinalError(format!("Missing resource name in: {}", effect_str)))?
            .to_string();
        let amount = parts.get(2)
            .and_then(|s| s.parse::<i32>().ok())
            .ok_or_else(|| CardinalError(format!("Invalid amount in: {}", effect_str)))?;
        
        if amount < 0 {
            return Err(CardinalError(format!(
                "Builtin gain_resource effect has negative amount: {} (effect: {})",
                amount, effect_str
            )));
        }
        
        Ok(vec![Command::GainResource {
            player: PlayerId(player),
            resource,
            amount,
        }])
    } else if effect_str.starts_with("spend_resource_") {
        // Format: spend_resource_{player_id}_{resource_name}_{amount}
        let parts: Vec<&str> = effect_str.strip_prefix("spend_resource_")
            .unwrap_or("")
            .splitn(3, '_')
            .collect();
        
        let player = parts.get(0)
            .and_then(|s| s.parse::<u8>().ok())
            .ok_or_else(|| CardinalError(format!("Invalid player id in: {}", effect_str)))?;
        let resource = parts.get(1)
            .ok_or_else(|| CardinalError(format!("Missing resource name in: {}", effect_str)))?
            .to_string();
        let amount = parts.get(2)
            .and_then(|s| s.parse::<i32>().ok())
            .ok_or_else(|| CardinalError(format!("Invalid amount in: {}", effect_str)))?;
        
        if amount < 0 {
            return Err(CardinalError(format!(
                "Builtin spend_resource effect has negative amount: {} (effect: {})",
                amount, effect_str
            )));
        }
        
        Ok(vec![Command::SpendResource {
            player: PlayerId(player),
            resource,
            amount,
        }])
    } else if effect_str.starts_with("set_resource_") {
        // Format: set_resource_{player_id}_{resource_name}_{amount}
        let parts: Vec<&str> = effect_str.strip_prefix("set_resource_")
            .unwrap_or("")
            .splitn(3, '_')
            .collect();
        
        let player = parts.get(0)
            .and_then(|s| s.parse::<u8>().ok())
            .ok_or_else(|| CardinalError(format!("Invalid player id in: {}", effect_str)))?;
        let resource = parts.get(1)
            .ok_or_else(|| CardinalError(format!("Missing resource name in: {}", effect_str)))?
            .to_string();
        let amount = parts.get(2)
            .and_then(|s| s.parse::<i32>().ok())
            .ok_or_else(|| CardinalError(format!("Invalid amount in: {}", effect_str)))?;
        
        if amount < 0 {
            return Err(CardinalError(format!(
                "Builtin set_resource effect has negative amount: {} (effect: {})",
                amount, effect_str
            )));
        }
        
        Ok(vec![Command::SetResource {
            player: PlayerId(player),
            resource,
            amount,
        }])
    } else if effect_str.starts_with("add_counter_") {
        // Format: add_counter_{card_id}_{counter_type}_{amount}
        let parts: Vec<&str> = effect_str.strip_prefix("add_counter_")
            .unwrap_or("")
            .splitn(3, '_')
            .collect();
        
        let card = parts.get(0)
            .and_then(|s| s.parse::<u32>().ok())
            .ok_or_else(|| CardinalError(format!("Invalid card id in: {}", effect_str)))?;
        let counter_type = parts.get(1)
            .ok_or_else(|| CardinalError(format!("Missing counter type in: {}", effect_str)))?
            .to_string();
        let amount = parts.get(2)
            .and_then(|s| s.parse::<i32>().ok())
            .ok_or_else(|| CardinalError(format!("Invalid amount in: {}", effect_str)))?;
        
        if amount < 0 {
            return Err(CardinalError(format!(
                "Builtin add_counter effect has negative amount: {} (effect: {})",
                amount, effect_str
            )));
        }
        
        Ok(vec![Command::AddCounter {
            card: CardId(card),
            counter_type,
            amount,
        }])
    } else if effect_str.starts_with("remove_counter_") {
        // Format: remove_counter_{card_id}_{counter_type}_{amount}
        let parts: Vec<&str> = effect_str.strip_prefix("remove_counter_")
            .unwrap_or("")
            .splitn(3, '_')
            .collect();
        
        let card = parts.get(0)
            .and_then(|s| s.parse::<u32>().ok())
            .ok_or_else(|| CardinalError(format!("Invalid card id in: {}", effect_str)))?;
        let counter_type = parts.get(1)
            .ok_or_else(|| CardinalError(format!("Missing counter type in: {}", effect_str)))?
            .to_string();
        let amount = parts.get(2)
            .and_then(|s| s.parse::<i32>().ok())
            .ok_or_else(|| CardinalError(format!("Invalid amount in: {}", effect_str)))?;
        
        if amount < 0 {
            return Err(CardinalError(format!(
                "Builtin remove_counter effect has negative amount: {} (effect: {})",
                amount, effect_str
            )));
        }
        
        Ok(vec![Command::RemoveCounter {
            card: CardId(card),
            counter_type,
            amount,
        }])
    } else if effect_str.starts_with("create_token_") {
        // Format: create_token_{player_id}_{token_type}_{zone}
        // Note: token_type can contain underscores (e.g., "1/1_soldier")
        // Strategy: split to get player, then find last underscore for zone
        let rest = effect_str.strip_prefix("create_token_")
            .unwrap_or("");
        
        // Split once to get player
        let mut parts = rest.splitn(2, '_');
        let player = parts.next()
            .and_then(|s| s.parse::<u8>().ok())
            .ok_or_else(|| CardinalError(format!("Invalid player id in: {}", effect_str)))?;
        
        let remainder = parts.next()
            .ok_or_else(|| CardinalError(format!("Missing token type and zone in: {}", effect_str)))?;
        
        // Find the last underscore to split token_type from zone
        let last_underscore = remainder.rfind('_')
            .ok_or_else(|| CardinalError(format!("Missing zone separator in: {}", effect_str)))?;
        
        let token_type = remainder[..last_underscore].to_string();
        let zone_str = &remainder[last_underscore + 1..];
        
        let zone = string_to_zone_id(zone_str);
        
        Ok(vec![Command::CreateToken {
            player: PlayerId(player),
            token_type,
            zone,
        }])
    } else if effect_str.starts_with("move_card_") {
        // Format: move_card_{card_id}_{from_zone}_{to_zone}
        let parts: Vec<&str> = effect_str.strip_prefix("move_card_")
            .unwrap_or("")
            .splitn(3, '_')
            .collect();
        
        let card = parts.get(0)
            .and_then(|s| s.parse::<u32>().ok())
            .ok_or_else(|| CardinalError(format!("Invalid card id in: {}", effect_str)))?;
        let from_zone_str = parts.get(1)
            .ok_or_else(|| CardinalError(format!("Missing from_zone in: {}", effect_str)))?;
        let to_zone_str = parts.get(2)
            .ok_or_else(|| CardinalError(format!("Missing to_zone in: {}", effect_str)))?;
        
        let from_zone = string_to_zone_id(from_zone_str);
        let to_zone = string_to_zone_id(to_zone_str);
        
        Ok(vec![Command::MoveCard {
            card: CardId(card),
            from: from_zone,
            to: to_zone,
        }])
    } else {
        Err(CardinalError(format!("Unknown builtin effect type: {}", effect_str)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::gamestate::{GameState, TurnState, PlayerState};
    use crate::ids::{PhaseId, StepId};
    use std::collections::HashMap;
    
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
                PlayerState { id: PlayerId(0), life: 20, resources: HashMap::new() },
                PlayerState { id: PlayerId(1), life: 20, resources: HashMap::new() },
            ],
            zones: vec![],
            stack: vec![],
            pending_choice: None,
            ended: None,
            card_instances: HashMap::new(),
        }
    }
    
    #[test]
    fn test_execute_damage_effect() {
        let effect = EffectRef::Builtin("damage_2");
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state, None);
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
        
        let result = execute_effect(&effect, None, controller, &state, None);
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
        
        let result = execute_effect(&effect, None, controller, &state, None);
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
        
        let result = execute_effect(&effect, None, controller, &state, None);
        // Pump not yet implemented, should return error
        assert!(result.is_err());
        assert!(result.unwrap_err().0.contains("not implemented yet"));
    }
    
    #[test]
    fn test_invalid_effect_string() {
        let effect = EffectRef::Builtin("invalid");
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state, None);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_invalid_damage_amount() {
        let effect = EffectRef::Builtin("damage_abc");
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state, None);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_execute_scripted_effect() {
        use crate::engine::scripting::RhaiEngine;
        
        let mut engine = RhaiEngine::new();
        let script = r#"
            fn execute_ability() {
                gain_life(0, 3)
            }
        "#;
        
        engine.register_script("test_card".to_string(), script).unwrap();
        
        let effect = EffectRef::Scripted("test_card".to_string());
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state, Some(&engine));
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::ChangeLife { player, delta } => {
                assert_eq!(*player, PlayerId(0));
                assert_eq!(*delta, 3);
            }
            _ => panic!("Expected ChangeLife command"),
        }
    }
    
    #[test]
    fn test_execute_scripted_damage_effect() {
        use crate::engine::scripting::RhaiEngine;
        
        let mut engine = RhaiEngine::new();
        let script = r#"
            fn execute_ability() {
                deal_damage(1, 5)
            }
        "#;
        
        engine.register_script("bolt_card".to_string(), script).unwrap();
        
        let effect = EffectRef::Scripted("bolt_card".to_string());
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state, Some(&engine));
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::ChangeLife { player, delta } => {
                assert_eq!(*player, PlayerId(1));
                assert_eq!(*delta, -5);
            }
            _ => panic!("Expected ChangeLife command"),
        }
    }
    
    #[test]
    fn test_scripted_lose_life() {
        use crate::engine::scripting::RhaiEngine;
        
        let mut engine = RhaiEngine::new();
        let script = r#"
            fn execute_ability() {
                lose_life(0, 3)
            }
        "#;
        
        engine.register_script("lose_life_card".to_string(), script).unwrap();
        
        let effect = EffectRef::Scripted("lose_life_card".to_string());
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state, Some(&engine));
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::ChangeLife { player, delta } => {
                assert_eq!(*player, PlayerId(0));
                assert_eq!(*delta, -3);
            }
            _ => panic!("Expected ChangeLife command"),
        }
    }
    
    #[test]
    fn test_scripted_set_life() {
        use crate::engine::scripting::RhaiEngine;
        
        let mut engine = RhaiEngine::new();
        let script = r#"
            fn execute_ability() {
                set_life(1, 10)
            }
        "#;
        
        engine.register_script("set_life_card".to_string(), script).unwrap();
        
        let effect = EffectRef::Scripted("set_life_card".to_string());
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state, Some(&engine));
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::SetLife { player, amount } => {
                assert_eq!(*player, PlayerId(1));
                assert_eq!(*amount, 10);
            }
            _ => panic!("Expected SetLife command"),
        }
    }
    
    #[test]
    fn test_scripted_pump() {
        use crate::engine::scripting::RhaiEngine;
        
        let mut engine = RhaiEngine::new();
        let script = r#"
            fn execute_ability() {
                pump_creature(source_card, 2, 2)
            }
        "#;
        
        engine.register_script("pump_card".to_string(), script).unwrap();
        
        let effect = EffectRef::Scripted("pump_card".to_string());
        let controller = PlayerId(0);
        let source = Some(CardId(5));
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, source, controller, &state, Some(&engine));
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::ModifyStats { card, power, toughness } => {
                assert_eq!(card.0, 5);
                assert_eq!(*power, 2);
                assert_eq!(*toughness, 2);
            }
            _ => panic!("Expected ModifyStats command"),
        }
    }
    
    #[test]
    fn test_scripted_set_stats() {
        use crate::engine::scripting::RhaiEngine;
        
        let mut engine = RhaiEngine::new();
        let script = r#"
            fn execute_ability() {
                set_stats(10, 5, 5)
            }
        "#;
        
        engine.register_script("set_stats_card".to_string(), script).unwrap();
        
        let effect = EffectRef::Scripted("set_stats_card".to_string());
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state, Some(&engine));
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::SetStats { card, power, toughness } => {
                assert_eq!(card.0, 10);
                assert_eq!(*power, 5);
                assert_eq!(*toughness, 5);
            }
            _ => panic!("Expected SetStats command"),
        }
    }
    
    #[test]
    fn test_scripted_grant_keyword() {
        use crate::engine::scripting::RhaiEngine;
        
        let mut engine = RhaiEngine::new();
        let script = r#"
            fn execute_ability() {
                grant_keyword(source_card, "flying")
            }
        "#;
        
        engine.register_script("keyword_card".to_string(), script).unwrap();
        
        let effect = EffectRef::Scripted("keyword_card".to_string());
        let controller = PlayerId(0);
        let source = Some(CardId(7));
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, source, controller, &state, Some(&engine));
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::GrantKeyword { card, keyword } => {
                assert_eq!(card.0, 7);
                assert_eq!(keyword, "flying");
            }
            _ => panic!("Expected GrantKeyword command"),
        }
    }
    
    #[test]
    fn test_scripted_remove_keyword() {
        use crate::engine::scripting::RhaiEngine;
        
        let mut engine = RhaiEngine::new();
        let script = r#"
            fn execute_ability() {
                remove_keyword(12, "quick")
            }
        "#;
        
        engine.register_script("remove_kw_card".to_string(), script).unwrap();
        
        let effect = EffectRef::Scripted("remove_kw_card".to_string());
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state, Some(&engine));
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::RemoveKeyword { card, keyword } => {
                assert_eq!(card.0, 12);
                assert_eq!(keyword, "quick");
            }
            _ => panic!("Expected RemoveKeyword command"),
        }
    }
    
    #[test]
    fn test_scripted_gain_resource() {
        use crate::engine::scripting::RhaiEngine;
        
        let mut engine = RhaiEngine::new();
        let script = r#"
            fn execute_ability() {
                gain_resource(controller, "mana", 5)
            }
        "#;
        
        engine.register_script("mana_card".to_string(), script).unwrap();
        
        let effect = EffectRef::Scripted("mana_card".to_string());
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state, Some(&engine));
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::GainResource { player, resource, amount } => {
                assert_eq!(*player, PlayerId(0));
                assert_eq!(resource, "mana");
                assert_eq!(*amount, 5);
            }
            _ => panic!("Expected GainResource command"),
        }
    }
    
    #[test]
    fn test_scripted_spend_resource() {
        use crate::engine::scripting::RhaiEngine;
        
        let mut engine = RhaiEngine::new();
        let script = r#"
            fn execute_ability() {
                spend_resource(controller, "mana", 3)
            }
        "#;
        
        engine.register_script("spend_card".to_string(), script).unwrap();
        
        let effect = EffectRef::Scripted("spend_card".to_string());
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state, Some(&engine));
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::SpendResource { player, resource, amount } => {
                assert_eq!(*player, PlayerId(0));
                assert_eq!(resource, "mana");
                assert_eq!(*amount, 3);
            }
            _ => panic!("Expected SpendResource command"),
        }
    }
    
    #[test]
    fn test_scripted_set_resource() {
        use crate::engine::scripting::RhaiEngine;
        
        let mut engine = RhaiEngine::new();
        let script = r#"
            fn execute_ability() {
                set_resource(controller, "energy", 10)
            }
        "#;
        
        engine.register_script("set_res_card".to_string(), script).unwrap();
        
        let effect = EffectRef::Scripted("set_res_card".to_string());
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state, Some(&engine));
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::SetResource { player, resource, amount } => {
                assert_eq!(*player, PlayerId(0));
                assert_eq!(resource, "energy");
                assert_eq!(*amount, 10);
            }
            _ => panic!("Expected SetResource command"),
        }
    }
    
    #[test]
    fn test_scripted_create_token() {
        use crate::engine::scripting::RhaiEngine;
        
        let mut engine = RhaiEngine::new();
        let script = r#"
            fn execute_ability() {
                create_token(controller, "goblin", "field")
            }
        "#;
        
        engine.register_script("token_card".to_string(), script).unwrap();
        
        let effect = EffectRef::Scripted("token_card".to_string());
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state, Some(&engine));
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::CreateToken { player, token_type, zone: _ } => {
                assert_eq!(*player, PlayerId(0));
                assert_eq!(token_type, "goblin");
            }
            _ => panic!("Expected CreateToken command"),
        }
    }
    
    #[test]
    fn test_scripted_add_counter() {
        use crate::engine::scripting::RhaiEngine;
        
        let mut engine = RhaiEngine::new();
        let script = r#"
            fn execute_ability() {
                add_counter(source_card, "+1/+1", 2)
            }
        "#;
        
        engine.register_script("counter_card".to_string(), script).unwrap();
        
        let effect = EffectRef::Scripted("counter_card".to_string());
        let controller = PlayerId(0);
        let source = Some(CardId(15));
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, source, controller, &state, Some(&engine));
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::AddCounter { card, counter_type, amount } => {
                assert_eq!(card.0, 15);
                assert_eq!(counter_type, "+1/+1");
                assert_eq!(*amount, 2);
            }
            _ => panic!("Expected AddCounter command"),
        }
    }
    
    #[test]
    fn test_scripted_remove_counter() {
        use crate::engine::scripting::RhaiEngine;
        
        let mut engine = RhaiEngine::new();
        let script = r#"
            fn execute_ability() {
                remove_counter(source_card, "charge", 1)
            }
        "#;
        
        engine.register_script("remove_counter_card".to_string(), script).unwrap();
        
        let effect = EffectRef::Scripted("remove_counter_card".to_string());
        let controller = PlayerId(0);
        let source = Some(CardId(18));
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, source, controller, &state, Some(&engine));
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::RemoveCounter { card, counter_type, amount } => {
                assert_eq!(card.0, 18);
                assert_eq!(counter_type, "charge");
                assert_eq!(*amount, 1);
            }
            _ => panic!("Expected RemoveCounter command"),
        }
    }
    
    #[test]
    fn test_scripted_modify_stat() {
        use crate::engine::scripting::RhaiEngine;
        
        let mut engine = RhaiEngine::new();
        let script = r#"
            fn execute_ability() {
                modify_stat(source_card, "range", 1)
            }
        "#;
        
        engine.register_script("range_card".to_string(), script).unwrap();
        
        let effect = EffectRef::Scripted("range_card".to_string());
        let controller = PlayerId(0);
        let source = Some(CardId(20));
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, source, controller, &state, Some(&engine));
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::ModifyStat { card, stat_name, delta } => {
                assert_eq!(card.0, 20);
                assert_eq!(stat_name, "range");
                assert_eq!(*delta, 1);
            }
            _ => panic!("Expected ModifyStat command"),
        }
    }
    
    #[test]
    fn test_scripted_set_stat() {
        use crate::engine::scripting::RhaiEngine;
        
        let mut engine = RhaiEngine::new();
        let script = r#"
            fn execute_ability() {
                set_stat(source_card, "element", "fire")
            }
        "#;
        
        engine.register_script("element_card".to_string(), script).unwrap();
        
        let effect = EffectRef::Scripted("element_card".to_string());
        let controller = PlayerId(0);
        let source = Some(CardId(25));
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, source, controller, &state, Some(&engine));
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::SetStat { card, stat_name, value } => {
                assert_eq!(card.0, 25);
                assert_eq!(stat_name, "element");
                assert_eq!(value, "fire");
            }
            _ => panic!("Expected SetStat command"),
        }
    }
    
    #[test]
    fn test_scripted_move_card() {
        use crate::engine::scripting::RhaiEngine;
        
        let mut engine = RhaiEngine::new();
        let script = r#"
            fn execute_ability() {
                move_card(10, "hand", "graveyard")
            }
        "#;
        
        engine.register_script("move_card".to_string(), script).unwrap();
        
        let effect = EffectRef::Scripted("move_card".to_string());
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state, Some(&engine));
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::MoveCard { card, from: _, to: _ } => {
                assert_eq!(card.0, 10);
            }
            _ => panic!("Expected MoveCard command"),
        }
    }
    
    #[test]
    fn test_scripted_shuffle_zone() {
        use crate::engine::scripting::RhaiEngine;
        
        let mut engine = RhaiEngine::new();
        let script = r#"
            fn execute_ability() {
                shuffle_zone(controller, "deck")
            }
        "#;
        
        engine.register_script("shuffle_card".to_string(), script).unwrap();
        
        let effect = EffectRef::Scripted("shuffle_card".to_string());
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        // shuffle_zone is not yet implemented, so it should return an error
        let result = execute_effect(&effect, None, controller, &state, Some(&engine));
        assert!(result.is_err());
        assert!(result.unwrap_err().0.contains("not yet implemented"));
    }
    
    #[test]
    fn test_scripted_multi_effect() {
        use crate::engine::scripting::RhaiEngine;
        
        let mut engine = RhaiEngine::new();
        let script = r#"
            fn execute_ability() {
                [
                    deal_damage(1, 3),
                    gain_life(controller, 3),
                    grant_keyword(source_card, "flying")
                ]
            }
        "#;
        
        engine.register_script("multi_card".to_string(), script).unwrap();
        
        let effect = EffectRef::Scripted("multi_card".to_string());
        let controller = PlayerId(0);
        let source = Some(CardId(30));
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, source, controller, &state, Some(&engine));
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 3);
    }
    
    // ===================================================================
    // TESTS FOR NEW BUILTIN EFFECTS
    // ===================================================================
    
    #[test]
    fn test_builtin_lose_life() {
        let effect = EffectRef::Builtin("lose_life_3_player_0");
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state, None);
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::ChangeLife { player, delta } => {
                assert_eq!(*player, PlayerId(0));
                assert_eq!(*delta, -3);
            }
            _ => panic!("Expected ChangeLife command"),
        }
    }
    
    #[test]
    fn test_builtin_set_life() {
        let effect = EffectRef::Builtin("set_life_20_player_1");
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state, None);
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::SetLife { player, amount } => {
                assert_eq!(*player, PlayerId(1));
                assert_eq!(*amount, 20);
            }
            _ => panic!("Expected SetLife command"),
        }
    }
    
    #[test]
    fn test_builtin_set_stats() {
        let effect = EffectRef::Builtin("set_stats_5_3_4");
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state, None);
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::SetStats { card, power, toughness } => {
                assert_eq!(*card, CardId(5));
                assert_eq!(*power, 3);
                assert_eq!(*toughness, 4);
            }
            _ => panic!("Expected SetStats command"),
        }
    }
    
    #[test]
    fn test_builtin_grant_keyword() {
        let effect = EffectRef::Builtin("grant_keyword_10_flying");
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state, None);
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::GrantKeyword { card, keyword } => {
                assert_eq!(*card, CardId(10));
                assert_eq!(keyword, "flying");
            }
            _ => panic!("Expected GrantKeyword command"),
        }
    }
    
    #[test]
    fn test_builtin_remove_keyword() {
        let effect = EffectRef::Builtin("remove_keyword_10_haste");
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state, None);
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::RemoveKeyword { card, keyword } => {
                assert_eq!(*card, CardId(10));
                assert_eq!(keyword, "haste");
            }
            _ => panic!("Expected RemoveKeyword command"),
        }
    }
    
    #[test]
    fn test_builtin_gain_resource() {
        let effect = EffectRef::Builtin("gain_resource_0_mana_3");
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state, None);
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::GainResource { player, resource, amount } => {
                assert_eq!(*player, PlayerId(0));
                assert_eq!(resource, "mana");
                assert_eq!(*amount, 3);
            }
            _ => panic!("Expected GainResource command"),
        }
    }
    
    #[test]
    fn test_builtin_spend_resource() {
        let effect = EffectRef::Builtin("spend_resource_1_mana_2");
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state, None);
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::SpendResource { player, resource, amount } => {
                assert_eq!(*player, PlayerId(1));
                assert_eq!(resource, "mana");
                assert_eq!(*amount, 2);
            }
            _ => panic!("Expected SpendResource command"),
        }
    }
    
    #[test]
    fn test_builtin_set_resource() {
        let effect = EffectRef::Builtin("set_resource_0_energy_10");
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state, None);
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::SetResource { player, resource, amount } => {
                assert_eq!(*player, PlayerId(0));
                assert_eq!(resource, "energy");
                assert_eq!(*amount, 10);
            }
            _ => panic!("Expected SetResource command"),
        }
    }
    
    #[test]
    fn test_builtin_add_counter() {
        let effect = EffectRef::Builtin("add_counter_7_+1/+1_2");
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state, None);
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::AddCounter { card, counter_type, amount } => {
                assert_eq!(*card, CardId(7));
                assert_eq!(counter_type, "+1/+1");
                assert_eq!(*amount, 2);
            }
            _ => panic!("Expected AddCounter command"),
        }
    }
    
    #[test]
    fn test_builtin_remove_counter() {
        let effect = EffectRef::Builtin("remove_counter_7_charge_1");
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state, None);
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::RemoveCounter { card, counter_type, amount } => {
                assert_eq!(*card, CardId(7));
                assert_eq!(counter_type, "charge");
                assert_eq!(*amount, 1);
            }
            _ => panic!("Expected RemoveCounter command"),
        }
    }
    
    #[test]
    fn test_builtin_create_token() {
        let effect = EffectRef::Builtin("create_token_0_1/1_soldier_field");
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state, None);
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::CreateToken { player, token_type, zone: _ } => {
                assert_eq!(*player, PlayerId(0));
                assert_eq!(token_type, "1/1_soldier");
            }
            _ => panic!("Expected CreateToken command"),
        }
    }
    
    #[test]
    fn test_builtin_move_card() {
        let effect = EffectRef::Builtin("move_card_15_graveyard_hand");
        let controller = PlayerId(0);
        let state = minimal_game_state();
        
        let result = execute_effect(&effect, None, controller, &state, None);
        assert!(result.is_ok());
        
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            Command::MoveCard { card, from, to } => {
                assert_eq!(*card, CardId(15));
                assert_eq!(*from, ZoneId("graveyard"));
                assert_eq!(*to, ZoneId("hand"));
            }
            _ => panic!("Expected MoveCard command"),
        }
    }
}
