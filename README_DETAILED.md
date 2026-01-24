# Cardinal Codex: A Complete Guide to the Game Engine

Welcome! This is **Cardinal** — a trading card game engine built in Rust.

If you're wondering "what is this project?" or "how do I use it?", you've found the right place. This guide explains **everything**, from high-level concepts to the implementation details.

## What is Cardinal?

**Cardinal is a game rules engine.** Think of it like the referee in a sporting event:

- **Players** make moves (play cards, attack, etc.)
- **Cardinal** validates those moves ("Is that legal?")
- **Cardinal** applies the effects ("OK, your creature entered play; does it trigger any abilities?")
- **Cardinal** tracks the game state (life totals, zones, whose turn it is)
- **Cardinal** emits events ("Here's what happened")

Cardinal is **headless** — it has no graphics, no UI, no sounds. It's just the logic. You provide the interface (terminal, web, mobile, etc.), and Cardinal provides the game rules.

### Why This Matters

Most TCG games couple the rules tightly with the UI. Change the rules? Rewrite the UI. Change the interface? Hope the rules still work. **Cardinal separates these concerns:**

```
User Interface       (web, mobile, desktop, terminal, AI)
         ↓
    Cardinal API     (apply_action → events)
         ↓
Cardinal Engine      (validation, logic, state)
         ↓
     Game State      (who has how much life, what cards are in play)
```

This means:
- **One engine, many interfaces** — The same Cardinal instance can power a web game, a mobile app, and an AI all at once
- **Rules are data, not code** — Define your game in TOML; no code changes needed
- **Deterministic** — Same inputs + seed = same outcome (replays, fairness, debugging all work perfectly)

---

## Quick Start

### Run the Interactive CLI

```bash
cargo run --bin cardinal-cli
```

You'll enter an interactive terminal game where you can play cards, pass priority, and see the game engine in action.

### Run the Tests

```bash
cargo test
```

**19 integration tests** covering triggers, state management, card abilities, turn progression, and more.

---

## Document Guide

### For Different Audiences

**"I just want to play the game"**
→ Read [crates/cardinal-cli/README.md](crates/cardinal-cli/README.md) for terminal gameplay instructions.

**"I want to understand the architecture"**
→ Read [ARCHITECTURE.md](ARCHITECTURE.md) for a deep dive into design principles and game flow.

**"I want to use Cardinal in my own project"**
→ Read [crates/cardinal/README.md](crates/cardinal/README.md) for API documentation and integration examples.

**"I want to modify the rules or add new cards"**
→ Edit [rules.toml](rules.toml) to define new cards and change game mechanics.

**"I want to understand the codebase"**
→ Start with [crates/cardinal/layout.md](crates/cardinal/layout.md) for the file structure, then explore [crates/cardinal/explanation.md](crates/cardinal/explanation.md) for detailed design patterns.

---

## The Four Core Principles

Cardinal is built on four immutable ideas:

### 1. **Determinism**

Same starting state + same actions + same random seed = identical game outcome every time.

**Why?**
- Replays work perfectly (show exactly what happened)
- Network games are fair (both players can verify)
- Debugging is possible (reproduce bugs exactly)

**How?**
- No system time, no threads, no external randomness
- All RNG comes from Cardinal's seeded generator

### 2. **Headless (No UI)**

Cardinal has zero knowledge of screens, buttons, or animations.

**Why?**
- It's embeddable (any frontend can use it)
- It's testable (no UI framework to mock)
- It's reusable (same engine, different interfaces)

**How?**
- Cardinal only deals with data (game state) and logic (rules)
- UI, input, networking live outside

### 3. **Actions In, Events Out**

Cardinal's interface is simple:

```
Player sends Action ("I want to play card #1")
              ↓
         Cardinal validates & applies
              ↓
    Cardinal emits Events ("Card was played, creature entered field, ability triggered, damage was dealt")
              ↓
        UI reads events & updates display
```

