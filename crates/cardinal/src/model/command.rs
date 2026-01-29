use crate::ids::{CardId, PlayerId, ZoneId};

#[derive(Debug, Clone)]
pub enum Command {
    MoveCard { card: CardId, from: ZoneId, to: ZoneId },
    ChangeLife { player: PlayerId, delta: i32 },
    SetLife { player: PlayerId, amount: i32 },
    PushStack { item: StackItem },
    RequestChoice { player: PlayerId, choice: PendingChoice },
    ShuffleZone { zone: ZoneId },
    ModifyStats { card: CardId, power: i32, toughness: i32 },
    SetStats { card: CardId, power: i32, toughness: i32 },
    ModifyStat { card: CardId, stat_name: String, delta: i32 },
    SetStat { card: CardId, stat_name: String, value: String },
    GrantKeyword { card: CardId, keyword: String },
    RemoveKeyword { card: CardId, keyword: String },
    GainResource { player: PlayerId, resource: String, amount: i32 },
    SpendResource { player: PlayerId, resource: String, amount: i32 },
    SetResource { player: PlayerId, resource: String, amount: i32 },
    CreateToken { player: PlayerId, token_type: String, zone: ZoneId },
    AddCounter { card: CardId, counter_type: String, amount: i32 },
    RemoveCounter { card: CardId, counter_type: String, amount: i32 },
}

#[derive(Debug, Clone)]
pub struct StackItem {
    pub id: u32,
    pub source: Option<CardId>,
    pub controller: PlayerId,
    pub effect: EffectRef,
}

#[derive(Debug, Clone)]
pub enum EffectRef {
    Builtin(&'static str),
    Scripted(String), // mod-defined
}

#[derive(Debug, Clone)]
pub struct PendingChoice {
    pub id: u32,
    pub prompt: String,
    pub kind: ChoiceKind,
}

#[derive(Debug, Clone)]
pub enum ChoiceKind {
    ChooseTarget { allowed: AllowedTargets },
}

#[derive(Debug, Clone)]
pub enum AllowedTargets {
    AnyCreatureOnField,
    AnyPlayer,
    // etc
}
