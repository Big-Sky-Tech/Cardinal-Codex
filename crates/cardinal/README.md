# Cardinal Library: Complete Guide

Welcome to **Cardinal** — the game engine library that powers TCG (trading card game) logic.

## What Does Cardinal Do?

Imagine you're building a trading card game (like Magic: The Gathering or Yu-Gi-Oh). You need:

- **Game state management** — tracking whose turn it is, what cards are in play, who has how much life
- **Rule validation** — checking if an action is legal before applying it
- **Effect execution** — running card abilities, applying damage, drawing cards
- **Event tracking** — recording what happened so the UI can show it to players

Cardinal **handles all of this**. You provide:
- A TOML file describing your game rules
- User input (which card to play, when to pass priority, etc.)

Cardinal gives you back:
- The updated game state
- A list of events describing what happened

## Using Cardinal: The Basic Loop

Here's how any game that uses Cardinal works:

```python
# 1. Create the engine
engine = CardinalEngine.new(rules_file="rules.toml", seed=12345)

# 2. Initialize the game
engine.start_game(player1_deck, player2_deck)

# 3. Game loop
while game_is_running:
    # Show the current state to the player
    display(engine.state)
    
    # Get their action (e.g., "play card #5")
    action = input("What do you do?")
    
    # Apply the action; get back events
    result = engine.apply_action(player_id, action)
    
    # Show what happened
    for event in result.events:
        print_event(event)
```

That's it. Cardinal handles the complexity; you handle the UI.

## Cardinal's Four Core Principles

### 1. Determinism
**Same game setup + same actions + same random seed = identical outcome.**

Why does this matter?
- **Replays** — You can record every move and replay the entire game perfectly
- **Network fairness** — Both players can run the engine on their machine and verify they got the same result
- **Debugging** — If a bug occurs, you can recreate it exactly by replaying

Example:
```
Seed: 42
Player 1 actions: [PlayCard(1), PassPriority, PlayCard(3), ...]
Player 2 actions: [PassPriority, PlayCard(2), ...]

Result: Player 1 wins with 3 life remaining

---

Run it again with the same seed and actions:
Player 1 still wins with exactly 3 life remaining. Every time.
```

### 2. Headless (No UI)
Cardinal has **no idea what a screen is**. It doesn't render anything. This is by design.

Why?
- **Reusable** — The same Cardinal engine can power a web game, desktop app, mobile game, Discord bot, or AI
- **Testable** — No UI framework to mock or deal with
- **Clean** — Game logic stays separate from presentation

The role of Cardinal:
- Take an action → validate it → apply it → emit events

The role of the UI:
- Take those events → render them → show animations/sounds

### 3. Actions In, Events Out
Cardinal's interface is **simple and unidirectional**:

```
┌─────────────────┐
│  Player/AI      │  Sends an action: "I want to play card #5"
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Cardinal       │  1. Validates: "Is this legal right now?"
│  Engine         │  2. Applies: "OK, moving card to field"
│                 │  3. Triggers: "Does this trigger any abilities?"
│                 │  4. Emits: "Here's what happened..."
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Events         │  [CardPlayed, CardMoved, AbilityTriggered, ...]
│  (what changed) │  
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  UI/Client      │  Reads events and updates the display
└─────────────────┘
```

This is **one-way communication**. The client doesn't directly query state; it listens to events. This keeps Cardinal decoupled from its consumers.

### 4. GameState is Authoritative
There is **one source of truth**: the `GameState` struct inside Cardinal.

Why?
- **No conflicts** — If two systems disagree about whose turn it is, GameState is the arbiter
- **Consistency** — Everything you need to know can be queried from the state
- **Reproducibility** — You can save and load the state at any point

```
GameState = {
  turn: 1,
  phase: "main",
  step: "untap",
  players: [Player { life: 20, }, Player { life: 18, }],
  zones: {
    hand[0]: [Card, Card, Card],
    field[0]: [Card],
    field[1]: [Card, Card],
    library[0]: [...],
    graveyard[0]: [...],
  },
  stack: [],
  ...
}
```

If you want to know "Can player 0 play a card right now?", you check:
- Is it their turn? (check `turn.active_player`)
- Is the game in a phase where playing is allowed? (check `turn.phase`)
- Do they have the card in hand? (check `zones.hand[0]`)
- Do they have enough mana? (check `players[0].mana`)

All answers come from one place: the state.

---

## Game Structure: Turns, Phases, Steps

A game follows a rigid sequence. This prevents chaos and ensures fairness.

