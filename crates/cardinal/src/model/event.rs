use crate::ids::{CardId, PlayerId, ZoneId, PhaseId, StepId};

#[derive(Debug, Clone)]
pub enum Event {
    PhaseAdvanced { phase: PhaseId, step: StepId },
    PriorityPassed { by: PlayerId },
    CardMoved { card: CardId, from: ZoneId, to: ZoneId },
    CardPlayed { player: PlayerId, card: CardId },
    LifeChanged { player: PlayerId, delta: i32 },
    StackPushed { item_id: u32 },
    StackResolved { item_id: u32 },
    ChoiceRequested { choice_id: u32, player: PlayerId },
    GameEnded { winner: Option<PlayerId>, reason: String },
}
