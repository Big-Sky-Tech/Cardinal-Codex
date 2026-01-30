# Cardinal Codex

A **headless, deterministic game engine** for trading card games (TCGs). Define your game rules in TOML, and Cardinal handles validation, state management, triggers, and event emission.

## Quick Links

- **[README_DETAILED.md](README_DETAILED.md)** — Complete guide (start here if you're new)
- **[ARCHITECTURE.md](ARCHITECTURE.md)** — Deep dive into design principles
- **[crates/cardinal/README.md](crates/cardinal/README.md)** — API documentation
- **[crates/cardinal-cli/README.md](crates/cardinal-cli/README.md)** — Interactive game guide
- **[docs/CLI_USAGE_GUIDE.md](docs/CLI_USAGE_GUIDE.md)** — CLI tools for validation, compilation, and testing

## Features

✅ **Fully Deterministic** — Same seed + actions = identical outcome  
✅ **Data-Driven Rules** — Define cards in TOML (no code changes)  
✅ **Hybrid Card System** — TOML builtins + Rhai scripts for flexibility  
✅ **Headless** — Embed in any interface (web, mobile, terminal, AI)  
✅ **Event-Based** — Complete game log for replays and debugging  
✅ **Well-Tested** — 31 tests covering core systems and scripting  
✅ **Clean Architecture** — Clear separation of concerns  
✅ **Production-Ready Tooling** — Comprehensive validation, compilation, and testing tools  

## Getting Started

### Play the Game

```bash
cargo run --bin cardinal-cli
```

Interactive terminal game with colored output and real-time state rendering.

### Validate Your Game Assets

```bash
# Validate rules
cardinal-cli validate rules rules.toml

# Validate cards
cardinal-cli validate cards-dir cards/

# Validate an entire pack
cardinal-cli validate pack my-pack/
```

See [CLI_USAGE_GUIDE.md](docs/CLI_USAGE_GUIDE.md) for complete CLI documentation.

### Build Game Artifacts

```bash
# Compile a pack with validation
cardinal-cli compile pack my-pack/ output/game.ccpack --verbose

# Test the pack
cardinal-cli test pack output/game.ccpack
```

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
| [SCRIPTING_GUIDE.md](SCRIPTING_GUIDE.md) | Card designers | Rhai scripting for custom card effects |
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

**31 tests** covering:
- Game initialization
- Turn progression
- Action validation
- Card abilities & triggers
- Builtin effect execution
- Scripted effect execution
- State consistency
- Determinism

## Configuration

Edit [rules.toml](rules.toml) to:
- Define new cards (TOML builtin effects)
- Add scripted cards (Rhai scripts in `examples/scripts/`)
- Change mana costs
- Add card abilities
- Customize phases
- Change game constants

No code changes needed. Cardinal supports both TOML-only cards and Rhai-scripted cards. See [SCRIPTING_GUIDE.md](SCRIPTING_GUIDE.md) for details.

### VS Code TOML Validation

Cardinal includes JSON schemas for validating TOML files in VS Code:

1. **Install the "Even Better TOML" extension** (recommended automatically)
2. **Open any TOML file** — invalid fields or values are highlighted
3. **Get autocomplete** — Press Ctrl+Space for valid field suggestions
4. **See exact errors** — Hover over red squiggles for validation messages

Schemas validate:
- Individual card files (`cards/*.toml`)
- Card array files (`cards.toml`)
- Rules definitions (`rules.toml`)
- Pack metadata (`pack.toml`)

See [schemas/README.md](schemas/README.md) for details.

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
- **Hybrid card system** (TOML builtins + Rhai scripts)
- **Data-driven rules** (TOML configuration)
- **Full determinism** (perfect replays)
- **Well-tested** (31 tests passing)

It's designed to be embedded, extended, and understood.
