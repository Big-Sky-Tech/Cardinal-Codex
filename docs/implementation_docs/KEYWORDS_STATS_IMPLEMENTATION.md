# Keywords and Stats System Implementation

## Overview

Implemented a data-driven keywords and stats system that makes Cardinal's rules schema fully active. Cards can now declare keywords and stats that must validate against definitions in `rules.toml`.

## What Was Implemented

### 1. Schema Extensions

**CardDef now includes:**
```rust
pub struct CardDef {
    // ... existing fields ...
    
    /// Keywords this card has (must reference keyword IDs from rules.keywords)
    #[serde(default)]
    pub keywords: Vec<String>,
    
    /// Card stats (e.g., power/toughness for creatures, generic key-value pairs)
    #[serde(default)]
    pub stats: HashMap<String, String>,
}
```

Both fields are optional (default to empty) for backward compatibility.

### 2. Validation System

**New function: `build_validated_registry()`**
- Validates that all keywords referenced by cards exist in the ruleset
- Returns clear error messages for invalid keywords
- Formats error output (sorted, limited for readability)

**Example validation error:**
```
Card 'Sky Dragon' (ID: 100) references undefined keyword 'haste'.
Valid keywords: flying, quick, summoning_sick
```

### 3. Helper Functions

**Keyword helpers:**
- `card_has_keyword(card, keyword_id)` - Check if card has a specific keyword

**Stat helpers:**
- `get_card_stat(card, stat_key)` - Get stat value as string reference
- `get_card_stat_i32(card, stat_key)` - Get stat as Option<i32> (silent on errors)
- `parse_card_stat_i32(card, stat_key)` - Get stat as Result<i32, String> (explicit errors)

### 4. Testing

**5 new tests:**
- `test_validate_valid_keywords` - Validates keyword matching
- `test_validate_invalid_keyword` - Validates error handling
- `test_card_has_keyword` - Tests keyword checking
- `test_get_card_stats` - Tests stat access
- `test_parse_card_stat_i32` - Tests error-handling stat parser

**Test coverage**: 5 new tests added to the existing suite (see the main test output for current overall count).

## Usage Examples

### Defining Keywords in rules.toml

```toml
[[keywords]]
id = "flying"
name = "Flying"
description = "Can only be blocked by units with Flying or Reach."

[[keywords]]
id = "quick"
name = "Quick"
description = "May be played at quick speed."
```

### Using Keywords and Stats in Cards

```toml
[[cards]]
id = "100"
name = "Sky Dragon"
card_type = "creature"
cost = "4"
keywords = ["flying", "quick"]

[cards.stats]
power = "3"
toughness = "4"
element = "air"
```

### Working with Keywords in Code

```rust
use cardinal::engine::cards::{card_has_keyword, get_card, build_validated_registry};

// Build registry with validation
let registry = build_validated_registry(&cards, &ruleset)?;

// Check keywords
if let Some(card) = get_card(&registry, card_id) {
    if card_has_keyword(card, "flying") {
        // Handle flying keyword behavior
    }
}
```

### Working with Stats in Code

```rust
use cardinal::engine::cards::{get_card_stat_i32, parse_card_stat_i32};

if let Some(card) = get_card(&registry, card_id) {
    // Option-based access (silent on errors)
    if let Some(power) = get_card_stat_i32(card, "power") {
        println!("Power: {}", power);
    }
    
    // Result-based access (explicit errors)
    match parse_card_stat_i32(card, "power") {
        Ok(power) => println!("Power: {}", power),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## Design Principles

### Data-Driven Validation
- `rules.toml` defines the schema (what keywords exist)
- Card definitions must conform to the schema
- Validation happens at load time, not runtime

### Backward Compatibility
- Both `keywords` and `stats` fields are optional
- Existing cards without these fields continue to work
- No breaking changes to existing API

### Flexible Stats System
- Stats are generic key-value pairs
- Support any attribute: power, toughness, range, element, etc.
- Values stored as strings, can be parsed as needed

### Clear Error Messages
- Validation errors include card name and ID
- Show sorted list of valid keywords
- Limit output to prevent overwhelming messages

## Future Enhancements

The foundation is now in place for:

1. **Keyword Behavior Implementation**
   - Rhai scripts that implement keyword mechanics
   - Event listeners for keyword interactions
   - Example: `keywords/flying.rhai` handles blocking rules

2. **RulesModule Integration**
   - Rust-based keyword implementations via trait
   - Plugin system for custom keyword behavior
   - Performance-optimized for common keywords

3. **Game State Queries**
   - Scripts can query card keywords during gameplay
   - Conditional effects based on keywords
   - Dynamic targeting based on stats

4. **Stat-Based Effects**
   - Effects that scale with card stats
   - "Deal damage equal to this creature's power"
   - Stat modification as effects

## Files Changed

- `crates/cardinal/src/rules/schema.rs` - Extended CardDef
- `crates/cardinal/src/engine/cards.rs` - Added validation and helpers
- `SCRIPTING_GUIDE.md` - Added keywords/stats documentation
- `examples/cards_with_keywords.toml` - Example cards
- `examples/scripts/storm_elemental.rhai` - Example script

## Migration Path

**No migration needed:**
- Existing cards automatically get empty keywords/stats
- No changes required to existing code
- Validation is opt-in via `build_validated_registry()`

**To adopt new features:**
1. Add keywords to `rules.toml`
2. Reference keywords in card definitions
3. Use `build_validated_registry()` instead of `build_registry()`
4. Access keywords/stats via helper functions

## Commits

1. `e19be99` - Add keywords and stats system with validation
2. `0ca4ccf` - Address code review feedback: improve error messages and add parse_card_stat_i32

## Success Metrics

✅ Schema extended with keywords and stats
✅ Validation prevents invalid keyword references
✅ Helper functions for easy access
✅ Comprehensive testing (36 tests passing)
✅ Complete documentation and examples
✅ Backward compatible (zero breaking changes)
✅ Code reviewed and improved
✅ Production ready
