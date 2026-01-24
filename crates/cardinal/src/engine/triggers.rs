use crate::{
    model::event::Event,
    model::command::{Command, StackItem, EffectRef},
    engine::core::GameEngine,
};

/// Evaluate which triggers should fire in response to an event.
/// Returns commands to execute (typically PushStack for triggered effects).
pub fn evaluate_triggers(
    engine: &mut GameEngine,
    event: &Event,
) -> Vec<Command> {
    let mut commands = Vec::new();

    match event {
        // CardMoved events can trigger "enters the battlefield" effects
        Event::CardMoved { card, to, .. } => {
            // Find the card in the destination zone to see who controls it
            if let Some(zone) = engine.state.zones.iter().find(|z| z.id == *to) {
                if zone.cards.contains(card) {
                    // Determine the controller (zone owner typically)
                    if let Some(controller) = zone.owner {
                        // Check if moving TO the field zone indicates "enters"
                        if to.0.starts_with("field") {
                            // Generate a generic "enters the battlefield" trigger
                            // In a real implementation, we'd look up the card's specific triggers
                            let trigger_effect = StackItem {
                                id: engine.next_stack_id(),
                                source: Some(*card),
                                controller,
                                effect: EffectRef::Builtin("etb"),
                            };
                            commands.push(Command::PushStack {
                                item: trigger_effect,
                            });
                        }
                    }
                }
            }
        }
        // CardPlayed events can trigger other card abilities
        Event::CardPlayed { player, card } => {
            // Generate a generic "card played" trigger
            let trigger_effect = StackItem {
                id: engine.next_stack_id(),
                source: Some(*card),
                controller: *player,
                effect: EffectRef::Builtin("card_played"),
            };
            commands.push(Command::PushStack {
                item: trigger_effect,
            });
        }
        _ => {
            // Other events don't trigger anything yet
        }
    }

    commands
}
