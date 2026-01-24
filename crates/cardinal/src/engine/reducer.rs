use crate::{
    engine::core::GameEngine,
    ids::PlayerId,
    model::action::Action,
    model::event::Event,
    model::command::Command,
    error::CardinalError,
};

pub fn apply(engine: &mut GameEngine, player: PlayerId, action: Action) -> Result<Vec<Event>, CardinalError> {
    match action {
        Action::PassPriority => {
            // Only the priority player can pass priority
            if player != engine.state.turn.priority_player {
                return Err(CardinalError("Only the priority player can pass priority".to_string()));
            }
            
            // Track this player's pass
            engine.state.turn.priority_passes += 1;
            
            // Check if all players have passed (priority_passes == num_players means full round)
            let num_players = engine.state.players.len() as u32;
            let all_passed = engine.state.turn.priority_passes >= num_players;
            
            // Rotate priority to next player if not all have passed
            if !all_passed {
                let next_priority_idx = (player.0 + 1) % num_players as u8;
                engine.state.turn.priority_player = crate::ids::PlayerId(next_priority_idx);
            }
            
            Ok(vec![Event::PriorityPassed { by: player }])
        }
        Action::Concede => {
            // Handle concede - determine winner as the other player (or None if no valid winner)
            let winner = engine.state.players.iter()
                .find(|p| p.id != player)
                .map(|p| p.id);
            
            // Mark game as ended
            engine.state.ended = Some(crate::state::gamestate::GameEnd {
                winner,
                reason: format!("Player {:?} conceded", player),
            });
            
            Ok(vec![Event::GameEnded { 
                winner, 
                reason: format!("Player {:?} conceded", player),
            }])
        }
        Action::PlayCard { card, from } => {
            // Look up the play_card action definition to find target zone
            let action_def = engine.rules.actions.iter()
                .find(|a| a.id == "play_card")
                .ok_or_else(|| CardinalError("play_card action not defined in rules".to_string()))?;
            
            let target_zone_str = action_def.target_zone.as_ref()
                .ok_or_else(|| CardinalError("play_card action has no target_zone defined".to_string()))?;
            
            // Construct the target zone ID (if it's player-owned, append player index)
            let target_zone_id = if let Some(zone_def) = engine.rules.zones.iter()
                .find(|z| z.id == *target_zone_str)
            {
                match zone_def.owner_scope {
                    crate::rules::schema::ZoneOwnerScope::Player => {
                        format!("{}@{}", target_zone_str, player.0)
                    }
                    crate::rules::schema::ZoneOwnerScope::Shared => {
                        target_zone_str.clone()
                    }
                }
            } else {
                return Err(CardinalError(format!("target zone '{}' not found in rules", target_zone_str)));
            };
            
            let target_zone_box: Box<str> = target_zone_id.into_boxed_str();
            let target_zone = crate::ids::ZoneId(Box::leak(target_zone_box));
            
            // Generate commands to move the card
            let commands = vec![
                Command::MoveCard { card, from, to: target_zone },
            ];
            
            // Commit commands to state and collect events
            let mut events = crate::engine::events::commit_commands(&mut engine.state, &commands);
            
            // Add the CardPlayed event
            events.push(Event::CardPlayed { player, card });
            
            Ok(events)
        }
        Action::ChooseTarget { choice_id: _, target: _ } => {
            // Clear the pending choice and emit appropriate event
            // For now, just remove the choice without applying effects
            // (effect handling will be part of the trigger system)
            engine.state.pending_choice = None;
            
            // In a full implementation, this would:
            // 1. Validate the target against the choice's allowed targets
            // 2. Generate commands based on the effect
            // 3. Apply those commands
            
            Ok(vec![])
        }
    }
}