**Why?**
- Clear boundaries (you always know what's happening)
- Auditability (events are a complete game log)
- Extensibility (new UIs read the same events)

### 4. **GameState is Authoritative**

One struct holds all truth: `GameState`. Everything else is derived from it.

**Why?**
- No conflicts (single source of truth)
- Consistency (if you know the state, you know everything)
- Simplicity (no syncing, no race conditions)

---

## Game Concepts at a Glance

### Zones
Places where cards can be:
- **Hand** — Your hidden cards
- **Field** — Cards in play (creatures, enchantments)
- **Library** — Your deck
- **Graveyard** — Discarded cards
- **Stack** — Spells/abilities waiting to resolve
- **Exile** — Cards removed from play

### Turns & Phases
```
Turn 1
├─ Start (untap, upkeep, draw)
├─ Main 1 (play spells)
├─ Combat (attack/block)
├─ Main 2 (play spells)
└─ End (cleanup)
```

### Actions
What players can do:
- `PlayCard` — Play from hand
- `PassPriority` — Let opponent act
- `ActivateAbility` — Use a card ability
- `DeclareAttackers` — In combat
- `Concede` — Give up

### Events
What happened:
- `CardPlayed` — A card was played
- `CardMoved` — Card moved zones
- `LifeChanged` — Someone took/gained life
- `AbilityTriggered` — A card ability fired
- `StackResolved` — A spell resolved

### Triggers
Card abilities that fire automatically:
- `etb` — Enters the battlefield
- `on_play` — When cast
- `at_turn_start` — At start of your turn
- `at_turn_end` — At end of turn

---

## Project Structure

```
Cardinal-Codex/
├─ README.md (this file)
├─ ARCHITECTURE.md (deep dive into design)
├─ rules.toml (game definitions)
│
├─ crates/
│  ├─ cardinal/ (the game engine library)
│  │  ├─ src/
│  │  │  ├─ lib.rs (main exports)
│  │  │  ├─ engine/ (game logic)
│  │  │  │  ├─ core.rs (GameEngine struct)
│  │  │  │  ├─ reducer.rs (apply effects)
│  │  │  │  ├─ legality.rs (validate actions)
│  │  │  │  ├─ triggers.rs (card abilities)
│  │  │  │  └─ cards.rs (card registry)
│  │  │  ├─ model/ (data structures)
│  │  │  │  ├─ action.rs
│  │  │  │  ├─ event.rs
│  │  │  │  └─ command.rs
│  │  │  ├─ state/ (game state)
│  │  │  ├─ rules/ (rules schema from TOML)
│  │  │  ├─ display.rs (terminal rendering)
│  │  │  └─ ...
│  │  ├─ tests/
│  │  │  └─ integration.rs (19 integration tests)
│  │  ├─ README.md (full documentation)
│  │  └─ explanation.md (design patterns)
│  │
│  └─ cardinal-cli/ (interactive terminal game)
│     ├─ src/
│     │  └─ main.rs (terminal UI)
│     ├─ README.md (gameplay guide)
│     └─ Cargo.toml
│
└─ Cargo.toml (workspace config)
```

---

## How It Works: A Card Play Example

You play a card. Here's what happens internally:

```
STEP 1: You choose "play card #1" (Goblin Scout)
    ↓
STEP 2: CLI sends Action::PlayCard to Cardinal
    ↓
STEP 3: Cardinal.legality checks:
    ✓ Is it your turn?
    ✓ Is the game in Main Phase?
    ✓ Do you own the card?
    ✓ Is it in your hand?
    ✓ Do you have 1R mana?
    → LEGAL
    ↓
STEP 4: Cardinal.reducer applies the action:
    - Remove card from hand zone
    - Add card to field zone
    - Subtract mana
    - Emit CardRemoved & CardAdded events
    ↓
STEP 5: Cardinal.triggers evaluates:
    - Did the card enter the field? YES
    - Any "enters the battlefield" triggers? YES
    - Goblin Scout: "deal 1 damage to opponent"
    - Create Command::DealDamage { target: Opponent, amount: 1 }
    - Add to stack
    ↓
STEP 6: Cardinal.reducer applies the stack command:
    - Reduce opponent life: 20 → 19
    - Emit LifeChanged event
    ↓
STEP 7: Return events to CLI:
    [
        CardRemoved { card: 1, zone: Hand },
        CardAdded { card: 1, zone: Field },
        AbilityTriggered { card: 1, ability: "etb_damage" },
        LifeChanged { player: Opponent, old: 20, new: 19 },
        StackResolved { effect: "damage", amount: 1 },
    ]
    ↓
STEP 8: CLI displays:
    "You played: Goblin Scout"
    "Goblin Scout entered the field"
    "Ability triggered: deal 1 damage"
    "Opponent took 1 damage"
    (Updates opponent life from 20 to 19)
```

That's one complete player action. The loop repeats for each decision.

---

## Key Features

### ✅ Fully Deterministic
- Same seed + actions = same outcome
- Perfect for replays and debugging
- Network games can be verified

### ✅ Data-Driven Rules
- Define cards in TOML (no code changes)
- Easy to mod and customize
- Future plugin/script support

### ✅ Event-Based
- Complete game log (every action is recorded)
- UI updates from events (no query overhead)
- Extensible (new event types can be added)

### ✅ Headless Design
- Embeddable in any project
- Testable without a UI framework
- One engine, many interfaces

### ✅ Well-Tested
- 19 integration tests (all passing)
- Tests cover triggers, initialization, card abilities, turn progression
- High confidence in core logic

### ✅ Clean Architecture
- Separation of concerns (engine, rules, state, model)
- Clear public API
- Easy to understand and modify

---

## Using Cardinal

### In the Terminal
```bash
cargo run --bin cardinal-cli
```

### In Your Own Rust Project
```toml
[dependencies]
cardinal = { path = "../path/to/cardinal/crates/cardinal" }
```

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

### In a Web/Mobile App
Same Cardinal library, different UI (JavaScript/TypeScript, Swift, Kotlin, etc.).

---

## Testing

Run all tests:
```bash
cargo test
```

Run specific test:
```bash
cargo test test_card_ability_etb_trigger
```

Tests cover:
- ✅ Game initialization (deck shuffling, hand drawing, first player)
- ✅ Turn progression (phase/step advancement)
- ✅ Action validation (legality checks)
- ✅ Card abilities (trigger evaluation)
- ✅ State consistency
- ✅ Determinism

All **19 tests pass**.

---

## Configuration

Modify [rules.toml](rules.toml) to:
- Define new cards
- Change mana costs
- Add new card abilities
- Customize phases/steps
- Change game constants

Example: Add a new card

```toml
[[cards]]
id = 6
name = "Dragon"
type = "creature"
cost = "4RR"
power = 5
toughness = 5
description = "A mighty dragon."

[[cards.abilities]]
trigger = "etb"
effect = "damage"
value = 3
target = "all_opponents"
```

Then run:
```bash
cargo run --bin cardinal-cli
```

The new card is in play (no code recompile needed, just data).

---

## Learning Path

1. **Play the game** — Run the CLI, see it in action
   ```bash
   cargo run --bin cardinal-cli
   ```

2. **Read ARCHITECTURE.md** — Understand the design principles

3. **Read crates/cardinal/README.md** — Learn the API and concepts

4. **Explore the code** — Start with `crates/cardinal/src/engine/core.rs`

5. **Modify rules.toml** — Add new cards and see them work

6. **Build your own UI** — Use Cardinal in your project

---

## Core Concepts Summary

| Concept | What | Why |
|---------|------|-----|
| `GameState` | Complete game snapshot | Single source of truth |
| `Action` | What a player wants to do | Clear input interface |
| `Event` | What happened | Clear output interface |
| `Zone` | Where a card is (hand, field, etc.) | Organizes gameplay |
| `Phase/Step` | Turn structure | Ensures fair turn order |
| `Priority` | Whose turn to act | Prevents simultaneous decisions |
| `Trigger` | Card ability that fires on events | Data-driven effects |
| `Command` | Intermediate effect awaiting validation | Safety and extensibility |
| `CardRegistry` | All card definitions (TOML data) | Fast lookups |

---

## Design Philosophy

Cardinal is built on **clarity, determinism, and reusability**:

- **Clarity** — Code reads like rules, not magic. Explicit > implicit.
- **Determinism** — Same setup → same outcome. Every time.
- **Reusability** — One engine, many interfaces.

These principles are non-negotiable. Any change that violates them needs strong justification.

---

## Common Questions

**Q: Can I play this online?**  
A: Not yet. But Cardinal's determinism makes it perfect for network games. You can record actions on both sides and verify the outcome.

**Q: Can I add my own cards without touching code?**  
A: Yes! Edit `rules.toml` and run the CLI. Cardinal is 100% data-driven.

**Q: Is this like Magic: The Gathering?**  
A: It's inspired by Magic's structure (phases, priority, stack, zones) but is generic. You can define any turn-based TCG.

**Q: Can I use this for a board game?**  
A: Yes! Cardinal's architecture works for any game with state + actions + events.

**Q: Is this production-ready?**  
A: It's a solid foundation with clean architecture and good test coverage. Extend it for your needs.

---

## Contributing & Modifying

The codebase is intentionally conservative and readable. When adding features:

1. **Keep it explicit** — No clever abstractions. Clarity > brevity.
2. **Don't break determinism** — No system time, threads, or external randomness.
3. **Stay headless** — No rendering, UI, or engine-specific assumptions.
4. **Write tests** — Especially for new actions and triggers.
5. **Document it** — Explain the why, not just the what.

---

## File Organization

```
📄 README.md ........................ This file (overview)
📄 ARCHITECTURE.md .................. Deep dive into design
📄 rules.toml ....................... Game rules (TOML data)

📁 crates/cardinal/ ................. The game engine library
   📄 README.md ..................... Full Cardinal documentation
   📄 explanation.md ................ Design patterns & structure
   📄 layout.md ..................... File organization
   📁 src/
      📄 lib.rs ..................... Main exports
      📁 engine/ .................... Game logic
      📁 model/ ..................... Data structures
      📁 state/ ..................... Game state
      📁 rules/ ..................... Rules & schemas
      📄 display.rs ................. Terminal rendering
   📁 tests/
      📄 integration.rs ............. 19 integration tests

📁 crates/cardinal-cli/ ............. Interactive terminal game
   📄 README.md ..................... Gameplay guide
   📁 src/
      📄 main.rs .................... Terminal UI
```

---

## Next Steps

- **Run the game** — `cargo run --bin cardinal-cli`
- **Read ARCHITECTURE.md** — Understand the design
- **Read crates/cardinal/README.md** — Learn the API
- **Modify rules.toml** — Add new cards
- **Run tests** — `cargo test`
- **Explore the code** — Start with `engine/core.rs`

Welcome to Cardinal! 🎮

