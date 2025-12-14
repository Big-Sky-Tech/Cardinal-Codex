use crate::{
    engine::core::GameEngine,
    ids::PlayerId,
    model::action::Action,
    error::CardinalError,
};

pub fn validate(engine: &GameEngine, player: PlayerId, action: &Action) -> Result<(), CardinalError> {
    // Placeholder validation
    match action {
        Action::PassPriority => Ok(()),
        Action::Concede => Ok(()),
        Action::PlayCard { .. } => Ok(()),
        Action::ChooseTarget { .. } => Ok(()),
    }
}
