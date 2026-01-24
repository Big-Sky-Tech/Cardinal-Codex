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

    /// Generate the next unique stack item ID
    pub fn next_stack_id(&mut self) -> u32 {
        let id = self.next_stack_id;
        self.next_stack_id += 1;
        id
    }

    /// Generate the next unique choice ID
    pub fn next_choice_id(&mut self) -> u32 {
        let id = self.next_choice_id;
        self.next_choice_id += 1;
        id
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

    fn advance_phase_if_ready(&mut self, events: &mut Vec<Event>) {
        // Phase advancement logic with priority system:
        // 1. Only advance if stack is empty and no pending choices
        // 2. Check if all players have passed priority
        // 3. If all passed: resolve stack items, reset passes, advance phase
        // 4. If not all passed: don't advance yet, wait for more passes
        
        if !self.state.stack.is_empty() || self.state.pending_choice.is_some() {
            // Can't advance while stack has items or choice is pending
            return;
        }

        let num_players = self.state.players.len() as u32;
        
        // Check if all players have passed priority
        if self.state.turn.priority_passes < num_players {
            // Not all players have passed yet, don't advance
            return;
        }
        
        // All players have passed! Reset priority counter and give priority to active player
        self.state.turn.priority_passes = 0;
        self.state.turn.priority_player = self.state.turn.active_player;

        // Find current phase index
        let current_phase_idx = self.rules.turn.phases.iter()
            .position(|p| p.id.as_str() == self.state.turn.phase.0)
            .unwrap_or(0);
        let current_phase = &self.rules.turn.phases[current_phase_idx];

        // Find current step index within current phase
        let current_step_idx = current_phase.steps.iter()
            .position(|s| s.id.as_str() == self.state.turn.step.0)
            .unwrap_or(0);

        // Try to advance to next step in current phase
        if current_step_idx + 1 < current_phase.steps.len() {
            let next_step = &current_phase.steps[current_step_idx + 1];
            let step_box: Box<str> = next_step.id.clone().into_boxed_str();
            let step_static: &'static str = Box::leak(step_box);
            self.state.turn.step = crate::ids::StepId(step_static);
            events.push(Event::PhaseAdvanced {
                phase: self.state.turn.phase.clone(),
                step: self.state.turn.step.clone(),
            });
            return;
        }

        // No more steps in current phase; advance to next phase
        if current_phase_idx + 1 < self.rules.turn.phases.len() {
            let next_phase = &self.rules.turn.phases[current_phase_idx + 1];
            let phase_box: Box<str> = next_phase.id.clone().into_boxed_str();
            let phase_static: &'static str = Box::leak(phase_box);
            self.state.turn.phase = crate::ids::PhaseId(phase_static);

            // Start at first step of new phase
            if let Some(first_step) = next_phase.steps.first() {
                let step_box: Box<str> = first_step.id.clone().into_boxed_str();
                let step_static: &'static str = Box::leak(step_box);
                self.state.turn.step = crate::ids::StepId(step_static);
            } else {
                self.state.turn.step = crate::ids::StepId("start");
            }

            events.push(Event::PhaseAdvanced {
                phase: self.state.turn.phase.clone(),
                step: self.state.turn.step.clone(),
            });
            return;
        }

        // End of turn: cycle back to first phase and advance turn number
        if let Some(first_phase) = self.rules.turn.phases.first() {
            let phase_box: Box<str> = first_phase.id.clone().into_boxed_str();
            let phase_static: &'static str = Box::leak(phase_box);
            self.state.turn.phase = crate::ids::PhaseId(phase_static);

            if let Some(first_step) = first_phase.steps.first() {
                let step_box: Box<str> = first_step.id.clone().into_boxed_str();
                let step_static: &'static str = Box::leak(step_box);
                self.state.turn.step = crate::ids::StepId(step_static);
            } else {
                self.state.turn.step = crate::ids::StepId("start");
            }

            self.state.turn.number += 1;

            // Rotate active player and give them priority
            let next_player_idx = (self.state.turn.active_player.0 + 1) % num_players as u8;
            self.state.turn.active_player = crate::ids::PlayerId(next_player_idx);
            self.state.turn.priority_player = self.state.turn.active_player;

            events.push(Event::PhaseAdvanced {
                phase: self.state.turn.phase.clone(),
                step: self.state.turn.step.clone(),
            });
        }
    }

    fn validate_action(&self, player: PlayerId, action: &Action) -> Result<(), LegalityError> {
        crate::engine::legality::validate(self, player, action)
    }
}
