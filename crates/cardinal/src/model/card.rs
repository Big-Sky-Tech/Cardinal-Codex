use crate::ids::CardId;

#[derive(Debug, Clone)]
pub struct Card {
    pub id: CardId,
    pub name: String,
    pub cost: i32,
    // Add more fields as needed
}
