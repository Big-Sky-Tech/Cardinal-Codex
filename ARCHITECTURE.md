# Cardinal Architecture & Design Guide

## What is Cardinal?

Cardinal is a **game rules engine** — think of it as the "brain" that runs a trading card game (TCG). It doesn't draw pictures, play sounds, or provide buttons to click. Instead, it:

1. **Tracks game state** — whose turn is it, what cards are in play, how much health does each player have
2. **Validates actions** — can you really play that card right now? Do you have enough mana?
3. **Applies effects** — when you play a creature with a "draw a card" ability, Cardinal makes sure that card gets drawn
4. **Fires triggers** — when something happens (like a creature entering the field), Cardinal checks if any card abilities trigger
5. **Emits events** — tells the outside world what happened, so a UI can show animations and updates

Cardinal is **headless** — it has no idea what a screen is. This makes it reusable: the same Cardinal engine could power a web game, a desktop app, a mobile game, or even a Discord bot.

## Core Design Philosophy

Cardinal is built on **four immutable principles**:

### 1. **Determinism**
Same starting state + same actions + same random seed = identical game outcome, every time.

Why? Because:
- **Replays work perfectly** — you can show exactly what happened
- **Network games are fair** — both players can verify the outcome without trusting the other
- **Debugging is possible** — you can reproduce bugs exactly

**How?** No system time, no threading, no random number generator calls outside Cardinal's control. All randomness comes from Cardinal's own seeded random number generator (RNG).

### 2. **Headless Architecture**
Cardinal has no UI, rendering, audio, or assumptions about how it will be used.

Why? Because:
- **It's embeddable** — any frontend can use it (web, desktop, mobile, AI, etc.)
- **Concerns are separated** — game logic stays pure; presentation can change
- **Testing is easier** — no UI framework to mock or struggle with

**How?** Cardinal only deals with data (game state) and logic (rules). All rendering, input handling, and networking live outside.

### 3. **Actions In, Events Out**
Cardinal's interface is simple and unidirectional:

```
User/AI chooses an action
         ↓
   Cardinal validates it
         ↓
  Cardinal applies it
         ↓
Cardinal emits events describing what happened
         ↓
    UI reads events and updates
```

Why?
- **Clear boundaries** — you always know what's happening and why
- **Auditability** — events become a complete game log
- **Extensibility** — new UIs can read the same event stream

**How?** Every change flows through `apply_action()`. No side effects, no hidden mutations.

### 4. **GameState is Authoritative**
There is one source of truth: the `GameState` struct. Everything else is derived from it or queried from it.

Why?
- **No conflicts** — you never have to ask "which version is correct?"
- **Consistency** — if you know the state, you know everything about the game
- **Simplicity** — no syncing, no race conditions

**How?** Cardinal stores the current `GameState`. When you apply an action, a new `GameState` is computed. Nothing else holds the truth.

---

## How Cardinal Works: The Game Loop

Here's what happens when a player plays a card:

### Step 1: Legality Check
The player says "I want to play card X."

Cardinal checks:
- Is it actually this player's turn?
- Is the game in a phase where playing cards is legal?
- Does the player own the card?
- Is the stack empty (so we can play a spell)?
- Does the player have enough mana?

If any check fails, Cardinal rejects the action with a clear error.

### Step 2: Effect Application
The action is legal. Cardinal applies the effect:
- Move the card from hand to field
- Subtract mana from the player's pool
- Mark the turn state to reflect that something happened

### Step 3: Trigger Evaluation
Now that something changed, Cardinal checks: "Did any card abilities trigger?"

For example:
- "Whenever a creature enters the field, draw a card" → fires!
- "At the start of your turn, gain 1 life" → doesn't fire (it's not the start of turn)

Triggered abilities create **stack items** that will resolve later.

### Step 4: Event Emission
Cardinal creates an event for each meaningful thing that happened:
- `CardPlayed { player, card, from_zone, to_zone }`
- `CardMoved { ... }`
- `AbilityTriggered { card, ability_name }`
- etc.

These events are returned to the caller (the UI).

### Step 5: UI Updates
The UI reads the events and updates the screen. For example:
- "Card played" → animate card moving from hand to field
- "Ability triggered" → show a notification
- "Life total changed" → update the life counter

---

## Game State: What Cardinal Remembers

The `GameState` struct is the core:

```
GameState {
  players: [Player, Player]  // Player 0 and Player 1
  turn: Turn {
    number: 1,                          // Which turn is it?
    phase: "main",                      // start, main, combat, main, end?
    step: "untap",                      // Within the phase, which step?
    active_player: PlayerId(0),         // Whose turn?
    priority_holder: PlayerId(0),       // Who can act right now?
  }
  zones: {
    hand[0]: [Card, Card, Card],        // Player 0's hand
    hand[1]: [Card],                    // Player 1's hand
    field[0]: [Card, Card],             // Player 0's field
    field[1]: [],                       // Player 1's field
    library[0]: [Card, ...],            // Player 0's deck
    library[1]: [Card, ...],            // Player 1's deck
    graveyard[0]: [],                   // Player 0's discard pile
    graveyard[1]: [],                   // Player 1's discard pile
  }
  stack: [Item, Item],  // Spells and abilities waiting to resolve
  players[0]: {
    life: 20,
    mana: { white: 2, blue: 1, red: 3 },  // Mana pools
    ...
  }
  players[1]: { life: 20, ... }
}
```

