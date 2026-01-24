use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ruleset {
    pub game: GameInfo,
    pub players: PlayerRules,
    pub zones: Vec<ZoneDef>,
    pub resources: Vec<ResourceDef>,
    pub turn: TurnStructure,
    pub actions: Vec<ActionDef>,
    pub stack: StackRules,
    pub trigger_kinds: Vec<TriggerKind>,
    pub keywords: Vec<Keyword>,
    pub win_conditions: Vec<WinCondition>,
    pub loss_conditions: Vec<LossCondition>,
    #[serde(default)]
    pub cards: Vec<CardDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerRules {
    pub min_players: usize,
    pub max_players: usize,
    pub starting_life: i32,
    pub max_life: i32,
    pub starting_hand_size: usize,
    pub max_hand_size: usize,
    pub min_deck_size: usize,
    pub max_deck_size: usize,
    pub mulligan_rule: String,
    pub first_player_rule: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneDef {
    pub id: String,
    pub name: String,
    pub owner_scope: ZoneOwnerScope,
    pub visibility: ZoneVisibility,
    pub ordered: bool,
    pub allow_duplicates: bool,
    pub default_capacity: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ZoneOwnerScope {
    Player,
    Shared,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ZoneVisibility {
    Public,
    Private,
    TopCardPublic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub min_value: i32,
    pub max_value: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnStructure {
    pub priority_system: bool,
    pub skip_first_turn_draw_for_first_player: bool,
    pub phases: Vec<PhaseDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseDef {
    pub id: String,
    pub name: String,
    pub order: usize,
    pub allow_actions: bool,
    pub steps: Vec<StepDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepDef {
    pub id: String,
    pub name: String,
    pub order: usize,
    pub allow_actions: bool,
    pub allow_triggers: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub source_zones: Option<Vec<String>>,
    pub target_zone: Option<String>,
    pub speed: Option<String>,
    #[serde(default)]
    pub requires_empty_stack: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackRules {
    pub enabled: bool,
    pub resolve_order: String,
    pub auto_resolve_on_pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerKind {
    pub id: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyword {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WinCondition {
    pub id: String,
    pub description: String,
    pub priority: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LossCondition {
    pub id: String,
    pub description: String,
    pub priority: usize,
}

/// Card definition: metadata and abilities for a playable card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardDef {
    /// Unique card identifier (can be string or number)
    pub id: String,
    /// Card name for display
    pub name: String,
    /// Card type (e.g., "creature", "spell", "enchantment")
    pub card_type: String,
    /// Cost to play (could be mana, resources, etc)
    pub cost: Option<String>,
    /// Card text / description
    pub description: Option<String>,
    /// Abilities this card has (triggered effects)
    #[serde(default)]
    pub abilities: Vec<CardAbility>,
}

/// An ability on a card that can be triggered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardAbility {
    /// What triggers this ability (e.g., "etb", "on_play", "on_damage")
    pub trigger: String,
    /// What effect to execute (e.g., "damage_2", "draw_1", "pump_1_1")
    pub effect: String,
    /// Optional parameters for the effect (e.g., amount, target)
    #[serde(default)]
    pub params: std::collections::HashMap<String, String>,
}
