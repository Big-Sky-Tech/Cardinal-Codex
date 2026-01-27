# Card Definitions

This directory contains individual card definition files for Cardinal Codex.

Cardinal supports **three methods** for loading cards. For typical projects, use **either** `cards.toml` **or** the `cards/` directory:

- **Priority 1:** If `cards/` directory exists, it is used (and `cards.toml` is ignored)
- **Priority 2:** If `cards/` doesn't exist, `cards.toml` is used (if it exists)
- **Advanced:** You can explicitly combine sources using `CardSource`

**Script Organization:**
- When using **`cards.toml`**: Place scripts in a `scripts/` directory at the project root
- When using **`cards/` directory**: Place scripts in `cards/scripts/` subdirectory

## Loading Methods

### Method 1: Single cards.toml File

Create a single `cards.toml` file with multiple `[[cards]]` entries (in the parent directory, not this one). When using this method, **scripts must go in a `scripts/` directory**:

```
project/
├── rules.toml
├── cards.toml          # All cards in one file
└── scripts/            # Scripts go here when using cards.toml
    ├── fireball.rhai
    └── healing.rhai
```

```toml
# cards.toml (in project root)
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
script_path = "scripts/fireball.rhai"  # Reference script in scripts/ directory
```

**When to use:** Small projects with few cards, or when you prefer a single file.

**Note:** This method is **automatically ignored** if the `cards/` directory exists.

Load it explicitly:
```rust
use cardinal::{load_game_config, CardSource};
let sources = vec![CardSource::File("./cards.toml".into())];
let ruleset = load_game_config("./rules.toml", Some(sources))?;
```

### Method 2: Individual Files in Directory (This Directory) ⭐ Recommended

Each card is defined in its own `.toml` file in this directory. This is the **recommended approach** for most projects. When using this method, **scripts can go in a `cards/scripts/` subdirectory**:

```
project/
├── rules.toml
└── cards/              # This directory
    ├── goblin_scout.toml
    ├── fireball.toml
    ├── knight_of_valor.toml
    └── scripts/        # Scripts go here when using cards/ directory
        ├── fireball.rhai
        └── healing.rhai
```

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

## Combining Sources (Advanced)

For typical projects, use **either** `cards.toml` **or** `cards/` directory. However, you can explicitly combine sources for advanced scenarios like base set + expansion packs:

```rust
use cardinal::{load_game_config, CardSource};

// Base cards from directory + expansion pack
let sources = vec![
    CardSource::Directory("./cards".into()),
    CardSource::Pack("./expansion.ccpack".into()),
];

let ruleset = load_game_config("./rules.toml", Some(sources))?;
```

**Remember:** When using default loading (no explicit sources), the `cards/` directory takes priority over `cards.toml`.
