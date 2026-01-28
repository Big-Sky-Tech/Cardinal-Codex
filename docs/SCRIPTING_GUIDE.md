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

Cardinal provides a comprehensive set of safe helper functions for scripts, organized by category:

#### Damage & Life Manipulation

##### `deal_damage(target: i32, amount: i32)`
Deal damage to a player.

```rhai
fn execute_ability() {
    deal_damage(1, 5)  // Deal 5 damage to player 1
}
```

##### `gain_life(player: i32, amount: i32)`
A player gains life.

```rhai
fn execute_ability() {
    gain_life(0, 3)  // Controller gains 3 life
}
```

##### `lose_life(player: i32, amount: i32)`
A player loses life (distinct from damage - doesn't trigger damage effects).

```rhai
fn execute_ability() {
    lose_life(1, 2)  // Opponent loses 2 life
}
```

##### `set_life(player: i32, amount: i32)`
Set a player's life to a specific value.

```rhai
fn execute_ability() {
    set_life(1, 10)  // Set opponent's life to exactly 10
}
```

#### Card Draw & Zone Movement

##### `draw_cards(player: i32, count: i32)`
A player draws cards from their deck.

```rhai
fn execute_ability() {
    draw_cards(0, 2)  // Controller draws 2 cards
}
```

##### `mill_cards(player: i32, count: i32)`
Move cards from top of a player's deck to their graveyard.

```rhai
fn execute_ability() {
    mill_cards(1, 3)  // Mill 3 cards from opponent's deck
}
```

##### `discard_cards(player: i32, count: i32)`
Move cards from a player's hand to their graveyard.

```rhai
fn execute_ability() {
    discard_cards(1, 2)  // Opponent discards 2 cards
}
```

##### `move_card(card: i32, from_zone: &str, to_zone: &str)`
General purpose card movement between zones.

```rhai
fn execute_ability() {
    move_card(source_card, "field", "graveyard")  // Destroy this card
}
```

##### `shuffle_zone(player: i32, zone: &str)`
Shuffle a zone (typically a deck).

```rhai
fn execute_ability() {
    shuffle_zone(0, "deck")  // Shuffle controller's deck
}
```

#### Creature & Stat Modification

##### `pump_creature(card: i32, power: i32, toughness: i32)`
Modify creature stats by a delta (can be negative).

```rhai
fn execute_ability() {
    pump_creature(source_card, 2, 2)  // +2/+2
}
```

##### `set_stats(card: i32, power: i32, toughness: i32)`
Set creature stats to specific values (not delta).

```rhai
fn execute_ability() {
    set_stats(source_card, 3, 3)  // Becomes a 3/3 creature
}
```

##### `modify_stat(card: i32, stat_name: &str, delta: i32)`
Modify any card stat by name (generic stat modification).

```rhai
fn execute_ability() {
    modify_stat(source_card, "range", 1)  // Increase range by 1
}
```

##### `set_stat(card: i32, stat_name: &str, value: &str)`
Set any card stat to a specific value.

```rhai
fn execute_ability() {
    set_stat(source_card, "element", "fire")  // Change element to fire
}
```

#### Keyword Manipulation

##### `grant_keyword(card: i32, keyword: &str)`
Grant a keyword to a card (e.g., "flying", "quick").

```rhai
fn execute_ability() {
    grant_keyword(source_card, "flying")  // Grant flying
}
```

##### `remove_keyword(card: i32, keyword: &str)`
Remove a keyword from a card.

```rhai
fn execute_ability() {
    remove_keyword(source_card, "flying")  // Remove flying
}
```

#### Resource Manipulation

##### `gain_resource(player: i32, resource: &str, amount: i32)`
Grant resources to a player (e.g., "mana", "action_points" from rules.toml).

```rhai
fn execute_ability() {
    gain_resource(controller, "mana", 3)  // Gain 3 mana
}
```

##### `spend_resource(player: i32, resource: &str, amount: i32)`
Spend/consume resources.

```rhai
fn execute_ability() {
    spend_resource(controller, "action_points", 2)  // Spend 2 action points
}
```

##### `set_resource(player: i32, resource: &str, amount: i32)`
Set resource to a specific value.

```rhai
fn execute_ability() {
    set_resource(controller, "mana", 5)  // Set mana to exactly 5
}
```

#### Token & Card Creation

##### `create_token(player: i32, token_type: &str, zone: &str)`
Create a token card in a specified zone.

```rhai
fn execute_ability() {
    create_token(controller, "soldier_token", "field")  // Create a soldier token
}
```

#### Counter & Marker Manipulation

##### `add_counter(card: i32, counter_type: &str, amount: i32)`
Add counters to a card (e.g., +1/+1 counters, charge counters).

```rhai
fn execute_ability() {
    add_counter(source_card, "+1/+1", 2)  // Add two +1/+1 counters
}
```

##### `remove_counter(card: i32, counter_type: &str, amount: i32)`
Remove counters from a card.

```rhai
fn execute_ability() {
    remove_counter(source_card, "+1/+1", 1)  // Remove one +1/+1 counter
}
```

#### Type Helpers - Common Patterns

##### `bolt(target: i32, damage: i32)`
Common pattern: simple direct damage (alias for deal_damage).

```rhai
fn execute_ability() {
    bolt(1, 3)  // Deal 3 damage to player 1
}
```

##### `drain(target: i32, amount: i32, controller: i32)`
Common pattern: damage opponent and gain life.

```rhai
fn execute_ability() {
    drain(1, 3, controller)  // Deal 3 damage and gain 3 life
}
```

##### `cantrip(player: i32, effect: Dynamic)`
Common pattern: effect + draw a card.

```rhai
fn execute_ability() {
    cantrip(controller, bolt(1, 2))  // Deal 2 damage, then draw
}
```

### Context Variables

Scripts have access to these context variables:

- `controller` - The player ID who controls the card (i32)
- `source_card` - The card ID that triggered the ability (i32)
- `active_player` - (Optional) The active player's ID (i32) - Available when provided by context
- `turn_number` - (Optional) Current turn number (i32) - Available when provided by context
- `phase` - (Optional) Current phase ID (string) - Available when provided by context

```rhai
fn execute_ability() {
    // Use context variables
    gain_life(controller, 5)  // Controller gains life
    
    // Future: conditional logic based on turn
    // if turn_number > 5 {
    //     bolt(1, 5)  // More damage in late game
    // }
}
```

## Accessing TOML-Defined Attributes

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

// Or use the drain type helper:
fn execute_ability_alt() {
    drain(1, 3, controller)
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

### Example 3: Resource Ritual

**File**: `scripts/dark_ritual.rhai`
```rhai
// Grant mana when played
fn execute_ability() {
    gain_resource(controller, "mana", 3)
}
```

### Example 4: Mill Effect

**File**: `scripts/mill_spell.rhai`
```rhai
// Mill opponent's deck
fn execute_ability() {
    mill_cards(1, 3)  // Opponent mills 3 cards
}
```

### Example 5: Creature with Stats and Keywords

**File**: `scripts/sky_knight.rhai`
```rhai
// When this creature enters, grant it flying
fn execute_ability() {
    grant_keyword(source_card, "flying")
}
```

**TOML**:
```toml
[[cards]]
id = "10"
name = "Sky Knight"
card_type = "creature"
cost = "3"
script_path = "scripts/sky_knight.rhai"
keywords = []  # Keywords granted by script

[cards.stats]
power = "2"
toughness = "2"

[[cards.abilities]]
trigger = "etb"
effect = "script:sky_knight"
```

### Example 6: Token Creation

**File**: `scripts/summon_soldiers.rhai`
```rhai
// Create two 1/1 soldier tokens
fn execute_ability() {
    [
        create_token(controller, "soldier_token", "field"),
        create_token(controller, "soldier_token", "field")
    ]
}
```

### Example 7: Counter Manipulation

**File**: `scripts/strengthen.rhai`
```rhai
// Add +1/+1 counters to target creature
fn execute_ability() {
    add_counter(source_card, "+1/+1", 2)
}
```

### Example 8: Complex Multi-Effect Card

**File**: `scripts/arcane_surge.rhai`
```rhai
// Complex card: damage, heal, draw, and mana
fn execute_ability() {
    [
        deal_damage(1, 2),        // Deal 2 damage to opponent
        gain_life(controller, 2), // Gain 2 life
        draw_cards(controller, 1),// Draw a card
        gain_resource(controller, "mana", 1)  // Gain 1 mana
    ]
}
```

### Example 9: Conditional Effect (Future)

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

## Accessing TOML-Defined Attributes

Scripts can manipulate attributes defined in `rules.toml` using the helper functions:

### Zones (from rules.toml)
```rhai
// Move cards between zones defined in rules.toml
move_card(card_id, "hand", "graveyard")
move_card(card_id, "deck", "field")
move_card(card_id, "field", "banished")

// Shuffle zones
shuffle_zone(controller, "deck")
```

### Resources (from rules.toml)
```rhai
// Manipulate resources defined in rules.toml [[resources]]
gain_resource(controller, "mana", 5)
spend_resource(controller, "action_points", 2)
set_resource(controller, "mana", 10)
```

### Keywords (from rules.toml)
```rhai
// Grant/remove keywords defined in rules.toml [[keywords]]
grant_keyword(source_card, "flying")
grant_keyword(source_card, "quick")
remove_keyword(source_card, "summoning_sick")
```

### Custom Stats (from card stats)
```rhai
// Modify any stat defined in card TOML [cards.stats]
modify_stat(source_card, "power", 2)    // Increase power by 2
modify_stat(source_card, "range", -1)   // Decrease range by 1
set_stat(source_card, "element", "fire") // Change element to fire
```

### Example: Card Using TOML Attributes

**rules.toml**:
```toml
[[resources]]
id = "mana"
name = "Mana"
# ... other fields

[[keywords]]
id = "flying"
name = "Flying"
# ... other fields

[[zones]]
id = "graveyard"
name = "Graveyard"
# ... other fields
```

**card.toml**:
```toml
[[cards]]
id = "20"
name = "Mystic Phoenix"
card_type = "creature"
script_path = "scripts/mystic_phoenix.rhai"

[cards.stats]
power = "3"
toughness = "2"
element = "fire"
```

**scripts/mystic_phoenix.rhai**:
```rhai
// When this enters the battlefield:
// - Grant it flying (keyword from rules.toml)
// - Gain mana (resource from rules.toml)
// - If destroyed, goes to graveyard (zone from rules.toml)
fn execute_ability() {
    [
        grant_keyword(source_card, "flying"),
        gain_resource(controller, "mana", 2)
    ]
}
```

## Complete Examples

## Script Execution Flow

1. **Load Time**: During host initialization, scripts must be loaded and registered with the engine (e.g., via `engine.scripting.register_script(...)`). `GameEngine::new` / `from_ruleset` only construct an empty scripting engine and do **not** automatically load scripts from `script_path`.
2. **Trigger Time**: When a card's ability triggers, the corresponding (builtin or scripted) effect is pushed to the stack
3. **Resolution Time**: When the stack resolves, the scripted effect's `execute_ability()` function is called (if the card uses a script)
4. **Effect Application**: The script's return value is converted to Commands and applied to the game state, just like builtin effects

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

1. **Game State Access** (Partially Complete):
   - ✅ Context variables: controller, source_card, active_player, turn_number, phase
   - 🔄 Read player life totals
   - 🔄 Query card counts in zones
   - 🔄 Access zone contents

2. **TOML Attribute Access** (Complete):
   - ✅ Zone manipulation (move_card, shuffle_zone, mill, discard)
   - ✅ Resource access/modification (gain_resource, spend_resource, set_resource)
   - ✅ Keyword manipulation (grant_keyword, remove_keyword)
   - ✅ Custom stat modification (modify_stat, set_stat)

3. **Target Selection** (Planned):
   - 🔄 Request player input for targets
   - 🔄 Filter valid targets by criteria
   - 🔄 Multi-target selection

4. **Conditional Logic** (Partially Available):
   - ✅ Scripts can use Rhai's if/else
   - 🔄 Game state queries for conditions
   - 🔄 Player choice resolution

5. **Card Queries** (Planned):
   - 🔄 Find cards in zones matching criteria
   - 🔄 Count cards with specific attributes
   - 🔄 Filter by keywords, stats, types

6. **Custom Triggers** (Planned):
   - 🔄 Define new trigger types in scripts
   - 🔄 Event listeners for game events
   - 🔄 Replacement effects

7. **Persistent Effects** (Planned):
   - 🔄 Effects that last multiple turns
   - 🔄 Until-end-of-turn modifiers
   - 🔄 Continuous effects

Legend:
- ✅ Complete
- 🔄 Planned/In Progress
- ❌ Not Started

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