```
Turn 1 (Player 0 is active)
├─ Start Phase
│  ├─ Untap Step: Untap all your permanents
│  ├─ Upkeep Step: Abilities that trigger "at the start of your turn" fire
│  └─ Draw Step: Draw 1 card
├─ Main Phase 1
│  ├─ Player 0 has priority (can play spells)
│  ├─ Player 1 can respond
│  └─ Continue until both pass consecutively
├─ Combat Phase
│  ├─ Player 0 declares which creatures attack
│  ├─ Player 1 declares blockers
│  └─ Damage is assigned
├─ Main Phase 2
│  ├─ Player 0 has priority again
│  └─ Can play more spells
└─ End Phase
   ├─ Abilities that trigger "at the end of the turn" fire
   └─ Cleanup

Turn 2 (Player 1 is active)
└─ Same structure, but Player 1 is now the active player
```

**Priority** is how fairness is enforced:

1. Player 0 has priority → can play spells
2. Player 0 passes priority to Player 1
3. Player 1 can respond with their own spells
4. Player 1 passes back to Player 0
5. Once both players pass consecutively → phase ends

This ensures no one player can spam actions without giving the other a chance to respond.

---

## How Cards Work

### Card Definitions (Static Data)

In `rules.toml`, you define a card **once**:

```toml
[[cards]]
id = 1
name = "Goblin Scout"
type = "creature"
cost = "1R"          # Cost: 1 generic mana + 1 red mana
description = "A small but feisty goblin."
power = 1
toughness = 1

[[cards.abilities]]
trigger = "etb"      # "enters the battlefield"
effect = "damage"    # type of effect
value = 1            # amount of damage
target = "opponent"  # who gets hit
```

### Card Execution (Data-Driven)

When a player plays this card:

```
Step 1: Player plays card #1
Step 2: Cardinal looks up card #1 in the registry → finds "Goblin Scout"
Step 3: Cardinal moves card from hand to field
Step 4: Cardinal checks: does this trigger any abilities?
        → Yes! "etb" trigger matches
Step 5: Cardinal creates a command: "Deal 1 damage to opponent"
Step 6: Command is added to the stack
Step 7: Stack resolves: 1 damage is dealt
Step 8: Events emitted: CardPlayed, CardMoved, AbilityTriggered, LifeChanged
```

**Key insight:** Cardinal **never hardcodes** card effects. All effects are defined in data (TOML). This means:
- You can create new cards without touching code
- You can customize the rule set per game
- Mods and plugins become possible

---

## Zones: Where Cards Live

Every card in the game is in exactly one **zone**:

