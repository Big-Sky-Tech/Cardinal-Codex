# Card Definitions

This directory contains individual card definition files for Cardinal Codex.

Cardinal supports **three methods** for loading cards:

## Loading Methods

### Method 1: Single cards.toml File

Create a single `cards.toml` file with multiple `[[cards]]` entries:

```toml
[[cards]]
id = "1"
name = "Goblin Scout"
card_type = "creature"
cost = "1R"

[[cards]]
id = "2"
name = "Fireball"
card_type = "spell"
cost = "2R"
```

Load it with:
```rust
use cardinal::{load_game_config, CardSource};
let sources = vec![CardSource::File("./cards.toml".into())];
let ruleset = load_game_config("./rules.toml", Some(sources))?;
```

### Method 2: Individual Files in Directory (This Directory)

Each card is defined in its own `.toml` file. Cards are automatically loaded by the engine when the game starts.

```toml
# cards/goblin_scout.toml
id = "unique_card_id"
name = "Card Name"
card_type = "creature"  # or "spell", "enchantment", etc.
cost = "2R"             # mana/resource cost
description = "Card text goes here."

# Optional: Card abilities
[[abilities]]
trigger = "etb"         # when this ability triggers
effect = "damage"       # what effect it has

[abilities.params]
amount = "2"
target = "opponent"

# Optional: Keywords (must be defined in rules.toml)
keywords = ["flying", "haste"]

# Optional: Stats (key-value pairs)
[stats]
power = "3"
toughness = "2"
```

**Examples in this directory:**
- `goblin_scout.toml` - A simple creature with an ETB ability
- `fireball.toml` - A damage spell
- `knight_of_valor.toml` - A creature that gains life on entry

**Loading from this directory:**

```bash
cargo run --bin cardinal-cli
```

Or in code:

```rust
use cardinal::load_game_config;

// Default: loads from cards/ directory automatically
let ruleset = load_game_config("./rules.toml", None)?;

// Explicit:
use cardinal::CardSource;
let sources = vec![CardSource::Directory("./cards".into())];
let ruleset = load_game_config("./rules.toml", Some(sources))?;
```

### Method 3: .ccpack Files

Cards can be packaged into compressed `.ccpack` files for distribution:

```bash
cardinal-cli build-pack ./cards ./my-cards.ccpack
```

Load from a pack:
```rust
use cardinal::{load_game_config, CardSource};
let sources = vec![CardSource::Pack("./my-cards.ccpack".into())];
let ruleset = load_game_config("./rules.toml", Some(sources))?;
```

See [PACK_SYSTEM.md](../PACK_SYSTEM.md) for more details.

## Combine Multiple Sources

You can load cards from all three methods simultaneously:

```rust
use cardinal::{load_game_config, CardSource};

let sources = vec![
    CardSource::File("./cards.toml".into()),
    CardSource::Directory("./cards".into()),
    CardSource::Pack("./expansion.ccpack".into()),
];

let ruleset = load_game_config("./rules.toml", Some(sources))?;
```
