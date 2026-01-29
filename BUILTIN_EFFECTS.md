# Builtin TOML Effects Reference

This document describes all available builtin card effects that can be used in TOML card definitions without requiring any scripting.

## Overview

Cardinal supports a comprehensive set of builtin effects that can be specified entirely in TOML using the `effect` and `params` fields. These effects mirror the capabilities of the Rhai scripting system, allowing game designers to create complex cards without writing any code.

## Effect Format

Each card ability has two parts:
- `effect`: The name of the effect (e.g., "damage", "gain_life", "draw")
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
- `player` (optional): The player who loses life (default: controller)

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
- `player` (optional): The target player (default: controller)

**Example:**
```toml
effect = "set_life"
[params]
amount = "20"
player = "0"
```

### Card Draw & Zone Manipulation

#### `draw`
Draw cards from the deck.

**Parameters:**
- `amount` (required): The number of cards to draw

**Example:**
```toml
effect = "draw"
[params]
amount = "2"
```

#### `mill_cards`
Move cards from the top of a player's deck to their graveyard.

**Parameters:**
- `count` (required): The number of cards to mill
- `player` (optional): The target player (default: controller)

**Example:**
```toml
effect = "mill_cards"
[params]
count = "3"
player = "1"
```

#### `discard_cards`
Move cards from a player's hand to their graveyard.

**Parameters:**
- `count` (required): The number of cards to discard
- `player` (optional): The target player (default: controller)

**Example:**
```toml
effect = "discard_cards"
[params]
count = "2"
player = "1"
```

#### `move_card`
Move a card from one zone to another.

**Parameters:**
- `card` (optional): The card to move (default: source card)
- `from_zone` (required): The origin zone (e.g., "hand", "graveyard", "field")
- `to_zone` (required): The destination zone (e.g., "field", "hand", "graveyard")

**Example:**
```toml
effect = "move_card"
[params]
card = "0"
from_zone = "graveyard"
to_zone = "field"
```

### Creature Stat Modification

#### `pump`
Temporarily modify a creature's power and toughness (until end of turn).

**Parameters:**
- `power` (required): The power modifier (can be negative)
- `toughness` (required): The toughness modifier (can be negative)

**Example:**
```toml
effect = "pump"
[params]
power = "3"
toughness = "3"
```

#### `set_stats`
Set a creature's power and toughness to specific values.

**Parameters:**
- `card` (optional): The target card (default: source card)
- `power` (required): The new power value
- `toughness` (required): The new toughness value

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
- `card` (optional): The target card (default: source card)
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
- `card` (optional): The target card (default: source card)
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
- `resource` (required): The type of resource (e.g., "mana")
- `amount` (required): The amount to gain
- `player` (optional): The target player (default: controller)

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
- `resource` (required): The type of resource (e.g., "mana")
- `amount` (required): The amount to spend
- `player` (optional): The target player (default: controller)

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
- `resource` (required): The type of resource (e.g., "mana")
- `amount` (required): The new resource value
- `player` (optional): The target player (default: controller)

**Example:**
```toml
effect = "set_resource"
[params]
resource = "mana"
amount = "0"
player = "0"
```

### Counter Manipulation

#### `add_counter`
Add counters to a card (e.g., +1/+1 counters, charge counters).

**Parameters:**
- `card` (optional): The target card (default: source card)
- `counter_type` (required): The type of counter (e.g., "+1/+1", "charge")
- `amount` (required): The number of counters to add

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
- `card` (optional): The target card (default: source card)
- `counter_type` (required): The type of counter (e.g., "+1/+1", "charge")
- `amount` (required): The number of counters to remove

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
- `player` (optional): The player who owns the token (default: controller)
- `token_type` (required): The type of token (e.g., "1/1_soldier", "2/2_bear")
- `zone` (required): The zone to place the token (e.g., "field", "hand")

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
id = "wall_of_omens"
name = "Wall of Omens"
card_type = "creature"
cost = "2"
description = "When Wall of Omens enters the battlefield, draw a card"

[[cards.abilities]]
trigger = "etb"
effect = "draw"
[cards.abilities.params]
amount = "1"

[cards.stats]
power = "0"
toughness = "4"
```

### Complex Multi-Effect Card (using multiple abilities)
```toml
[[cards]]
id = "ancestral_vision"
name = "Ancestral Vision"
card_type = "spell"
cost = "3"
description = "Draw 3 cards and gain 3 life"

[[cards.abilities]]
trigger = "on_play"
effect = "draw"
[cards.abilities.params]
amount = "3"

[[cards.abilities]]
trigger = "on_play"
effect = "gain_life"
[cards.abilities.params]
amount = "3"
```

## Notes

- All numeric parameters are specified as strings in TOML but are parsed as integers
- Player IDs are typically 0 (you) or 1 (opponent) in a two-player game
- Zone names must match those defined in your `rules.toml` file
- Card IDs referenced in parameters should be valid card IDs
- If a parameter is marked as "optional", the effect will use a sensible default when the parameter is omitted
- Some effects (like draw, mill, discard, move_card) may not be fully implemented yet but are structurally supported

## Scripted Effects

If the builtin effects don't meet your needs, you can still use Rhai scripts for more complex behavior. See [SCRIPTING_GUIDE.md](SCRIPTING_GUIDE.md) for details.

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
