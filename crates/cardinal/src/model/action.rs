use crate::ids::{CardId, PlayerId, ZoneId};

#[derive(Debug, Clone)]
pub enum Action {
    PassPriority,
    Concede,

    // Example: play a card from a zone (usually hand)
    PlayCard {
        card: CardId,
        from: ZoneId,
    },

    // Example: choose target for a pending choice
    ChooseTarget {
        choice_id: u32,
        target: TargetRef,
    },
}

#[derive(Debug, Clone)]
pub enum TargetRef {
    Player(PlayerId),
    Card(CardId),
}
