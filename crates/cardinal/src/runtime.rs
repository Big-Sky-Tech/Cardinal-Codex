use std::path::Path;

use crate::{
    Action,
    CardSource,
    EngineError,
    GameEngine,
    GameState,
    StepResult,
    error::CardinalError,
    ids::{CardId, PlayerId},
    load_game_config,
};

/// Minimal runtime wrapper for loading a ruleset, creating decks, and driving a match.
pub struct GameRuntime {
    engine: GameEngine,
}

impl GameRuntime {
    pub fn load<P: AsRef<Path>>(
        rules_path: P,
        card_sources: Option<Vec<CardSource>>,
        seed: u64,
    ) -> Result<Self, CardinalError> {
        let rules = load_game_config(rules_path, card_sources)?;
        Ok(Self { engine: GameEngine::new(rules, seed) })
    }

    pub fn engine(&self) -> &GameEngine {
        &self.engine
    }

    pub fn public_state(&self) -> GameState {
        self.engine.public_state()
    }

    pub fn player_view(&self, player: PlayerId) -> GameState {
        self.engine.player_view(player)
    }

    pub fn legal_actions(&self, player: PlayerId) -> Vec<Action> {
        self.engine.legal_actions(player)
    }

    pub fn apply_action(&mut self, player: PlayerId, action: Action) -> Result<StepResult, EngineError> {
        self.engine.apply_action(player, action)
    }

    pub fn start_game(&mut self, decks: Vec<Vec<CardId>>) -> Result<(), EngineError> {
        self.engine.start_game(decks)
    }

    pub fn build_mirror_decks(&self, deck_size: usize) -> Result<Vec<Vec<CardId>>, EngineError> {
        let mut card_ids: Vec<CardId> = self.engine.rules.cards.iter()
            .filter_map(|card| card.id.parse::<u32>().ok().map(CardId))
            .collect();

        if card_ids.is_empty() {
            card_ids = (0..deck_size.max(1))
                .map(|index| CardId(index as u32))
                .collect();
        }

        let mut decks = Vec::new();
        for _ in 0..self.engine.state().players.len() {
            let deck = (0..deck_size)
                .map(|idx| card_ids[idx % card_ids.len()])
                .collect();
            decks.push(deck);
        }

        Ok(decks)
    }
}
