use crate::{
    model::event::Event,
    model::command::Command,
    engine::core::GameEngine,
};

/// Evaluate which triggers should fire in response to an event.
/// Returns commands to execute (typically PushStack for triggered effects).
/// This uses card definitions to determine which abilities should fire.
pub fn evaluate_triggers(
    engine: &mut GameEngine,
    event: &Event,
) -> Vec<Command> {
    let mut commands = Vec::new();
    let mut next_stack_id = engine.next_stack_id();

    match event {
        // CardMoved events can trigger "enters the battlefield" effects (ETB triggers)
        Event::CardMoved { card, to, .. } => {
            // Check if moving TO the field zone indicates "enters"
            if to.0.starts_with("field") {
                // Find the controller (zone owner)
                if let Some(zone) = engine.state.zones.iter().find(|z| z.id == *to) {
                    if let Some(controller) = zone.owner {
                        // Look up card's abilities and fire matching triggers
                        let ability_commands = crate::engine::cards::generate_ability_commands(
                            *card,
                            "etb",
                            controller,
                            &engine.cards,
                            &mut next_stack_id,
                        );
                        commands.extend(ability_commands);
                    }
                }
            }
        }
        // CardPlayed events can trigger on_play card abilities
        Event::CardPlayed { player, card } => {
            // Look up card's abilities and fire matching triggers
            let ability_commands = crate::engine::cards::generate_ability_commands(
                *card,
                "on_play",
                *player,
                &engine.cards,
                &mut next_stack_id,
            );
            commands.extend(ability_commands);
        }
        _ => {
            // Other events don't trigger anything yet
        }
    }

    commands
}
