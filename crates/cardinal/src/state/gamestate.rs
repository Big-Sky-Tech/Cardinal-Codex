use crate::ids::{PlayerId, ZoneId, PhaseId, StepId, CardId};
use crate::model::command::{PendingChoice, StackItem};
use crate::rules::schema::Ruleset;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct GameState {
    pub turn: TurnState,
    pub players: Vec<PlayerState>,
    pub zones: Vec<ZoneState>,
    pub stack: Vec<StackItem>,
    pub pending_choice: Option<PendingChoice>,
    pub ended: Option<GameEnd>,
    /// Card instance data (stats, counters, keywords)
    pub card_instances: HashMap<CardId, CardInstanceData>,
}

#[derive(Debug, Clone)]
pub struct TurnState {
    pub number: u32,
    pub active_player: PlayerId,
    pub priority_player: PlayerId,
    pub phase: PhaseId,
    pub step: StepId,
    pub priority_passes: u32,  // Number of consecutive players who have passed priority
}

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub id: PlayerId,
    pub life: i32,
    /// Named resources (e.g., "mana", "action_points")
    pub resources: HashMap<String, i32>,
}

#[derive(Debug, Clone)]
pub struct ZoneState {
    pub id: ZoneId,
    pub owner: Option<PlayerId>, // None for shared zones like stack
    pub cards: Vec<CardId>,
}

#[derive(Debug, Clone)]
pub struct GameEnd {
    pub winner: Option<PlayerId>,
    pub reason: String,
}

/// Runtime data for a card instance
#[derive(Debug, Clone)]
pub struct CardInstanceData {
    /// Current stats (e.g., "power", "toughness", "range")
    pub stats: HashMap<String, String>,
    /// Temporary stat modifiers (power/toughness pumps, etc.)
    pub stat_modifiers: HashMap<String, i32>,
    /// Keywords the card currently has
    pub keywords: Vec<String>,
    /// Counters on the card (e.g., "+1/+1", "charge")
    pub counters: HashMap<String, i32>,
}

impl GameState {
    /// Build an initial `GameState` from a `Ruleset`. This is intentionally conservative
    /// and does not shuffle or populate decks; it just creates players, zones, and a starting turn.
    pub fn from_ruleset(rules: &Ruleset) -> Self {
        let min_players = rules.players.min_players as usize;
        let mut players = Vec::new();
        for i in 0..min_players {
            players.push(PlayerState { 
                id: PlayerId(i as u8), 
                life: rules.players.starting_life,
                resources: HashMap::new(),
            });
        }

        // Build zones: player-owned zones get one ZoneState per player; shared zones get a single ZoneState
        let mut zones = Vec::new();
        for z in &rules.zones {
            match z.owner_scope {
                crate::rules::schema::ZoneOwnerScope::Player => {
                    for i in 0..min_players {
                        let zid_string = format!("{}@{}", z.id, i);
                        let boxed = zid_string.into_boxed_str();
                        let static_str: &'static str = Box::leak(boxed);
                        zones.push(ZoneState { id: ZoneId(static_str), owner: Some(PlayerId(i as u8)), cards: Vec::new() });
                    }
                }
                crate::rules::schema::ZoneOwnerScope::Shared => {
                    let boxed = z.id.clone().into_boxed_str();
                    let static_str: &'static str = Box::leak(boxed);
                    zones.push(ZoneState { id: ZoneId(static_str), owner: None, cards: Vec::new() });
                }
            }
        }

        // Starting phase/step: use first defined phase/step if present, otherwise fallbacks
        let (phase_id, step_id) = if let Some(first_phase) = rules.turn.phases.first() {
            let ph_box: Box<str> = first_phase.id.clone().into_boxed_str();
            let ph_static: &'static str = Box::leak(ph_box);
            if let Some(first_step) = first_phase.steps.first() {
                let st_box: Box<str> = first_step.id.clone().into_boxed_str();
                let st_static: &'static str = Box::leak(st_box);
                (PhaseId(ph_static), StepId(st_static))
            } else {
                (PhaseId(ph_static), StepId("start"))
            }
        } else {
            (PhaseId("start"), StepId("untap"))
        };

        GameState {
            turn: TurnState { number: 1, active_player: PlayerId(0), priority_player: PlayerId(0), phase: phase_id, step: step_id, priority_passes: 0 },
            players,
            zones,
            stack: Vec::new(),
            pending_choice: None,
            ended: None,
            card_instances: HashMap::new(),
        }
    }
}
