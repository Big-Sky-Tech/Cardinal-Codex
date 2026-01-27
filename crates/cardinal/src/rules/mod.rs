pub mod schema;
pub mod query;
pub mod card_loader;

use crate::rules::schema::Ruleset;
use crate::state::gamestate::GameState;

/// Rule module trait: returns commands in response to events or legality overrides.
pub trait RulesModule {
    fn on_event(&self, _ctx: &RulesContext, _ev: &crate::model::event::Event) -> Vec<crate::model::command::Command> {
        Vec::new()
    }

    fn legal_overrides(&self, _ctx: &RulesContext, _player: crate::ids::PlayerId) -> Vec<crate::model::action::Action> {
        Vec::new()
    }
}

pub struct RulesContext<'a> {
    pub rules: &'a Ruleset,
    pub state: &'a GameState,
}
