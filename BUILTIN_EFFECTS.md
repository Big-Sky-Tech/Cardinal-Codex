# Builtin TOML Effects Reference

This document describes all available builtin card effects that can be used in TOML card definitions without requiring any scripting.

## Overview

Cardinal supports a set of builtin effects that can be specified entirely in TOML using the `effect` and `params` fields. These effects allow game designers to create cards without writing any code.

**Note:** The `draw` effect is not yet fully implemented (returns empty commands). For complex effects or those not yet supported, use Rhai scripting instead (see [SCRIPTING_GUIDE.md](SCRIPTING_GUIDE.md)).

## Effect Format

Each card ability has two parts:
- `effect`: The name of the effect (e.g., "damage", "gain_life")
- `params`: A table of parameters specific to that effect

Example:
```toml
[[cards.abilities]]
trigger = "on_play"
effect = "damage"
[cards.abilities.params]
amount = "2"
```

## Available Effects

### Life & Damage Effects

#### `damage`
Deal damage to a player (reduces their life).

**Parameters:**
- `amount` (required): The amount of damage to deal

**Example:**
```toml
effect = "damage"
[params]
amount = "3"
```

#### `gain_life`
Restore life to a player.

**Parameters:**
- `amount` (required): The amount of life to gain

**Example:**
```toml
effect = "gain_life"
[params]
amount = "5"
```

