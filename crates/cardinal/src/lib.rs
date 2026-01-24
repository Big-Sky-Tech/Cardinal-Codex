pub mod error;
pub mod ids;

pub mod model;
pub mod rules;
pub mod state;
pub mod engine;
pub mod util;
pub mod display;

pub use engine::core::{GameEngine, StepResult};
pub use engine::init::initialize_game;
pub use error::{EngineError, LegalityError};
pub use model::action::Action;
pub use model::command::Command;
pub use model::event::Event;
pub use rules::schema::Ruleset;
pub use rules::RulesModule;
pub use state::gamestate::GameState;
pub use util::rng::GameRng;
pub use display::{GameDisplay, LogEntry};

use std::fs;
use std::path::Path;

use crate::error::CardinalError;
use crate::rules::schema::Ruleset as RulesetToml;

/// Load a `Ruleset` from a TOML file. Returns a conservative `CardinalError` on failure.
pub fn load_rules<P: AsRef<Path>>(path: P) -> Result<RulesetToml, CardinalError> {
    let content = fs::read_to_string(path).map_err(|e| CardinalError(format!("Failed to read rules file: {}", e)))?;
    toml::from_str(&content).map_err(|e| CardinalError(format!("Failed to parse TOML: {}", e)))
}
