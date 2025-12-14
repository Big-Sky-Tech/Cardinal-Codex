#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerId(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CardId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ZoneId(pub &'static str);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhaseId(pub &'static str);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StepId(pub &'static str);
