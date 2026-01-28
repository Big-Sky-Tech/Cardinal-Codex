# Cardinal Script Examples

This directory contains example Rhai scripts demonstrating Cardinal's scripting capabilities.

## Basic Examples

### `lightning_bolt.rhai`
Simple direct damage spell - demonstrates basic `deal_damage` helper.

### `healing_touch.rhai`
Simple life gain spell - demonstrates `gain_life` helper.

### `dark_ritual.rhai`
Resource generation spell - demonstrates `gain_resource` helper.

### `storm_elemental.rhai`
Creature with enter-the-battlefield (ETB) effect.

## Advanced Examples

### `arcane_transmuter.rhai`
**Complex multi-effect card** demonstrating:
- `mill_cards` - Zone manipulation
- `gain_resource` - Resource access from rules.toml
- `draw_cards` - Card draw
- `grant_keyword` - Keyword manipulation from rules.toml
- `add_counter` - Counter mechanics

### `mana_ritual.rhai`
Resource generation using `gain_resource` to access mana from rules.toml.

### `wings_of_freedom.rhai`
Enchantment demonstrating:
- `grant_keyword` - Grant flying keyword
- `pump_creature` - Stat modification

### `mill_storm.rhai`
Mill spell using `mill_cards` for deck manipulation.

### `token_generator.rhai`
Token creation using `create_token` to spawn multiple creatures.

### `power_surge.rhai`
Buff spell combining:
- `pump_creature` - Temporary stat boost
- `add_counter` - Permanent counter addition

### `mystic_bolt.rhai`
**Type helper demonstration** using:
- `cantrip` - Common "effect + draw" pattern
- `bolt` - Simple damage alias

## Helper Function Categories

See `advanced_examples.rhai` for comprehensive examples of all helper functions, organized by category:

1. **Damage & Life**: deal_damage, gain_life, lose_life, set_life
2. **Card Movement**: draw_cards, mill_cards, discard_cards, move_card, shuffle_zone
3. **Stats**: pump_creature, set_stats, modify_stat, set_stat
4. **Keywords**: grant_keyword, remove_keyword
5. **Resources**: gain_resource, spend_resource, set_resource
6. **Tokens**: create_token
7. **Counters**: add_counter, remove_counter
8. **Type Helpers**: bolt, drain, cantrip

## TOML Attribute Access

Scripts can manipulate attributes defined in `rules.toml`:

### Zones
```rhai
move_card(card, "hand", "graveyard")
shuffle_zone(player, "deck")
mill_cards(player, 3)  // deck -> graveyard
```

### Resources
```rhai
gain_resource(controller, "mana", 5)
spend_resource(controller, "action_points", 2)
```

### Keywords
```rhai
grant_keyword(card, "flying")      // Must be defined in rules.toml
remove_keyword(card, "summoning_sick")
```

### Custom Stats
```rhai
modify_stat(card, "power", 2)      // Modify any stat from card.stats
set_stat(card, "element", "fire")
```

## Context Variables

Scripts have access to:
- `controller` - Player ID controlling the effect
- `source_card` - Card ID that triggered the ability
- `active_player` - (Optional) Active player's ID
- `turn_number` - (Optional) Current turn number
- `phase` - (Optional) Current phase ID

## Documentation

For complete documentation, see:
- `docs/SCRIPTING_GUIDE.md` - Full scripting reference
- `examples/advanced_cards.toml` - Example card definitions
- `rules.toml` - Game rules and attribute definitions
