# Cardinal Codex

A **headless, deterministic game engine** for trading card games (TCGs). Define your game rules in TOML, and Cardinal handles validation, state management, triggers, and event emission.

## Quick Links

- **[README_DETAILED.md](README_DETAILED.md)** — Complete guide (start here if you're new)
- **[ARCHITECTURE.md](ARCHITECTURE.md)** — Deep dive into design principles
- **[crates/cardinal/README.md](crates/cardinal/README.md)** — API documentation
- **[crates/cardinal-cli/README.md](crates/cardinal-cli/README.md)** — Interactive game guide

## Features

✅ **Fully Deterministic** — Same seed + actions = identical outcome  
✅ **Data-Driven Rules** — Define cards in TOML (no code changes)  
✅ **Headless** — Embed in any interface (web, mobile, terminal, AI)  
✅ **Event-Based** — Complete game log for replays and debugging  
✅ **Well-Tested** — 19 integration tests covering core systems  
✅ **Clean Architecture** — Clear separation of concerns  

## Getting Started

### Play the Game

```bash
cargo run --bin cardinal-cli
```

Interactive terminal game with colored output and real-time state rendering.

### Run Tests

```bash
cargo test
```

19 integration tests covering triggers, initialization, card abilities, and turn progression. All passing.

### Use in Your Project

```rust
use cardinal::{GameEngine, Action, PlayerId};

let engine = GameEngine::new_from_file("rules.toml", seed)?;
engine.start_game(deck_0, deck_1)?;

let result = engine.apply_action(player_id, action)?;
for event in &result.events {
    // Process event...
}
```

## What is Cardinal?

Cardinal is a **game engine referee**:

```
Player: "I want to play card #1"
         ↓
  Cardinal: Validates action, applies effects, evaluates triggers
         ↓
  Returns: Events describing what happened
         ↓
    UI: Reads events and updates display
```

Cardinal enforces:
- **Turn structure** — Phases, steps, priority rules
- **Zone management** — Hand, field, graveyard, stack, deck
- **Action validation** — Legality checks before applying
- **Card abilities** — Data-driven triggers and effects
- **State consistency** — GameState is the single source of truth

## Core Principles

### 1. Determinism
Same starting state + same actions + same seed = identical outcome every time.

### 2. Headless (No UI)
Cardinal has no knowledge of screens or graphics. Any interface can use it.

### 3. Actions In, Events Out
Simple, unidirectional interface. Player sends action → Cardinal emits events.

### 4. GameState is Authoritative
One struct holds all truth. Everything else is derived from it.

## Documentation

| Document | Audience | Content |
|----------|----------|---------|
| [README_DETAILED.md](README_DETAILED.md) | Everyone | Overview, concepts, quick start |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Developers | Design principles, game flow, data structures |
| [crates/cardinal/README.md](crates/cardinal/README.md) | API users | Usage guide, integration examples, concepts |
| [crates/cardinal-cli/README.md](crates/cardinal-cli/README.md) | Players | Terminal game guide, controls, examples |
| [crates/cardinal/explanation.md](crates/cardinal/explanation.md) | Code explorers | Design patterns, module layout, architecture |

## Project Structure

```
crates/
  cardinal/          — The game engine library
  cardinal-cli/      — Interactive terminal game

rules.toml          — Game definitions (cards, abilities, phases)
README_DETAILED.md  — Complete guide
ARCHITECTURE.md     — Design deep dive
```

## Example: Playing a Card

You play "Goblin Scout":

```
1. Validation
   ✓ Your turn?
   ✓ Main phase?
   ✓ Own the card?
   ✓ In your hand?
   ✓ Have mana?

2. Apply Effect
   - Remove from hand
   - Add to field
   - Subtract mana
   → Event: CardPlayed, CardMoved

3. Evaluate Triggers
   - "enters the battlefield" trigger matches
   → Command: "deal 1 damage"

4. Resolve Stack
   - Opponent takes 1 damage
   → Event: LifeChanged

5. Return to Caller
   - Here are all the events that happened
   - UI renders them
```

## Key Concepts

| Concept | Meaning |
|---------|---------|
| **Zone** | Where a card is (hand, field, graveyard, stack, etc.) |
| **Phase/Step** | Turn structure (start, main, combat, end) |
| **Priority** | Whose turn to act (determines action order) |
| **Trigger** | Card ability that fires on events |
| **Stack** | Where spells/abilities wait to resolve |
| **Event** | Something that happened (recorded for replay) |
| **Command** | Intermediate effect awaiting validation |

## Tests

All tests passing:

```bash
cargo test
```

**19 integration tests** covering:
- Game initialization
- Turn progression
- Action validation
- Card abilities & triggers
- State consistency
- Determinism

## Configuration

Edit [rules.toml](rules.toml) to:
- Define new cards
- Change mana costs
- Add card abilities
- Customize phases
- Change game constants

No code changes needed. Cardinal is 100% data-driven.

## Next Steps

1. **[README_DETAILED.md](README_DETAILED.md)** — Understand the system
2. **`cargo run --bin cardinal-cli`** — Play the game
3. **[crates/cardinal/README.md](crates/cardinal/README.md)** — Learn the API
4. **[ARCHITECTURE.md](ARCHITECTURE.md)** — Deep dive into design
5. **Edit rules.toml** — Customize your game

## Summary

Cardinal is a **clean, deterministic, reusable game engine**:

- **One engine** for any turn-based TCG
- **Many interfaces** (terminal, web, mobile, AI)
- **Data-driven rules** (TOML configuration)
- **Full determinism** (perfect replays)
- **Well-tested** (19 integration tests)

It's designed to be embedded, extended, and understood.
