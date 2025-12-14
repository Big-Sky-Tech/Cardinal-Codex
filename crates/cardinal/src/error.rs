#[derive(Debug)]
pub struct CardinalError(pub String);

pub type EngineError = CardinalError;
pub type LegalityError = CardinalError;
