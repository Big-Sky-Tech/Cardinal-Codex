Build **Cardinal** like a little operating system for rules: small, deterministic “kernel” (the library), and everything else (UI, CLI, mods) calls it through a tight API.

Here’s a solid way to write the `cardinal` **lib crate** so it stays clean and moddable.

## The core design rules

1. **Headless + deterministic**

* No rendering, no time, no threads, no RNG calls directly (engine owns seeded RNG).
* Same inputs → same outputs.

2. **Actions in, Events out**

* UI/AI sends an `Action`.
* Engine validates it, applies it, returns `Vec<Event>` (for UI + logs).

3. **State is authoritative**

* One `GameState` struct contains truth.
* Everything else is derived or queried.

4. **No “magic” mutations**

* All changes happen via a controlled `apply_action()` pipeline.
* Effects return “commands” (like syscalls), engine commits them.

This is what makes “swap rules without rewriting engine” actually possible.

---

## Recommended crate layout

```
crates/cardinal/src/
  lib.rs
  error.rs
  ids.rs
  state/
    mod.rs
    gamestate.rs
    zones.rs
  rules/
    mod.rs
    schema.rs        # loaded from TOML (phases/zones/action defs)
    query.rs         # what engine asks rules/mods
  engine/
    mod.rs
    core.rs          # GameEngine struct
    reducer.rs       # apply action -> state changes
    legality.rs      # validate actions
    events.rs
  model/
    mod.rs
    action.rs
    event.rs
    command.rs
    choice.rs
    card.rs
  util/
    rng.rs
    log.rs
```

This keeps “data model” separate from “engine logic” and separate from “rules schema / plugins”.

---

## The minimal API surface you want

### Public types (stable-ish)

* `GameEngine`
* `GameState` (or `PublicStateView` + `PlayerView`)
* `Action`
* `Event`
* `LegalityError` / `EngineError`
* `PendingChoice` (for “engine needs player input now”)

### Public functions

* `GameEngine::new(rules: Ruleset, seed: u64) -> Self`
* `engine.start_game(setup: GameSetup) -> EngineResult`
* `engine.player_view(player_id) -> PlayerView`
* `engine.legal_actions(player_id) -> Vec<Action>`
* `engine.apply_action(player_id, action) -> StepResult`

Where `StepResult` includes:

* new state snapshot info (or just keep state in engine)
* `events: Vec<Event>`
* `pending_choice: Option<PendingChoice>` (if more input needed)

---

## The “kernel” core structs (skeleton)

Below is a *minimal* foundation you can paste and grow. It’s intentionally conservative and clean.

### `lib.rs`

```rust
pub mod error;
pub mod ids;

pub mod model;
pub mod rules;
pub mod state;
pub mod engine;

pub use engine::core::{GameEngine, StepResult};
pub use error::{EngineError, LegalityError};
pub use model::{Action, Event};
pub use rules::schema::Ruleset;
pub use state::gamestate::GameState;
```

### `ids.rs`

Use newtypes so you don’t mix up IDs accidentally.

```rust
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
```

### `model/action.rs`

```rust
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
```

### `model/event.rs`

Events are what the UI and replay system live on.

```rust
use crate::ids::{CardId, PlayerId, ZoneId, PhaseId, StepId};

#[derive(Debug, Clone)]
pub enum Event {
    PhaseAdvanced { phase: PhaseId, step: StepId },
    PriorityPassed { by: PlayerId },
    CardMoved { card: CardId, from: ZoneId, to: ZoneId },
    CardPlayed { player: PlayerId, card: CardId },
    LifeChanged { player: PlayerId, delta: i32 },
    StackPushed { item_id: u32 },
    StackResolved { item_id: u32 },
    ChoiceRequested { choice_id: u32, player: PlayerId },
    GameEnded { winner: Option<PlayerId>, reason: String },
}
```

### `model/command.rs`

Commands are what rules/plugins produce; engine commits them.

```rust
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
```

### `state/gamestate.rs`

Keep it boring and explicit.

