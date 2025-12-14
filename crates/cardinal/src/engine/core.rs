use crate::{
    error::{EngineError, LegalityError},
    ids::PlayerId,
    model::action::Action,
    model::event::Event,
    rules::schema::Ruleset,
    state::gamestate::GameState,
};

pub struct GameEngine {
    pub rules: Ruleset,
    pub state: GameState,
    seed: u64,
    next_choice_id: u32,
    next_stack_id: u32,
}

pub struct StepResult {
    pub events: Vec<Event>,
}

impl GameEngine {
    pub fn new(rules: Ruleset, seed: u64, initial_state: GameState) -> Self {
        Self { rules, state: initial_state, seed, next_choice_id: 1, next_stack_id: 1 }
    }

    pub fn legal_actions(&self, player: PlayerId) -> Vec<Action> {
        // Start simple: implement legality later in engine/legality.rs
        // Return only actions that make sense (PassPriority, PlayCard if allowed, etc).
        vec![Action::PassPriority]
    }

    pub fn apply_action(&mut self, player: PlayerId, action: Action) -> Result<StepResult, EngineError> {
        // 1) validate
        self.validate_action(player, &action)?;

        // 2) apply (reducer)
        let events = crate::engine::reducer::apply(self, player, action)?;

        // 3) post-step checks (win/loss, auto-resolve stack, advance phase)
        // TODO

        Ok(StepResult { events })
    }

    fn validate_action(&self, player: PlayerId, action: &Action) -> Result<(), LegalityError> {
        crate::engine::legality::validate(self, player, action)
    }
}