Everything the game needs to know is here. A player can ask "what's the game state?" and get a complete picture.

---

## Cards: How They Work

### Card Definition (Declarative)

In `rules.toml`, you define a card once:

```toml
[[cards]]
id = 1
name = "Goblin Scout"
type = "creature"
cost = "1R"  # 1 generic + 1 red mana
description = "A small but feisty goblin."

[[cards.abilities]]
trigger = "etb"  # "enters the battlefield"
effect = "damage"
value = 1
target = "opponent"
```

### Card Execution (Data-Driven)

When the card is played:

1. **Load the definition** — Cardinal looks up card #1 in the registry
2. **Find matching triggers** — "Does 'etb' apply right now?" → Yes!
3. **Create a command** — "Do damage 1 to opponent"
4. **Push to stack** — The damage waits to resolve
5. **Resolve the stack** — Damage is applied, events are emitted

There's no hardcoded logic for "Goblin Scout" in the engine. The card is 100% defined in data (TOML). This means:
- **New cards don't require code changes**
- **Rules can be customized per game**
- **Modding becomes feasible**

---

## Turns: The Flow of Time

A turn follows a rigid structure:

```
Turn 1
├─ Start Phase
│  ├─ Untap Step (untap your permanents)
│  ├─ Upkeep Step (triggers that say "at the start of turn")
│  └─ Draw Step (draw 1 card)
├─ Main Phase 1
│  ├─ Both players have priority in turn order
│  └─ Can play lands, creatures, spells
├─ Combat Phase
│  └─ Active player declares attackers, defender blocks
├─ Main Phase 2
│  └─ Can play spells or pass
└─ End Phase
   └─ "At end of turn" triggers fire
   
Turn 2 (Opponent)
└─ Same structure, but opponent is the active player
```

Priority is how Cardinal ensures fair turn structure:

1. **Active player** gets priority first
2. Can play spells, cast abilities, etc.
3. Passes priority to opponent
4. Opponent can respond
5. Once both players pass consecutively, the phase ends

---

## Zones: Where Cards Live

A **zone** is a place where cards can be:

