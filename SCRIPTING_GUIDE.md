# Cardinal Scripting Guide

## Overview

Cardinal supports a **hybrid card system** that allows cards to be defined using either:
1. **TOML-based builtin effects** - Simple, declarative effects for common actions
2. **Rhai scripts** - Full scripting power for custom, complex card behaviors

This guide covers how to create and use scripted cards.

## Why Use Scripts?

- **Flexibility**: Create complex, multi-step effects that builtins can't express
- **Customization**: Define unique card mechanics without modifying the engine
- **Safety**: Scripts run in a sandboxed environment with deterministic execution
- **Performance**: Builtins are still faster; use scripts only when needed

## Card Definition Structure

### TOML-Based Builtin Card

```toml
[[cards]]
id = "1"
name = "Lightning Shock"
card_type = "spell"
description = "Deal 2 damage"

[[cards.abilities]]
trigger = "on_play"
effect = "damage"
[cards.abilities.params]
amount = "2"
```

### Rhai-Scripted Card

```toml
[[cards]]
id = "10"
name = "Lightning Bolt"
card_type = "spell"
description = "Deal 3 damage (scripted)"
script_path = "scripts/lightning_bolt.rhai"  # Path to script file

[[cards.abilities]]
trigger = "on_play"
effect = "script:lightning_bolt"  # Effect name with "script:" prefix
```

## Writing Rhai Card Scripts

### Basic Structure

Every card script must define an `execute_ability()` function:

```rhai
fn execute_ability() {
    // Return a single effect
    deal_damage(1, 3)
}
```

Or return multiple effects as an array:

```rhai
fn execute_ability() {
    // Return multiple effects
    [
        deal_damage(1, 2),
        gain_life(0, 3)
    ]
}
```

### Available Helper Functions

Cardinal provides safe helper functions for scripts:

#### `deal_damage(target: i32, amount: i32)`
Deal damage to a player.

```rhai
fn execute_ability() {
    deal_damage(1, 5)  // Deal 5 damage to player 1
}
```

#### `gain_life(player: i32, amount: i32)`
A player gains life.

```rhai
fn execute_ability() {
    gain_life(0, 3)  // Controller gains 3 life
}
```

#### `draw_cards(player: i32, count: i32)`
A player draws cards (not yet implemented in executor).

```rhai
fn execute_ability() {
    draw_cards(0, 2)  // Controller draws 2 cards
}
```

#### `pump_creature(card: i32, power: i32, toughness: i32)`
Modify creature stats (not yet implemented in executor).

```rhai
fn execute_ability() {
    pump_creature(source_card, 2, 2)  // +2/+2
}
```

### Context Variables

Scripts have access to these context variables:

- `controller` - The player ID who controls the card (i32)
- `source_card` - The card ID that triggered the ability (i32)

```rhai
fn execute_ability() {
    // Use context variables
    gain_life(controller, 5)  // Controller gains life
}
```

## Complete Examples

### Example 1: Simple Damage

**File**: `scripts/shock.rhai`
```rhai
// Simple damage spell
fn execute_ability() {
    deal_damage(1, 2)
}
```

**TOML**:
```toml
[[cards]]
id = "5"
name = "Shock"
card_type = "spell"
script_path = "scripts/shock.rhai"

[[cards.abilities]]
trigger = "on_play"
effect = "script:shock"
```

### Example 2: Life Drain

**File**: `scripts/life_drain.rhai`
```rhai
// Drain life: damage opponent and gain life
fn execute_ability() {
    [
        deal_damage(1, 3),
        gain_life(0, 3)
    ]
}
```

**TOML**:
```toml
[[cards]]
id = "6"
name = "Drain Life"
card_type = "spell"
script_path = "scripts/life_drain.rhai"

[[cards.abilities]]
trigger = "on_play"
effect = "script:life_drain"
```

### Example 3: Conditional Effect (Future)

```rhai
// Future: conditional effects based on game state
fn execute_ability() {
    // When game state access is added
    if player_life(0) < 10 {
        gain_life(0, 5)
    } else {
        deal_damage(1, 3)
    }
}
```

## Script Execution Flow

1. **Load Time**: Scripts are compiled when the game engine is initialized
2. **Trigger Time**: When a card's ability triggers, the corresponding effect is pushed to the stack
3. **Resolution Time**: When the stack resolves, the script's `execute_ability()` function is called
4. **Effect Application**: The script's return value is converted to Commands and applied to the game state

## Determinism and Safety

Cardinal's scripting engine is configured for deterministic, safe execution:

- **No I/O**: Scripts cannot read files, access the network, or perform I/O
- **No Threading**: Scripts run single-threaded
- **No System Time**: Scripts cannot access the system clock
- **Operation Limits**: Scripts have a maximum operation count to prevent infinite loops
- **Recursion Limits**: Limited expression depth to prevent stack overflow

## Performance Considerations

