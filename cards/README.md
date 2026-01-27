# Card Definitions

This directory contains individual card definition files for Cardinal Codex.

## Structure

Each card is defined in its own `.toml` file. Cards are automatically loaded by the engine when the game starts.

## Card File Format

```toml
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

## Examples

See the existing `.toml` files in this directory for examples:
- `goblin_scout.toml` - A simple creature with an ETB ability
- `fireball.toml` - A damage spell
- `knight_of_valor.toml` - A creature that gains life on entry

## Loading Cards

Cards are automatically loaded from this directory when you run:

```bash
cargo run --bin cardinal-cli
```

Or in code:

```rust
use cardinal::load_game_config;

// Loads rules.toml and all .toml files from cards/
let ruleset = load_game_config("./rules.toml", None)?;
```

## Distribution

Cards can be packaged into `.ccpack` files for distribution:

```bash
cardinal-cli build-pack ./cards ./my-cards.ccpack
```

See [PACK_SYSTEM.md](../PACK_SYSTEM.md) for more details.