```rust
use crate::ids::{PlayerId, ZoneId, PhaseId, StepId, CardId};
use crate::model::command::{PendingChoice, StackItem};

#[derive(Debug, Clone)]
pub struct GameState {
    pub turn: TurnState,
    pub players: Vec<PlayerState>,
    pub zones: Vec<ZoneState>,
    pub stack: Vec<StackItem>,
    pub pending_choice: Option<PendingChoice>,
    pub ended: Option<GameEnd>,
}

#[derive(Debug, Clone)]
pub struct TurnState {
    pub number: u32,
    pub active_player: PlayerId,
    pub priority_player: PlayerId,
    pub phase: PhaseId,
    pub step: StepId,
}

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub id: PlayerId,
    pub life: i32,
    // resources, flags, etc
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
```

### `rules/schema.rs`

This is what you load from TOML. Start minimal.

```rust
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
```

### `engine/core.rs`

The engine owns state, rules, RNG, and emits events.

```rust
use crate::{
    error::{EngineError, LegalityError},
    ids::PlayerId,
    model::{Action, Event},
    rules::schema::Ruleset,
    state::gamestate::GameState,
};

pub struct GameEngine {
    pub rules: Ruleset,
    pub state: GameState,
    seed: u64,
    next_choice_id: u32,
    next_stack_id: u32,
}

pub struct StepResult {
    pub events: Vec<Event>,
}

impl GameEngine {
    pub fn new(rules: Ruleset, seed: u64, initial_state: GameState) -> Self {
        Self { rules, state: initial_state, seed, next_choice_id: 1, next_stack_id: 1 }
    }

    pub fn legal_actions(&self, player: PlayerId) -> Vec<Action> {
        // Start simple: implement legality later in engine/legality.rs
        // Return only actions that make sense (PassPriority, PlayCard if allowed, etc).
        vec![Action::PassPriority]
    }

    pub fn apply_action(&mut self, player: PlayerId, action: Action) -> Result<StepResult, EngineError> {
        // 1) validate
        self.validate_action(player, &action).map_err(EngineError::Illegal)?;

        // 2) apply (reducer)
        let events = crate::engine::reducer::apply(self, player, action)?;

        // 3) post-step checks (win/loss, auto-resolve stack, advance phase)
        // TODO

        Ok(StepResult { events })
    }

    fn validate_action(&self, player: PlayerId, action: &Action) -> Result<(), LegalityError> {
        crate::engine::legality::validate(self, player, action)
    }
}
```

---

## How to grow it in the right order

If you want momentum without building a monster:

1. **Phase/step advance + PassPriority**

* Implement phase stepping from `Ruleset`.
* Add priority passing that auto-advances when both pass (if stack empty).

2. **Zones + move card**

* Implement “draw” and “play card” as basic commands.
* Emit events always.

3. **Pending choices**

* Engine can pause and ask for target selection.

4. **Stack**

* Push effects onto stack, resolve LIFO.

5. **Triggers**

* Event bus: events generate triggers, triggers push stack items.

6. **Plugin boundary**

* Introduce a `RulesModule` trait (or scripted interface) that can:

    * inspect state read-only
    * return commands to apply

That’s when your “swap rules without rewriting core” dream becomes real.

---

## The plugin hook you’ll want (even before scripting)

Before you do Rust-script, start with a trait so you can test the idea:

```rust
pub trait RulesModule {
    fn on_event(&self, ctx: &RulesContext, ev: &crate::model::Event) -> Vec<crate::model::command::Command>;
    fn legal_overrides(&self, ctx: &RulesContext, player: crate::ids::PlayerId) -> Vec<crate::model::Action>;
}

pub struct RulesContext<'a> {
    pub rules: &'a crate::rules::schema::Ruleset,
    pub state: &'a crate::state::gamestate::GameState,
}
```

Later, your “script engine” becomes an implementation of `RulesModule`.

---

If you follow this, you’ll end up with a library that:

* is easy to embed in Unity/Godot/Bevy
* is testable like a compiler
* can accept new rules via modules without core rewrites

When you’re ready, I can also sketch the **exact `GameState`/`Action`/`Event`/`Command` shapes** that make card scripting and format overrides painless (and not an endless refactor loop).
