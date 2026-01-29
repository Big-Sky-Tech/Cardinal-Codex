# VS Code TOML Validation - User Experience

This document shows what developers will see when using the Cardinal TOML schemas in VS Code.

## Setup (One-Time)

When you first open the repository in VS Code:

1. **Extension Prompt**: VS Code shows: *"This workspace recommends extensions. Do you want to install them?"*
   - Click **Install** to add "Even Better TOML"

2. **Ready!** Open any TOML file and schemas automatically activate

## Validation in Action

### ✅ Valid Card (No Errors)

```toml
# cards/my_card.toml
id = "1"
name = "Lightning Bolt"
card_type = "spell"
cost = "1R"
description = "Deal 3 damage to any target."
```

**VS Code shows:** Green checkmark ✓, no squiggles

---

### ❌ Missing Required Field

```toml
# cards/broken_card.toml
id = "2"
name = "Broken Card"
# Missing: card_type (required!)
cost = "2U"
```

**VS Code shows:**
- Red squiggle under the entire file or `name` field
- Hover message: `'card_type' is a required property`
- Error in Problems panel

---

### ❌ Typo in Field Name

```toml
# cards/typo_card.toml
id = "3"
name = "Card With Typo"
card_type = "creature"
descritpion = "This field is misspelled"  # ← typo here
```

**VS Code shows:**
- Red squiggle under `descritpion`
- Hover message: `Additional properties are not allowed ('descritpion' was unexpected)`
- Suggestion might show `description` as alternative

---

### ❌ Wrong Data Type

```toml
# rules.toml
[players]
min_players = "two"  # ← should be integer, not string
max_players = 2
```

**VS Code shows:**
- Red squiggle under `"two"`
- Hover message: `'two' is not of type 'integer'`

---

### ❌ Invalid Enum Value

```toml
# rules.toml
[[zones]]
id = "deck"
name = "Deck"
owner_scope = "global"  # ← Invalid! Only "player" or "shared" allowed
visibility = "private"
ordered = true
allow_duplicates = true
```

**VS Code shows:**
- Red squiggle under `"global"`
- Hover message: `'global' is not one of ['player', 'shared']`
- Shows valid options: `player`, `shared`

---

## Autocomplete / IntelliSense

When editing TOML files:

1. **Press Ctrl+Space (or Cmd+Space on Mac)**
2. **See available fields:**
   ```
   ┌─────────────────────────┐
   │ id                      │ ← Required
   │ name                    │ ← Required
   │ card_type               │ ← Required
   │ abilities               │
   │ cost                    │
   │ description             │
   │ keywords                │
   │ script_path             │
   │ stats                   │
   └─────────────────────────┘
   ```

3. **Arrow keys to select, Enter to insert**

---

## Field Documentation on Hover

Hover over any field to see its documentation:

```toml
card_type = "creature"
  ↑
  Hover here
```

**Tooltip shows:**
```
card_type (string)
Card type (e.g., 'creature', 'spell', 'enchantment')

Examples:
  - "creature"
  - "spell"
  - "enchantment"
  - "artifact"
```

---

## Complex Nested Validation

The schemas validate nested structures too:

```toml
[[abilities]]
trigger = "etb"
effect = "damage"

[abilities.params]
amout = "1"  # ← Typo: should be "amount"
target = "opponent"
```

**VS Code shows:**
- Red squiggle under `amout`
- Message: `Additional properties are not allowed ('amout' was unexpected)`
- The nested path `abilities.params.amout` is highlighted

---

## Problems Panel

All errors appear in VS Code's **Problems** panel (Ctrl+Shift+M):

```
PROBLEMS
 ⚠ cards/broken_card.toml
   ✗ 'card_type' is a required property (line 1)
 ⚠ cards/typo_card.toml
   ✗ Additional properties are not allowed ('descritpion' was unexpected) (line 4)
 ⚠ rules.toml
   ✗ 'global' is not one of ['player', 'shared'] (line 65)
```

Click any error to jump to that line.

---

## Benefits Summary

| Feature | Benefit |
|---------|---------|
| **Real-time validation** | See errors as you type |
| **Precise error messages** | Know exactly what's wrong |
| **Autocomplete** | Discover available fields |
| **Hover documentation** | Learn what fields do |
| **Type checking** | Catch wrong types immediately |
| **Enum validation** | Only allow valid values |
| **Required field checks** | Never forget mandatory fields |
| **Typo detection** | Catch misspelled field names |

---

## Before vs After

### Before Schemas ❌
1. Edit TOML file (with typo)
2. Run `cargo build` or `cargo test`
3. See cryptic serde error 5 minutes later
4. Hunt for the problem
5. Fix and repeat

### After Schemas ✅
1. Edit TOML file (with typo)
2. Red squiggle appears instantly
3. Hover to see exact problem
4. Fix immediately
5. No wasted time!

---

## File Pattern Associations

The schemas automatically activate for these file patterns:

| Pattern | Schema | Example |
|---------|--------|---------|
| `**/cards/*.toml` | `card.schema.json` | `cards/goblin_scout.toml` |
| `**/cards.toml` | `cards.schema.json` | `examples/advanced_cards.toml` |
| `**/rules.toml` | `rules.schema.json` | `rules.toml` |
| `**/pack.toml` | `pack.schema.json` | `examples/example-pack/pack.toml` |

No manual configuration needed—it just works!

---

## Testing Your TOML Files

You can also validate TOML files programmatically:

```bash
pip install toml jsonschema

python3 << 'EOF'
import json, toml
from jsonschema import validate

with open('schemas/card.schema.json') as f:
    schema = json.load(f)
with open('cards/my_card.toml') as f:
    card = toml.load(f)

validate(instance=card, schema=schema)
print("✓ Valid!")
EOF
```

---

## Conclusion

With JSON schemas integrated, Cardinal provides a **professional developer experience** where errors are caught immediately in the editor, not minutes later during compilation or runtime.

This dramatically speeds up the development cycle and makes it much easier to create correct TOML configurations on the first try.
