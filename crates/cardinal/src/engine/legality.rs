use crate::{
    engine::core::GameEngine,
    ids::PlayerId,
    model::action::Action,
    error::CardinalError,
};

/// Validate that an action is legal in the current game state.
/// Checks:
/// - Only the active player can take most actions
/// - The current phase allows actions
/// - Stack requirements are met (if action requires empty stack)
/// - Zone ownership and card ownership are valid
pub fn validate(engine: &GameEngine, player: PlayerId, action: &Action) -> Result<(), CardinalError> {
    // If game has ended, no more actions allowed
    if engine.state.ended.is_some() {
        return Err(CardinalError("Game has ended".to_string()));
    }

    // Only active player can take most actions (PassPriority and Concede are always allowed)
    match action {
        Action::PassPriority | Action::Concede => {
            // Always allowed
        }
        _ => {
            if player != engine.state.turn.active_player {
                return Err(CardinalError(format!(
                    "Only active player ({:?}) can take this action",
                    engine.state.turn.active_player
                )));
            }
        }
    }

    // Check phase permissions
    let current_phase = engine.rules.turn.phases.iter()
        .find(|p| p.id.as_str() == engine.state.turn.phase.0)
        .ok_or_else(|| CardinalError("Invalid phase".to_string()))?;

    match action {
        Action::PassPriority | Action::Concede => Ok(()),
        Action::PlayCard { card, from } => {
            // PlayCard requires the phase to allow actions
            if !current_phase.allow_actions {
                return Err(CardinalError(format!(
                    "Current phase '{}' does not allow card plays",
                    current_phase.name
                )));
            }

            // Verify the source zone exists and is owned by the player
            let zone = engine.state.zones.iter()
                .find(|z| z.id == *from)
                .ok_or_else(|| CardinalError("Source zone does not exist".to_string()))?;

            if let Some(owner) = zone.owner {
                if owner != player {
                    return Err(CardinalError("Cannot play cards from opponent's zones".to_string()));
                }
            }

            // Verify the card exists in the source zone
            if !zone.cards.contains(card) {
                return Err(CardinalError("Card is not in the specified source zone".to_string()));
            }

            // If action requires empty stack, check that stack is empty
            if let Some(action_def) = engine.rules.actions.iter()
                .find(|a| a.id == "play_card")
            {
                if action_def.requires_empty_stack && !engine.state.stack.is_empty() {
                    return Err(CardinalError(
                        "Cannot play card: stack is not empty and action requires empty stack"
                            .to_string(),
                    ));
                }
            }

            Ok(())
        }
        Action::ChooseTarget { choice_id, target: _ } => {
            // ChooseTarget is only valid if there's a pending choice with matching ID
            match &engine.state.pending_choice {
                Some(choice) if choice.id == *choice_id => Ok(()),
                Some(choice) => Err(CardinalError(format!(
                    "Choice ID mismatch: expected {}, got {}",
                    choice.id, choice_id
                ))),
                None => Err(CardinalError("No pending choice to respond to".to_string())),
            }
        }
    }
}
