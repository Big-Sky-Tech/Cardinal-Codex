use crate::ids::{CardId, PlayerId, ZoneId};

#[derive(Debug, Clone)]
pub enum Command {
    MoveCard { card: CardId, from: ZoneId, to: ZoneId },
    ChangeLife { player: PlayerId, delta: i32 },
    PushStack { item: StackItem },
    RequestChoice { player: PlayerId, choice: PendingChoice },
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
