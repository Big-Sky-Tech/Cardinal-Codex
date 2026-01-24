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

    /// Build a GameEngine directly from a `Ruleset`. This will create a minimal GameState
    /// via `GameState::from_ruleset`.
    pub fn from_ruleset(rules: Ruleset, seed: u64) -> Self {
        let initial = GameState::from_ruleset(&rules);
        Self { rules, state: initial, seed, next_choice_id: 1, next_stack_id: 1 }
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
        let mut events = crate::engine::reducer::apply(self, player, action)?;

        // 3) post-step checks (win/loss, auto-resolve stack, advance phase)
        
        // Check for win/loss conditions
        self.check_game_end(&mut events);
        
        // Auto-resolve stack if it has items and no pending choice
        self.auto_resolve_stack(&mut events);
        
        // Advance to next phase/step if appropriate
        self.advance_phase_if_ready(&mut events);

        Ok(StepResult { events })
    }

    fn check_game_end(&mut self, events: &mut Vec<Event>) {
        // Check if any player has <= 0 life (loses)
        let losers: Vec<PlayerId> = self.state.players.iter()
            .filter(|p| p.life <= 0)
            .map(|p| p.id)
            .collect();

        if !losers.is_empty() {
            // Determine winner: last player with > 0 life
            let winner = self.state.players.iter()
                .find(|p| p.life > 0)
                .map(|p| p.id);
            
            self.state.ended = Some(crate::state::gamestate::GameEnd {
                winner,
                reason: "Life total reached 0".to_string(),
            });
            events.push(Event::GameEnded { winner, reason: "Life total reached 0".to_string() });
        }
    }

    fn auto_resolve_stack(&mut self, events: &mut Vec<Event>) {
        // If the stack has items and there's no pending choice, resolve the top item
        while !self.state.stack.is_empty() && self.state.pending_choice.is_none() {
            if let Some(item) = self.state.stack.pop() {
                let item_id = item.id;
                // Future: emit item's effects as commands
                events.push(Event::StackResolved { item_id });
            }
        }
    }

    fn advance_phase_if_ready(&mut self, _events: &mut Vec<Event>) {
        // TODO: implement phase/step progression logic
        // This should advance the turn phase when appropriate
    }

    fn validate_action(&self, player: PlayerId, action: &Action) -> Result<(), LegalityError> {
        crate::engine::legality::validate(self, player, action)
    }
}
