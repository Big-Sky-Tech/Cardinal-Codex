use crate::state::gamestate::GameState;

/// Apply a batch of commands to the `GameState` and return emitted events.
/// This is intentionally minimal: it provides the commit point where full
/// command application logic will live. Right now it is a placeholder that
/// does not modify state but returns an empty Vec.
pub fn commit_commands(_state: &mut GameState, _commands: &[crate::model::command::Command]) -> Vec<crate::model::event::Event> {
    // TODO: implement command application (MoveCard, ChangeLife, PushStack, RequestChoice)
    Vec::new()
}

// Event handling logic
