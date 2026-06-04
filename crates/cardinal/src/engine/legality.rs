use crate::{
    engine::core::GameEngine,
    error::CardinalError,
    ids::PlayerId,
    model::action::Action,
};

/// Validate that an action is legal in the current game state.
/// Checks:
/// - Only the priority player can pass priority
/// - Only the active player can take other actions
/// - The current phase allows actions
/// - Stack requirements are met (if action requires empty stack)
/// - Zone ownership and card ownership are valid
pub fn validate(engine: &GameEngine, player: PlayerId, action: &Action) -> Result<(), CardinalError> {
    if engine.state.ended.is_some() {
        return Err(CardinalError("Game has ended".to_string()));
    }

    match action {
        Action::PassPriority => {
            if player != engine.state.turn.priority_player {
                return Err(CardinalError(format!(
                    "Only priority player ({:?}) can pass priority",
                    engine.state.turn.priority_player
                )));
            }
            Ok(())
        }
        Action::Concede => Ok(()),
        Action::PlayCard { card, from } => {
            if player != engine.state.turn.active_player {
                return Err(CardinalError(format!(
                    "Only active player ({:?}) can take this action",
                    engine.state.turn.active_player
                )));
            }

            let current_phase = engine.rules.turn.phases.iter()
                .find(|phase| phase.id.as_str() == engine.state.turn.phase.0)
                .ok_or_else(|| CardinalError("Invalid phase".to_string()))?;

            if !current_phase.allow_actions {
                return Err(CardinalError(format!(
                    "Current phase '{}' does not allow card plays",
                    current_phase.name
                )));
            }

            let zone = engine.state.zones.iter()
                .find(|zone| zone.id == *from)
                .ok_or_else(|| CardinalError("Source zone does not exist".to_string()))?;

            if let Some(owner) = zone.owner {
                if owner != player {
                    return Err(CardinalError("Cannot play cards from opponent's zones".to_string()));
                }
            }

            if !zone.cards.contains(card) {
                return Err(CardinalError("Card is not in the specified source zone".to_string()));
            }

            if let Some(action_def) = engine.rules.actions.iter().find(|action| action.id == "play_card") {
                if action_def.requires_empty_stack && !engine.state.stack.is_empty() {
                    return Err(CardinalError(
                        "Cannot play card: stack is not empty and action requires empty stack".to_string(),
                    ));
                }
            }

            Ok(())
        }
        Action::ChooseTarget { choice_id, target: _ } => match &engine.state.pending_choice {
            Some(choice) if choice.id == *choice_id => Ok(()),
            Some(choice) => Err(CardinalError(format!(
                "Choice ID mismatch: expected {}, got {}",
                choice.id, choice_id
            ))),
            None => Err(CardinalError("No pending choice to respond to".to_string())),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::CardId;
    use crate::model::command::{EffectRef, StackItem};

    fn load_rules() -> crate::rules::schema::Ruleset {
        crate::load_game_config("../../rules.toml", None).expect("load rules")
    }

    fn build_demo_decks(rules: &crate::rules::schema::Ruleset, deck_size: usize) -> Vec<Vec<CardId>> {
        let ids: Vec<CardId> = rules.cards.iter()
            .filter_map(|card| card.id.parse::<u32>().ok().map(CardId))
            .collect();
        assert!(!ids.is_empty());

        (0..rules.players.min_players)
            .map(|_| {
                (0..deck_size)
                    .map(|index| ids[index % ids.len()])
                    .collect()
            })
            .collect()
    }

    fn advance_to_play_card_window(engine: &mut GameEngine) -> (PlayerId, crate::ids::ZoneId, CardId) {
        for _ in 0..64 {
            let active_player = engine.state.turn.active_player;
            let hand_zone = engine.state.zones.iter()
                .find(|zone| zone.id.0 == format!("hand@{}", active_player.0))
                .expect("hand zone");

            if let Some(card) = hand_zone.cards.first().copied() {
                let action = Action::PlayCard { card, from: hand_zone.id.clone() };
                if validate(engine, active_player, &action).is_ok() {
                    return (active_player, hand_zone.id.clone(), card);
                }
            }

            let player = engine.state.turn.priority_player;
            engine.apply_action(player, Action::PassPriority).expect("advance window");
        }

        panic!("no playable hand found");
    }

    #[test]
    fn play_card_is_rejected_in_non_action_phase() {
        let rules = load_rules();
        let mut engine = GameEngine::new(rules.clone(), 42);
        engine.start_game(build_demo_decks(&rules, 10)).expect("start game");

        let active_player = engine.state.turn.active_player;
        let hand_zone = engine.state.zones.iter()
            .find(|zone| zone.id.0 == format!("hand@{}", active_player.0))
            .expect("hand zone");
        let card = hand_zone.cards.first().copied().unwrap_or(CardId(999));

        let result = validate(
            &engine,
            active_player,
            &Action::PlayCard { card, from: hand_zone.id.clone() },
        );

        assert!(result.is_err());
    }

    #[test]
    fn play_card_requires_empty_stack() {
        let rules = load_rules();
        let mut engine = GameEngine::new(rules.clone(), 42);
        engine.start_game(build_demo_decks(&rules, 10)).expect("start game");
        let (active_player, hand_zone, card) = advance_to_play_card_window(&mut engine);

        engine.state.stack.push(StackItem {
            id: 1,
            source: Some(card),
            controller: active_player,
            effect: EffectRef::Builtin("test"),
        });

        let result = validate(
            &engine,
            active_player,
            &Action::PlayCard { card, from: hand_zone },
        );

        assert!(result.is_err());
    }
}