| Zone | Purpose | Public/Hidden |
|------|---------|-------|
| **Library** | Your deck | Hidden (cards face-down) |
| **Hand** | Cards in your hand | Hidden (opponent can't see) |
| **Field** | Cards in play (creatures, enchantments) | Public (everyone sees) |
| **Graveyard** | Cards you've discarded or destroyed | Public (everyone sees) |
| **Stack** | Spells and abilities waiting to resolve | Public (everyone sees) |
| **Exile** | Cards removed from the game | Public (everyone sees) |

When you play a card:
- Remove it from **Hand**
- Add it to **Field**

If it dies:
- Remove it from **Field**
- Add it to **Graveyard**

---

## Actions: What Players Can Do

An **action** is a command the player sends to Cardinal. Examples:

```rust
// Play a card from hand
Action::PlayCard { 
  card: CardId(5),        // The card ID
  from: Zone::Hand,       // Where it came from
}

// Pass priority to the opponent
Action::PassPriority

// Pay mana and activate an ability
Action::ActivateAbility {
  card: CardId(3),
  ability_index: 0,
  target: SomeTarget,
}

// Declare attackers in combat
Action::DeclareAttackers {
  attackers: vec![CardId(1), CardId(2)],
}
```

Cardinal validates every action:
- Is it legal right now?
- Does the player have what they need?
- Is the order of actions sensible?

If the action is legal, it's applied. If not, an error is returned.

---

## Events: The Game's Narrative

An **event** is something that happened. The UI reads events to know what to show.

Examples:

```rust
Event::CardPlayed { 
  player: PlayerId(0),
  card: CardId(5),
}

Event::LifeChanged { 
  player: PlayerId(1),
  old_life: 20,
  new_life: 18,  // Took 2 damage
}

Event::CardMoved {
  card: CardId(3),
  from_zone: Zone::Hand,
  to_zone: Zone::Field,
}

Event::AbilityTriggered {
  card: CardId(1),
  ability: "etb_damage",
  effect: "damage 1 to opponent",
}

Event::StackResolved {
  effect: "deal 1 damage",
  target: "opponent",
  result: "2 damage dealt",  // Maybe it was modified
}
```

The UI might:
- Animate the card movement
- Update life totals
- Show a notification: "Ability triggered!"
- Play a sound effect

Cardinal doesn't care what the UI does. It just says "here's what happened."

---

## The Engine: Core Components

### GameEngine Struct
Holds the game state, the card registry, and the random number generator.

```rust
pub struct GameEngine {
  pub state: GameState,
  pub cards: CardRegistry,  // HashMap of card definitions
  rng: StdRng,              // Seeded RNG for reproducibility
}
```

### apply_action()
The main function. Takes an action, validates it, applies it, returns events.

```
apply_action(player, action)
  ├─ Validate legality
  ├─ Apply the action (mutate state)
  ├─ Evaluate triggers
  ├─ Emit events
  └─ Return StepResult { events, ... }
```

### Legality Module
Checks if an action is allowed. Examples:

- `can_play_card()` — Is it your turn? Is the stack empty? Do you have mana?
- `is_phase_legal()` — Are we in a phase where this is allowed?
- `owns_card()` — Is the card actually yours?

### Reducer Module
Applies an action to the state. Examples:

- `play_card_reducer()` — Move card from hand to field, subtract mana
- `draw_cards_reducer()` — Add cards from top of library to hand
- `take_damage_reducer()` — Reduce life total, emit LifeChanged event

### Triggers Module
Evaluates card abilities. Examples:

- On `CardMoved → Field`: Fire all "enters the battlefield" abilities
- On `CardPlayed`: Fire all "when you cast a spell" abilities
- On `PhaseChanged → EndPhase`: Fire all "at end of turn" abilities

---

## How Commands Work (Intermediate Concept)

When a card ability triggers, it doesn't directly mutate the state. Instead, it returns a **Command**:

```rust
pub enum Command {
  DealDamage { target: PlayerId, amount: u32 },
  DrawCards { player: PlayerId, count: u32 },
  GainLife { player: PlayerId, amount: u32 },
  PumpCreature { card: CardId, power: i32, toughness: i32 },
}
```

The **engine** then validates and applies the command:

```
Card ability fires
  ├─ Returns: Command::DealDamage { target: Player(1), amount: 1 }
  ├─ Engine validates: "Is the target legal?"
  ├─ Engine applies: reduce player 1's life by 1
  └─ Engine emits: Event::LifeChanged { ... }
```

Why have commands as an intermediate layer?

1. **Safety** — The engine validates before applying
2. **Auditability** — You can see what was requested and what happened
3. **Extensibility** — New commands can be added without rewriting the engine
4. **Scripting** — Mods/plugins can emit commands without direct state access

---

## How the CLI Works (Not Part of Cardinal Core)

The cardinal-cli binary is a separate program that:

1. **Uses** the Cardinal library (imports GameEngine, etc.)
2. **Creates a game** by calling `GameEngine::new()`
3. **Enters a loop**:
   - Render the game state to the terminal
   - Show what actions are legal
   - Read player input
   - Call `engine.apply_action(action)`
   - Process returned events
   - Update display

Here's the loop in pseudocode:

```
loop {
  render_game_state(engine.state)
  show_legal_actions(engine)
  input = read_player_input()
  result = engine.apply_action(player, input)
  process_events(result.events)
  update_log(result.events)
}
```

The CLI is **not** Cardinal itself. It's a client that **uses** Cardinal. You could replace it with a web UI, a mobile app, or an AI, and Cardinal would work exactly the same.

---

## Key Data Structures at a Glance

| Struct | Purpose |
|--------|---------|
| `GameEngine` | Main engine object; holds state, rules, RNG |
| `GameState` | Complete game state snapshot |
| `Player` | One player's data (life, mana, zones) |
| `Card` | A card instance in play (has an ID, owner, location) |
| `CardDef` | Definition of a card type (loaded from rules.toml) |
| `CardAbility` | A card's triggered or activated ability |
| `Action` | What a player wants to do |
| `Event` | What happened as a result |
| `Command` | An intermediate effect that needs applying |
| `PendingChoice` | "Engine is waiting for a player to choose something" |
| `StepResult` | Return value from `apply_action()`: events + state info |

---

## Testing Strategy

Each layer has tests:

**Unit Tests:**
- `legality.rs` — Test that invalid actions are rejected, valid ones pass
- `reducer.rs` — Test that effects are applied correctly
- `triggers.rs` — Test that triggers fire at the right time

**Integration Tests:**
- Full game flows: "Play a creature, opponent responds, creature enters field, trigger fires, damage is dealt"
- State consistency: "Game state is always valid after each action"
- Determinism: "Same seed → same outcome"

**All 19 tests currently passing** — covering triggers, initialization, card abilities, and more.

---

## Summary

Cardinal is a **game engine in the truest sense**: it validates moves, applies rules, and emits a stream of events describing what happened. It stays out of the UI, stays deterministic, and keeps game state as a single source of truth.

The key insight: **By separating the engine from the UI, we get reusability. By enforcing determinism, we get fairness. By keeping state centralized, we get clarity.**

