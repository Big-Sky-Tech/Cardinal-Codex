use crate::ids::{PlayerId, ZoneId, PhaseId, StepId, CardId};
use crate::model::command::{PendingChoice, StackItem};

#[derive(Debug, Clone)]
pub struct GameState {
    pub turn: TurnState,
    pub players: Vec<PlayerState>,
    pub zones: Vec<ZoneState>,
    pub stack: Vec<StackItem>,
    pub pending_choice: Option<PendingChoice>,
    pub ended: Option<GameEnd>,
}

#[derive(Debug, Clone)]
pub struct TurnState {
    pub number: u32,
    pub active_player: PlayerId,
    pub priority_player: PlayerId,
    pub phase: PhaseId,
    pub step: StepId,
}

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub id: PlayerId,
    pub life: i32,
    // resources, flags, etc
}

#[derive(Debug, Clone)]
pub struct ZoneState {
    pub id: ZoneId,
    pub owner: Option<PlayerId>, // None for shared zones like stack
    pub cards: Vec<CardId>,
}

#[derive(Debug, Clone)]
pub struct GameEnd {
    pub winner: Option<PlayerId>,
    pub reason: String,
}
