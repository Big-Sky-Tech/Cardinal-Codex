use crate::state::gamestate::GameState;
use crate::model::command::Command;
use crate::model::event::Event;

/// Apply a batch of commands to the `GameState` and return emitted events.
/// Each command mutates the state and produces one or more events.
pub fn commit_commands(state: &mut GameState, commands: &[Command]) -> Vec<Event> {
    let mut events = Vec::new();

    for cmd in commands {
        match cmd {
            Command::MoveCard { card, from, to } => {
                // Remove card from source zone
                if let Some(zone) = state.zones.iter_mut().find(|z| z.id == from.clone()) {
                    zone.cards.retain(|c| c != card);
                }
                // Add card to destination zone
                if let Some(zone) = state.zones.iter_mut().find(|z| z.id == to.clone()) {
                    zone.cards.push(*card);
                }
                events.push(Event::CardMoved { card: *card, from: from.clone(), to: to.clone() });
            }
            Command::ChangeLife { player, delta } => {
                if let Some(p) = state.players.iter_mut().find(|pl| pl.id == *player) {
                    p.life += delta;
                }
                events.push(Event::LifeChanged { player: *player, delta: *delta });
            }
            Command::PushStack { item } => {
                let stack_id = item.id;
                state.stack.push(item.clone());
                events.push(Event::StackPushed { item_id: stack_id });
            }
            Command::RequestChoice { player, choice } => {
                state.pending_choice = Some(choice.clone());
                events.push(Event::ChoiceRequested { choice_id: choice.id, player: *player });
            }
        }
    }

    events
}

// Event handling logic