| Zone | What is it? | Public/Hidden | What can happen here? |
|------|-----------|------|------|
| **Library** | Your deck | Hidden | Cards are drawn from the top |
| **Hand** | Cards in your possession | Hidden (opponent can't see) | You play cards from here |
| **Field** | Cards in play | Public | Creatures attack, enchantments apply effects |
| **Graveyard** | Discard pile | Public | Cards that have been destroyed or discarded |
| **Stack** | Spells/abilities waiting to resolve | Public | Items wait in order, then resolve one by one |
| **Exile** | Cards removed from the game | Public | Typically can't be brought back |

Example: Playing a card

```
Before:  Hand[0] = [Goblin Scout, Knight of Valor, ...]
         Field[0] = []

Player plays Goblin Scout

After:   Hand[0] = [Knight of Valor, ...]
         Field[0] = [Goblin Scout]
```

The card moved from one zone to another. This triggers events and potentially card abilities.

---

## Actions: What Players Can Do

An **action** is what a player tells Cardinal to do. Examples:

```rust
// Play a card from your hand
PlayCard { 
  card_id: 1,           // which card (Goblin Scout)
  from_zone: Hand,      // where it came from
}

// Pass priority to the opponent
PassPriority

// Activate a card ability
ActivateAbility {
  card_id: 3,           // which card
  ability_index: 0,     // which ability on that card
  target: Opponent,     // who it targets
  mana_paid: "RR",      // mana spent to activate
}

// In combat: declare which creatures attack
DeclareAttackers {
  attackers: [1, 2, 5], // creature IDs
}

// In combat: declare which creatures block
DeclareBlockers {
  blockers: [3],        // creature ID
  blocking: {3: 1},     // card 3 blocks card 1
}

// Concede (give up)
Concede
```

Cardinal **validates every action**:

- Is it your turn?
- Is the game in a phase where this is allowed?
- Do you own the card?
- Do you have enough mana?
- Is the target legal?

If validation fails, an error is returned. Otherwise, the action is applied.

---

## Events: What Happened

An **event** describes something that happened in the game. The UI reads events to know what to show.

Examples:

```rust
// A card was played
CardPlayed {
  player: PlayerId(0),
  card: CardId(1),      // Goblin Scout
}

// A card moved from one zone to another
CardMoved {
  card: CardId(1),
  from_zone: Hand,
  to_zone: Field,
}

// A creature entered the field (triggers abilities)
CreatureEntered {
  card: CardId(1),      // Goblin Scout
  controller: PlayerId(0),
}

// An ability triggered
AbilityTriggered {
  card: CardId(1),
  ability: "etb_damage",
  effect: "deal 1 damage",
}

// A life total changed
LifeChanged {
  player: PlayerId(1),
  old_life: 20,
  new_life: 19,        // Took 1 damage
}

// Stack item resolved
StackResolved {
  item: "deal 1 damage to opponent",
  result: "opponent lost 1 life",
}

// Priority passed
PriorityPassed {
  from: PlayerId(0),
  to: PlayerId(1),
}

// Phase advanced
PhaseChanged {
  old_phase: "main",
  new_phase: "end",
}
```

A typical UI might:
- Animate card movement when it sees `CardMoved`
- Update the life counter when it sees `LifeChanged`
- Play a sound effect when it sees `AbilityTriggered`
- Show a notification when it sees `PriorityPassed`

Cardinal doesn't care what the UI does. It just says "here's what happened."

---

## Commands: The Intermediate Layer

When a card ability triggers, it doesn't directly change the game state. Instead, it emits a **command** that the engine validates and applies.

Why have this intermediate layer?

```
Card says: "Deal 1 damage"
         ↓
   Returns Command::DealDamage { target: Opponent, amount: 1 }
         ↓
   Engine validates: "Is the target valid? Do they exist?"
         ↓
   Engine applies: Reduce opponent's life by 1
         ↓
   Engine emits: Event::LifeChanged { old_life: 20, new_life: 19 }
```

**Benefits:**
1. **Safety** — Validation happens before mutation
2. **Auditability** — You can see what was requested and what was applied
3. **Extensibility** — New command types can be added without rewriting the engine
4. **Scripting** — Future mod/plugin systems can emit commands without direct state access

---

## The Trigger System: Reactive Logic

Triggers are how card abilities fire in response to events.

### Trigger Types

```toml
# Trigger on entry
[[cards.abilities]]
trigger = "etb"        # "enters the battlefield"

# Trigger when played (cast)
[[cards.abilities]]
trigger = "on_play"    # "when you cast this spell"

# Trigger at specific times
[[cards.abilities]]
trigger = "at_turn_start"    # "at the start of your turn"
trigger = "at_turn_end"      # "at the end of your turn"

# Trigger on events
[[cards.abilities]]
trigger = "when_creature_dies"   # "when a creature dies"
trigger = "when_damage_dealt"    # "when damage is dealt"
```

### How Triggers Work

```
Event: CardPlayed { card: CardId(1) }
         ↓
   Engine checks all cards:
   "Does any card have an on_play trigger?"
         ↓
   Card 1: "Inspiration" has on_play trigger
         ↓
   Fire the trigger:
   Create Command::DrawCards { count: 1 }
         ↓
   Push to stack
         ↓
   Stack resolves:
   Player draws 1 card
         ↓
   Emit: Event::CardDrawn { player, count: 1 }
```

This is **data-driven**. No hardcoded logic for each card. The engine is generic; cards define their behavior.

---

## Integration Example: Playing a Card

Let's trace through what happens when you play a card:

```
Player says: "I want to play Goblin Scout (card #1) from my hand"

STEP 1: VALIDATION
└─ Is it your turn? YES
└─ Is the game in Main Phase? YES
└─ Do you own card #1? YES
└─ Is card #1 in your hand? YES
└─ Do you have 1 generic + 1 red mana? YES
└─ Decision: LEGAL ✓

STEP 2: EFFECT APPLICATION
└─ Remove card #1 from your hand
└─ Add card #1 to your field
└─ Subtract mana from your pool (1 generic, 1 red)
└─ Emit: Event::CardRemoved { card: 1, zone: Hand }
└─ Emit: Event::CardAdded { card: 1, zone: Field }

STEP 3: TRIGGER EVALUATION
└─ Event: CardMoved { from: Hand, to: Field }
└─ Check all cards: "Do any have an 'enters the field' trigger?"
└─ Goblin Scout has etb trigger: "deal 1 damage to opponent"
└─ Create Command::DealDamage { target: Opponent, amount: 1 }
└─ Add to stack

STEP 4: STACK RESOLUTION
└─ Stack has 1 item: DealDamage
└─ Resolve it: Subtract 1 from opponent's life (20 → 19)
└─ Emit: Event::LifeChanged { player: Opponent, old: 20, new: 19 }
└─ Remove from stack

STEP 5: RETURN EVENTS
└─ Return to player:
   [
     CardRemoved { card: 1, zone: Hand },
     CardAdded { card: 1, zone: Field },
     AbilityTriggered { card: 1, ability: etb_damage },
     LifeChanged { player: Opponent, old: 20, new: 19 },
     StackResolved { effect: DealDamage, amount: 1 },
   ]

UI reads events:
└─ CardRemoved/CardAdded → Animate card moving from hand to field
└─ AbilityTriggered → Show "Goblin Scout's ability triggered!"
└─ LifeChanged → Update opponent's life counter to 19
└─ StackResolved → Log "1 damage dealt to opponent"
```

That's one complete action. The loop repeats for each player action.

---

## Testing

Cardinal has comprehensive tests:

**19 Integration Tests** covering:
- Game initialization (decks, hand drawing, first player)
- Turn progression (phase/step advancement)
- Action legality (validation rules)
- Card abilities (triggers, effects)
- Determinism (same seed → same outcome)

Run tests:
```bash
cargo test
```

Each test is a small game scenario:
```rust
#[test]
fn test_card_ability_etb_trigger() {
    // Setup: Create a game with a card that deals damage on ETB
    let engine = create_test_game();
    
    // Action: Play the card
    engine.apply_action(player_0, Action::PlayCard { ... });
    
    // Assertion: Opponent took damage
    assert_eq!(engine.state.players[1].life, 19);
}
```

---

## File Organization

```
crates/cardinal/src/
  lib.rs                 # Main library exports
  
  error.rs               # Error types
  ids.rs                 # NewType IDs (PlayerId, CardId, etc.)
  
  state/
    mod.rs               # State module exports
    gamestate.rs         # The GameState struct (complete game snapshot)
    zones.rs             # Zone management (hand, field, graveyard, etc.)
  
  rules/
    mod.rs               # Rules module exports
    schema.rs            # CardDef, CardAbility (data from TOML)
  
  engine/
    mod.rs               # Engine module exports
    core.rs              # GameEngine struct and main apply_action()
    reducer.rs           # Apply effects to state
    legality.rs          # Validate actions
    triggers.rs          # Evaluate triggered abilities
    cards.rs             # CardRegistry (lookup cards by ID)
  
  model/
    mod.rs               # Model module exports
    action.rs            # What players can do
    event.rs             # What happened
    command.rs           # Intermediate effects
    choice.rs            # Player input needed (pending choices)
  
  display.rs             # Terminal UI rendering (colors, formatting)
  
  util/
    rng.rs               # Random number generator (seeded for determinism)
```

---

## Key Concepts Summary

| Concept | What | Why |
|---------|------|-----|
| **GameState** | The complete game snapshot | Single source of truth |
| **Action** | What a player wants to do | Clear input interface |
| **Event** | What happened | Clear output interface |
| **Command** | Intermediate effect awaiting validation | Safety and auditability |
| **Trigger** | Card ability that fires in response to events | Data-driven card logic |
| **Zone** | Where a card is (hand, field, graveyard, etc.) | Organizes game structure |
| **Priority** | Whose turn to act | Ensures fairness |
| **Phase/Step** | What part of the turn are we in | Rigid structure prevents chaos |
| **CardRegistry** | HashMap of card definitions | O(1) card lookups |
| **Determinism** | Same inputs + seed = same outputs | Replays, fairness, debugging |

---

## Using Cardinal in Your Project

### 1. Add to Cargo.toml
```toml
[dependencies]
cardinal = { path = "../../crates/cardinal" }
```

### 2. Create a rules.toml
Define your game:
```toml
[game]
name = "My Cool TCG"

[[phases]]
name = "start"
steps = ["untap", "upkeep", "draw"]

[[phases]]
name = "main"

[[phases]]
name = "combat"

[[phases]]
name = "end"

[[zones]]
name = "hand"
visible_to = "owner"

[[zones]]
name = "field"
visible_to = "all"

[[cards]]
id = 1
name = "Goblin Scout"
type = "creature"
cost = "1R"
# ... more cards
```

### 3. Initialize and Run
```rust
use cardinal::{GameEngine, Action, PlayerId};

let engine = GameEngine::new_from_file("rules.toml", seed)?;
engine.start_game(deck_0, deck_1)?;

loop {
    let action = get_player_input();
    let result = engine.apply_action(player_id, action)?;
    
    for event in &result.events {
        display_event(event);
    }
}
```

---

## Next Steps

- Read [ARCHITECTURE.md](ARCHITECTURE.md) for a deeper dive into design
- Check [../cardinal-cli/](../cardinal-cli/) for a working example
- Run tests: `cargo test`
- Explore the code: `crates/cardinal/src/engine/core.rs` is the entry point

Cardinal is designed to be **clear and extensible**. Questions? The code is well-commented.

