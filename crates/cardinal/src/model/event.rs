use crate::ids::{CardId, PlayerId, ZoneId, PhaseId, StepId};

#[derive(Debug, Clone)]
pub enum Event {
    PhaseAdvanced { phase: PhaseId, step: StepId },
    PriorityPassed { by: PlayerId },
    CardMoved { card: CardId, from: ZoneId, to: ZoneId },
    CardPlayed { player: PlayerId, card: CardId },
    LifeChanged { player: PlayerId, delta: i32 },
    LifeSet { player: PlayerId, amount: i32 },
    StackPushed { item_id: u32 },
    StackResolved { item_id: u32 },
    ChoiceRequested { choice_id: u32, player: PlayerId },
    GameEnded { winner: Option<PlayerId>, reason: String },
    ZoneShuffled { zone: ZoneId },
    StatsModified { card: CardId, power: i32, toughness: i32 },
    StatsSet { card: CardId, power: i32, toughness: i32 },
    StatModified { card: CardId, stat_name: String, delta: i32 },
    StatSet { card: CardId, stat_name: String, value: String },
    KeywordGranted { card: CardId, keyword: String },
    KeywordRemoved { card: CardId, keyword: String },
    ResourceGained { player: PlayerId, resource: String, amount: i32 },
    ResourceSpent { player: PlayerId, resource: String, amount: i32 },
    ResourceSet { player: PlayerId, resource: String, amount: i32 },
    TokenCreated { player: PlayerId, token_type: String, card: CardId, zone: ZoneId },
    CounterAdded { card: CardId, counter_type: String, amount: i32 },
    CounterRemoved { card: CardId, counter_type: String, amount: i32 },
}