- **Builtin effects** are fastest - use them for simple, common effects
- **Scripted effects** have overhead from script execution
- **Hybrid approach**: Use builtins for most cards, scripts for unique mechanics

## Migration Path

Existing TOML-only cards continue to work without changes:
- No `script_path` field = builtin-only card
- With `script_path` = hybrid card (can have both builtin and scripted abilities)

## Future Enhancements

Planned features for the scripting system:

1. **Game State Access**: Read player life, card counts, zone contents
2. **Target Selection**: Request player input for targets
3. **Conditional Logic**: If/else based on game state
4. **Card Queries**: Find cards in zones matching criteria
5. **Custom Triggers**: Define new trigger types in scripts
6. **Persistent Effects**: Effects that last multiple turns

## Troubleshooting

### Script Won't Load
- Check that the `script_path` in TOML is correct
- Ensure the script defines `execute_ability()` function
- Check for syntax errors in the Rhai script

### Effect Not Executing
- Verify the effect name in TOML matches the script_path basename
- Use "script:" prefix in the effect field
- Check that the script returns valid effect maps

### Type Errors
- All numbers in Rhai are i32 (32-bit integers)
- Ensure player IDs and amounts are integer values
- Use appropriate helper functions for effect types

## References

- [Rhai Language Documentation](https://rhai.rs/)
- Cardinal Architecture: See `ARCHITECTURE.md`
- Example Scripts: See `examples/scripts/`
- Example Cards: See `examples/hybrid_cards.toml`

## Keywords and Stats System

### Overview

Cardinal's rules schema is now fully active - cards can declare keywords and stats that must match definitions in `rules.toml`. This ensures consistency and enables data-driven gameplay mechanics.

### Defining Keywords in Rules

Keywords are defined in `rules.toml`:

```toml
[[keywords]]
id          = "flying"
name        = "Flying"
description = "Can only be blocked by units with Flying or Reach."

[[keywords]]
id          = "quick"
name        = "Quick"
description = "May be played at quick speed (during opponent's turn)."
```

### Using Keywords in Cards

Cards reference keywords by their `id`:

```toml
[[cards]]
id = "100"
name = "Sky Dragon"
card_type = "creature"
keywords = ["flying", "quick"]  # Must match keyword IDs from rules.toml

[cards.stats]
power = "3"
toughness = "4"
```

### Card Stats

Stats are key-value pairs that can represent any game-relevant attributes:

```toml
[cards.stats]
power = "3"           # Creature combat power
toughness = "4"       # Creature durability
range = "2"           # Custom stat: attack range
element = "fire"      # Custom stat: elemental type
```

### Validation

**Keyword Validation:**
- Cards can only reference keywords defined in `rules.toml`
- Unknown keywords will cause validation errors at load time
- Use `build_validated_registry()` to enable validation

**Example Error:**
```
Card 'Sky Dragon' (ID: 100) references undefined keyword 'haste'.
Valid keywords: ["flying", "quick", "summoning_sick"]
```

### Working with Keywords and Stats in Code

**Check if a card has a keyword:**
```rust
use cardinal::engine::cards::{card_has_keyword, get_card};

if let Some(card) = get_card(registry, card_id) {
    if card_has_keyword(card, "flying") {
        // Card has flying keyword
    }
}
```

**Access card stats:**
```rust
use cardinal::engine::cards::{get_card_stat, get_card_stat_i32};

if let Some(card) = get_card(registry, card_id) {
    // Get stat as string
    if let Some(element) = get_card_stat(card, "element") {
        println!("Element: {}", element);
    }
    
    // Get stat as integer
    if let Some(power) = get_card_stat_i32(card, "power") {
        println!("Power: {}", power);
    }
}
```

### Future: Keyword Behavior Implementation

Keywords will be implemented via:
1. **Rhai scripts** - Custom behavior for each keyword
2. **RulesModule trait** - Rust-based keyword implementations
3. **Event listeners** - Keywords react to game events

Example future implementation:
```rhai
// keywords/flying.rhai
fn on_declare_blockers(ctx) {
    // Only allow flying/reach creatures to block
    if !ctx.blocker_has_keyword("flying") && !ctx.blocker_has_keyword("reach") {
        return reject_block("Only flying creatures can block flying")
    }
}
```

### Migration Path

**Existing cards remain compatible:**
- `keywords` field is optional (defaults to empty array)
- `stats` field is optional (defaults to empty map)
- No breaking changes to existing card definitions

### Complete Example

```toml
[[cards]]
id = "104"
name = "Storm Elemental"
card_type = "creature"
cost = "5"
description = "A powerful elemental that damages opponents when it enters"
keywords = ["flying"]
script_path = "scripts/storm_elemental.rhai"

[cards.stats]
power = "4"
toughness = "5"
element = "air"

[[cards.abilities]]
trigger = "etb"
effect = "script:storm_elemental"
```

See `examples/cards_with_keywords.toml` for more examples.
