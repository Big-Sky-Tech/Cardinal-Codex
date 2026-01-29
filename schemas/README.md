# Cardinal TOML Schema Definitions

This directory contains JSON Schema definitions for validating TOML files used in the Cardinal rules engine.

## Schema Files

- **card.schema.json** - Schema for individual card TOML files (e.g., `cards/goblin_scout.toml`)
- **cards.schema.json** - Schema for files containing multiple cards using `[[cards]]` array format
- **rules.schema.json** - Schema for `rules.toml` files that define game structure
- **pack.schema.json** - Schema for `pack.toml` files in card packs

## VS Code Integration

To enable automatic TOML validation in VS Code:

1. **Install the recommended extension:**
   - Open VS Code in this repository
   - You should see a prompt to install recommended extensions
   - Install "Even Better TOML" (tamasfe.even-better-toml)
   
   Or manually install it from the Extensions marketplace.

2. **The schema associations are already configured** in `.vscode/settings.json`:
   - Files in `cards/` directory → `card.schema.json`
   - Files named `cards.toml` → `cards.schema.json`
   - Files named `rules.toml` → `rules.schema.json`
   - Files named `pack.toml` → `pack.schema.json`

3. **Using the validation:**
   - Open any TOML file covered by the schemas
   - Invalid fields or values will be highlighted with red squiggly lines
   - Hover over the error to see the exact validation message
   - Get autocomplete suggestions for valid fields (Ctrl+Space)

## What Gets Validated

### Individual Card Files (`card.schema.json`)

**Required fields:**
- `id` - Unique card identifier
- `name` - Card display name
- `card_type` - Type of card (e.g., "creature", "spell", "enchantment")

**Optional fields:**
- `cost` - Mana/resource cost
- `description` - Card text
- `abilities` - Array of card abilities with `trigger`, `effect`, and `params`
- `script_path` - Path to Rhai script file
- `keywords` - Array of keyword IDs from rules.toml
- `stats` - Object with card stats (e.g., `power`, `toughness`)

**Example:**
```toml
id = "1"
name = "Goblin Scout"
card_type = "creature"
cost = "1R"
description = "A small but feisty goblin."

[[abilities]]
trigger = "etb"
effect = "damage"

[abilities.params]
amount = "1"
target = "opponent"
```

### Cards Array Files (`cards.schema.json`)

Used for files that define multiple cards in one file using `[[cards]]` array syntax.

**Example:**
```toml
[[cards]]
id = "1"
name = "First Card"
card_type = "creature"

[[cards]]
id = "2"
name = "Second Card"
card_type = "spell"
```

### Rules Files (`rules.schema.json`)

**Required top-level sections:**
- `[game]` - Basic game info (id, name, version, description)
- `[players]` - Player rules (life totals, deck sizes, mulligan rules)
- `[[zones]]` - Zone definitions (deck, hand, field, etc.)
- `[[resources]]` - Resource definitions (mana, action points, etc.)
- `[turn]` - Turn structure with phases and steps
- `[[actions]]` - Available action types
- `[stack]` - Stack rules
- `[[trigger_kinds]]` - Trigger type definitions
- `[[keywords]]` - Keyword definitions
- `[[win_conditions]]` - Win condition rules
- `[[loss_conditions]]` - Loss condition rules

**Optional:**
- `[[cards]]` - Inline card definitions

### Pack Metadata (`pack.schema.json`)

**Required fields:**
- `pack_id` - Unique pack identifier
- `version` - Semantic version (e.g., "1.0.0")

**Optional fields:**
- `name` - Human-readable name
- `description` - Pack description
- `dependencies` - Array of required pack IDs

## Error Messages

The schemas provide precise error messages when validation fails:

- **Missing required field:** `'field_name' is a required property`
- **Invalid field:** `Additional properties are not allowed ('field_name' was unexpected)`
- **Wrong type:** `'value' is not of type 'expected_type'`
- **Invalid enum value:** `'value' is not one of ['valid1', 'valid2']`

## Testing Schemas

You can test schema validation using Python:

```bash
pip install toml jsonschema

python3 << 'EOF'
import json
import toml
from jsonschema import validate

# Load schema and TOML file
with open('schemas/card.schema.json', 'r') as f:
    schema = json.load(f)
with open('cards/your_card.toml', 'r') as f:
    data = toml.load(f)

# Validate
validate(instance=data, schema=schema)
print("Valid!")
EOF
```

## Schema Updates

When updating the Rust structs in `crates/cardinal/src/rules/schema.rs`:

1. Update the corresponding JSON schema files in this directory
2. Test with existing TOML files to ensure compatibility
3. Update this README if new fields or validation rules are added

## Benefits

- **Catch errors early** - Invalid configurations are caught in the editor before runtime
- **Better developer experience** - Autocomplete and inline documentation
- **Documentation** - Schemas serve as machine-readable documentation of valid TOML structure
- **CI/CD integration** - Schemas can be used in automated testing pipelines
