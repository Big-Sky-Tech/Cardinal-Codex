# Test Game - A Minimal Cardinal Example

This is a fully functional, minimal example game built using the Cardinal engine. It demonstrates all the essential components needed to create a working TCG-like game.

## What's Included

This test game includes:

- ✅ **Game binary** - A runnable executable that initializes the Cardinal engine
- ✅ **Rules definition** - Complete game rules in `rules.toml`
- ✅ **Card definitions** - Example cards demonstrating different types
- ✅ **Project structure** - Shows how to organize a Cardinal-based game
- ✅ **Build configuration** - Cargo.toml with proper dependencies

## Quick Start

### Building the Game

From the repository root:

```bash
cargo build --bin test-game
```

### Running the Game

From the repository root:

```bash
cargo run --bin test-game
```

You should see output showing:
- Game initialization
- Rules and cards loaded
- Current game state (turn, phase, player info)
- Next steps for making it interactive

## Project Structure

```
test-game/
├── Cargo.toml           # Build configuration
├── README.md            # This file
├── rules.toml           # Game rules and basic cards
├── cards/               # Individual card definitions
│   ├── goblin_warrior.toml
│   ├── healing_potion.toml
│   └── fireball.toml
└── src/
    └── main.rs          # Game executable
```

## How It Works

### 1. Rules Configuration (`rules.toml`)

Defines the core game mechanics:
- Game metadata (name, version, description)
- Initial state (starting life, hand size, resources)
- Turn structure (phases and steps)
- Game zones (hand, deck, field, graveyard, stack)
- Basic card definitions

### 2. Card Definitions (`cards/*.toml`)

Individual card files demonstrating:
- **Creatures** - Cards with power/toughness stats
- **Spells** - Cards with instant effects
- **Abilities** - Card effects using builtin effects (damage, gain_life)

### 3. Game Binary (`src/main.rs`)

A minimal program that:
1. Loads game configuration
2. Creates initial game state
3. Populates test decks
4. Initializes the game engine
5. Displays the current state

## Customizing This Game

### Add More Cards

Create new `.toml` files in the `cards/` directory:

```toml
id = "my_card"
name = "My Card"
card_type = "Creature"
cost = "2"
description = "My custom card"

[stats]
power = "3"
toughness = "2"
```

### Modify Game Rules

Edit `rules.toml` to change:
- Starting life and resources
- Turn phases and steps
- Zone definitions
- Game constants

### Make It Interactive

The current implementation shows the initialized state. To make it playable:

1. **Add a game loop** - Process player input continuously
2. **Handle actions** - Implement handlers for playing cards, passing turns
3. **Process events** - Display what happens when actions are taken
4. **Add UI** - Create a display system (terminal, GUI, web)

See `crates/cardinal-cli/src/main.rs` for a complete interactive example.

## Using This as a Template

To create your own game based on this template:

1. **Copy the entire `test-game` folder**
   ```bash
   cp -r test-game my-game
   ```

2. **Update `Cargo.toml`**
   - Change the package name
   - Update the description

3. **Customize `rules.toml`**
   - Add your game's rules
   - Define your card mechanics
   - Set up your turn structure

4. **Create your cards**
   - Add card definitions in `cards/`
   - Or define them inline in `rules.toml`

5. **Build your game logic**
   - Extend `src/main.rs` with your game loop
   - Add UI rendering
   - Implement player interaction

## Key Concepts Demonstrated

### Cardinal Engine Initialization

```rust
let rules = cardinal::load_game_config(rules_path, None)?;
let initial_state = GameState::from_ruleset(&rules);
// Populate player decks and other zones on `initial_state` here; see src/main.rs for a full example.
let state = cardinal::initialize_game(initial_state, &rules, seed);
let engine = GameEngine::new(rules, seed, state);
```

### Game State Structure

- **Turn** - Turn number, current phase, step, active player
- **Players** - Life, resources, player-specific state
- **Zones** - Hand, deck, field, graveyard, stack (per player or shared)
- **Cards** - Card instances with unique IDs

### Determinism

The game uses a fixed seed (42) so the same sequence of actions will always produce the same result. This enables:
- Perfect replays
- Testing and debugging
- Network synchronization

## Next Steps

1. **Explore the Cardinal CLI** - See a full interactive game
   ```bash
   cargo run --bin cardinal-cli
   ```

2. **Read the documentation**
   - `README.md` - Project overview
   - `docs/README_DETAILED.md` - Complete guide
   - `BUILTIN_EFFECTS.md` - Available card effects
   - `docs/SCRIPTING_GUIDE.md` - Advanced card scripting

3. **Study the examples**
   - `examples/example-pack/` - Complete card pack
   - `examples/builtin_effects_showcase.toml` - Effect examples
   - `crates/cardinal-cli/` - Full interactive game

4. **Experiment**
   - Add more cards
   - Create custom effects
   - Build a UI
   - Add AI opponents

## Validation

Validate your game assets:

```bash
# Validate rules
cargo run --bin cardinal-cli -- validate rules test-game/rules.toml

# Validate individual cards
cargo run --bin cardinal-cli -- validate card test-game/cards/goblin_warrior.toml

# Validate all cards in directory
cargo run --bin cardinal-cli -- validate cards-dir test-game/cards/
```

## Why This Matters

This test game shows that Cardinal is:

- **Embeddable** - You can create a standalone game binary
- **Self-contained** - Everything needed is in one folder
- **Data-driven** - Game behavior defined in TOML, not code
- **Modular** - Cards, rules, and logic are separate
- **Production-ready** - This structure scales to real games

You can copy this folder, customize it, and have a working foundation for your own TCG!

## Support

- **Issue Tracker** - https://github.com/Big-Sky-Tech/Cardinal-Codex/issues
- **Documentation** - See `docs/` directory
- **Examples** - See `examples/` directory

---

**Cardinal Test Game** - A minimal but complete example of what's possible with the Cardinal engine.
