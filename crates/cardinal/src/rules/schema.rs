use crate::ids::{ZoneId, PhaseId, StepId};

#[derive(Debug, Clone)]
pub struct Ruleset {
    pub zones: Vec<ZoneDef>,
    pub turn: TurnDef,
    pub priority_system: bool,
}

#[derive(Debug, Clone)]
pub struct ZoneDef {
    pub id: ZoneId,
    pub shared: bool,
    pub ordered: bool,
}

#[derive(Debug, Clone)]
pub struct TurnDef {
    pub phases: Vec<PhaseDef>,
}

#[derive(Debug, Clone)]
pub struct PhaseDef {
    pub id: PhaseId,
    pub steps: Vec<StepDef>,
}

#[derive(Debug, Clone)]
pub struct StepDef {
    pub id: StepId,
    pub allow_actions: bool,
    pub allow_triggers: bool,
}