#### `lose_life`
Lose life (similar to damage but doesn't trigger damage-related effects).

**Parameters:**
- `amount` (required): The amount of life to lose
- `player` (optional, default: controller): The player who loses life

**Example:**
```toml
effect = "lose_life"
[params]
amount = "3"
player = "0"
```

#### `set_life`
Set a player's life to a specific value.

**Parameters:**
- `amount` (required): The new life total
- `player` (optional, default: controller): The target player

**Example:**
```toml
effect = "set_life"
[params]
amount = "20"
player = "0"
```

### Card Draw

#### `draw`
Draw cards from the deck.

**Status:** ⚠️ Not yet fully implemented (currently returns empty commands)

**Parameters:**
- `amount` (required): The number of cards to draw

**Example:**
```toml
effect = "draw"
[params]
amount = "2"
```

### Card Movement

#### `move_card`
Move a card from one zone to another.

**Parameters:**
- `card` (optional, default: source card): The card to move
- `from_zone` (optional, default: "hand"): The origin zone
- `to_zone` (optional, default: "field"): The destination zone

**Example:**
```toml
effect = "move_card"
[params]
card = "0"
from_zone = "graveyard"
to_zone = "field"
```

### Creature Stat Modification

#### `set_stats`
Set a creature's power and toughness to specific values.

**Parameters:**
- `card` (optional, default: source card): The target card
- `power` (optional, default: 0): The new power value
- `toughness` (optional, default: 0): The new toughness value

**Example:**
```toml
effect = "set_stats"
[params]
card = "0"
power = "0"
toughness = "1"
```

### Keyword Abilities

#### `grant_keyword`
Grant a keyword ability to a card (e.g., "flying", "haste", "vigilance").

**Parameters:**
- `card` (optional, default: source card): The target card
- `keyword` (required): The keyword to grant

**Example:**
```toml
effect = "grant_keyword"
[params]
card = "0"
keyword = "flying"
```

#### `remove_keyword`
Remove a keyword ability from a card.

**Parameters:**
- `card` (optional, default: source card): The target card
- `keyword` (required): The keyword to remove

**Example:**
```toml
effect = "remove_keyword"
[params]
card = "0"
keyword = "flying"
```

### Resource Manipulation

#### `gain_resource`
Add resources to a player (e.g., mana, energy, action points).

**Parameters:**
- `resource` (optional, default: "mana"): The type of resource
- `amount` (required): The amount to gain (must be non-negative)
- `player` (optional, default: controller): The target player

**Example:**
```toml
effect = "gain_resource"
[params]
resource = "mana"
amount = "3"
player = "0"
```

#### `spend_resource`
Remove resources from a player.

**Parameters:**
- `resource` (optional, default: "mana"): The type of resource
- `amount` (required): The amount to spend (must be non-negative)
- `player` (optional, default: controller): The target player

**Example:**
```toml
effect = "spend_resource"
[params]
resource = "mana"
amount = "2"
player = "1"
```

#### `set_resource`
Set a player's resource to a specific value.

**Parameters:**
- `resource` (optional, default: "mana"): The type of resource
- `amount` (required): The new resource value (must be non-negative)
- `player` (optional, default: controller): The target player

**Example:**
```toml
effect = "set_resource"
[params]
resource = "mana"
amount = "10"
player = "0"
```

### Counter Manipulation

#### `add_counter`
Add counters to a card (e.g., +1/+1 counters, charge counters).

**Parameters:**
- `card` (optional, default: source card): The target card
- `counter_type` (optional, default: "+1/+1"): The type of counter
- `amount` (required): The number of counters to add (must be non-negative)

**Example:**
```toml
effect = "add_counter"
[params]
card = "0"
counter_type = "+1/+1"
amount = "2"
```

#### `remove_counter`
Remove counters from a card.

**Parameters:**
- `card` (optional, default: source card): The target card
- `counter_type` (optional, default: "+1/+1"): The type of counter
- `amount` (required): The number of counters to remove (must be non-negative)

**Example:**
```toml
effect = "remove_counter"
[params]
card = "0"
counter_type = "+1/+1"
amount = "1"
```

### Token Creation

#### `create_token`
Create a token card and place it in a zone.

**Parameters:**
- `player` (optional, default: controller): The player who owns the token
- `token_type` (optional, default: "1/1_soldier"): The type of token
- `zone` (optional, default: "field"): The zone to place the token

**Example:**
```toml
effect = "create_token"
[params]
player = "0"
token_type = "1/1_soldier"
zone = "field"
```

## Complete Card Examples

### Spell: Lightning Bolt
```toml
[[cards]]
id = "lightning_bolt"
name = "Lightning Bolt"
card_type = "spell"
cost = "1"
description = "Deal 3 damage to any target"

[[cards.abilities]]
trigger = "on_play"
effect = "damage"
[cards.abilities.params]
amount = "3"
```

### Creature with ETB Effect
```toml
[[cards]]
id = "healer_angel"
name = "Healer Angel"
card_type = "creature"
cost = "3"
description = "When this enters the battlefield, gain 3 life"

[[cards.abilities]]
trigger = "etb"
effect = "gain_life"
[cards.abilities.params]
amount = "3"

[cards.stats]
power = "2"
toughness = "2"
```

### Resource Generation
```toml
[[cards]]
id = "mana_ritual"
name = "Mana Ritual"
card_type = "spell"
cost = "1"
description = "Gain 3 mana"

[[cards.abilities]]
trigger = "on_play"
effect = "gain_resource"
[cards.abilities.params]
resource = "mana"
amount = "3"
```

## Notes

- All numeric parameters are specified as strings in TOML but are parsed as integers
- Player IDs are typically 0 (you) or 1 (opponent) in a two-player game
- Zone names must match those defined in your `rules.toml` file
- Card IDs referenced in parameters should be valid card IDs
- Parameters marked as "optional" have default values that will be used if omitted
- Negative amounts are not allowed for resource/counter operations to prevent unintended game states
- The draw effect is not yet fully implemented

## Scripted Effects

For effects not supported by builtins or for more complex behavior, you can use Rhai scripts. See [SCRIPTING_GUIDE.md](SCRIPTING_GUIDE.md) for details.

To use a scripted effect:
```toml
[[cards.abilities]]
trigger = "on_play"
effect = "script:my_script_name"
```

The script file should be specified in the card definition:
```toml
script_path = "path/to/my_script.rhai"
```
