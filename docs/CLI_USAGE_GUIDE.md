# Cardinal CLI Usage Guide

The Cardinal CLI provides comprehensive tools for developing, validating, compiling, and testing trading card games using the Cardinal engine.

## Table of Contents

- [Installation](#installation)
- [Basic Commands](#basic-commands)
- [Validation](#validation)
- [Compilation](#compilation)
- [Testing](#testing)
- [Pack Management](#pack-management)
- [Examples](#examples)

## Installation

```bash
# Build from source
cargo build --release --bin cardinal-cli

# Run directly
cargo run --bin cardinal-cli -- [COMMAND]
```

## Basic Commands

### Play Interactive Game

Play the game with a terminal interface:

```bash
# Use default rules.toml
cardinal-cli play

# Specify custom rules file
cardinal-cli play --rules path/to/custom-rules.toml
```

### Get Help

```bash
# General help
cardinal-cli --help

# Command-specific help
cardinal-cli validate --help
cardinal-cli compile --help
cardinal-cli test --help
```

## Validation

Validate game assets before using them in production. All validation commands provide detailed error messages and warnings.

### Validate Rules File

Validate a `rules.toml` file for logical consistency:

```bash
cardinal-cli validate rules rules.toml
```

**Checks:**
- Game metadata is complete
- Phases and steps are properly defined
- Zones are defined without duplicates
- Player rules are valid
- No duplicate IDs

**Example output:**
```
Validating rules file: rules.toml
✓ Asset validation passed
```

### Validate Card Files

**Single card file:**
```bash
cardinal-cli validate card cards/goblin_scout.toml
```

**Cards directory:**
```bash
cardinal-cli validate cards-dir cards/
```

**Cards array file:**
```bash
cardinal-cli validate cards-file cards.toml
```

**Checks:**
- Card IDs are unique
- Required fields are present
- Card types are valid
- Creature stats are defined for creature cards
- Script paths exist (if specified)

### Validate Script Files

Validate Rhai script syntax:

```bash
cardinal-cli validate script scripts/fireball.rhai
```

**Checks:**
- Script compiles without syntax errors
- File is not empty

### Validate Pack Directory

Validate an entire pack before building:

```bash
cardinal-cli validate pack examples/example-pack/
```

**Checks:**
- `pack.toml` exists and is valid
- All cards are valid
- All scripts compile
- No duplicate card IDs

**Example output:**
```
Validating pack directory: examples/example-pack
✓ Asset validation passed
```

## Compilation

Compile game assets into optimized `.ccpack` artifacts for distribution.

### Compile Pack

Build a `.ccpack` file with automatic validation:

```bash
# Basic compilation (includes validation)
cardinal-cli compile pack examples/example-pack output/my-pack.ccpack

# Verbose output
cardinal-cli compile pack examples/example-pack output/my-pack.ccpack --verbose

# Skip validation (faster, but not recommended)
cardinal-cli compile pack examples/example-pack output/my-pack.ccpack --no-validate
```

**What it does:**
1. Validates the pack directory (unless `--no-validate` is used)
2. Collects all files (cards, scripts, pack.toml)
3. Generates manifest with SHA-256 hashes
4. Creates compressed `.ccpack` archive

**Verbose output example:**
```
Compiling pack...
  Input: examples/example-pack
  Output: output/my-pack.ccpack

Validating pack...
  ✓ Validation passed

Building pack...
✓ Pack built successfully: output/my-pack.ccpack
  Pack ID: starter-set
  Version: 1.0.0
  Files: 9

✓ Pack compiled successfully!
```

## Testing

Test and simulate game scenarios without building a full UI.

### Test Game Simulation

Run a basic game simulation to verify the engine works:

```bash
# Basic test
cardinal-cli test game

# Custom rules file
cardinal-cli test game --rules path/to/rules.toml

# Custom seed for deterministic testing
cardinal-cli test game --seed 12345

# Custom starting hand size
cardinal-cli test game --hand-size 7

# Verbose output
cardinal-cli test game --verbose
```

**What it does:**
- Initializes a game with test decks
- Attempts several game actions
- Reports success/failure and event counts

**Example output:**
```
Test completed successfully!
  Actions taken: 0
  Events emitted: 0
```

### Test Pack Loading

Verify a `.ccpack` file loads correctly:

```bash
# Basic pack test
cardinal-cli test pack output/my-pack.ccpack

# Verbose output
cardinal-cli test pack output/my-pack.ccpack --verbose
```

**What it does:**
- Loads the pack into memory
- Verifies manifest integrity
- Counts cards and scripts

**Example output:**
```
Pack test completed!
  Cards: 4
  Scripts: 4
  Total files: 9
```

## Pack Management

Manage `.ccpack` files for distribution.

### Build Pack (Legacy)

Build a pack without validation (use `compile pack` instead for better workflow):

```bash
cardinal-cli build-pack input-directory/ output.ccpack
```

### List Pack Contents

Inspect a `.ccpack` file:

```bash
cardinal-cli list-pack output/my-pack.ccpack
```

**Example output:**
```
Pack: starter-set
Version: 1.0.0
Name: Starter Card Set
Description: A starter collection of basic cards for Cardinal Codex

Files (9):
  cards/dark_ritual.toml (154 bytes, sha256: 42daec16...)
  cards/healing_touch.toml (145 bytes, sha256: 253dc830...)
  cards/lightning_bolt.toml (159 bytes, sha256: 116d6c2a...)
  cards/storm_elemental.toml (222 bytes, sha256: 1b101410...)
  pack.toml (157 bytes, sha256: 37f8ba9f...)
  scripts/dark_ritual.rhai (386 bytes, sha256: fa0f08ce...)
  scripts/healing_touch.rhai (178 bytes, sha256: 410bd1a7...)
  scripts/lightning_bolt.rhai (277 bytes, sha256: 575549de...)
  scripts/storm_elemental.rhai (238 bytes, sha256: c8b22cd5...)
```

### Unpack Pack

Extract a `.ccpack` file for inspection or modification:

```bash
cardinal-cli unpack-pack output/my-pack.ccpack extracted/
```

## Examples

### Complete Development Workflow

```bash
# 1. Create your pack directory
mkdir my-game-pack
cd my-game-pack

# 2. Create pack.toml
cat > pack.toml << EOF
pack_id = "my-game"
version = "1.0.0"
name = "My Awesome Game"
description = "An amazing card game"
dependencies = []
EOF

# 3. Create cards directory
mkdir cards
cat > cards/test_card.toml << EOF
id = "test_001"
name = "Test Card"
card_type = "creature"
cost = "1R"
description = "A test creature"

[stats]
power = "2"
toughness = "2"
EOF

# 4. Validate the pack
cd ..
cardinal-cli validate pack my-game-pack/

# 5. Compile the pack
cardinal-cli compile pack my-game-pack/ output/my-game.ccpack --verbose

# 6. Test the pack
cardinal-cli test pack output/my-game.ccpack --verbose

# 7. List pack contents
cardinal-cli list-pack output/my-game.ccpack
```

### Quick Validation Before Commit

```bash
# Validate everything before committing
cardinal-cli validate rules rules.toml
cardinal-cli validate cards-dir cards/
cardinal-cli validate pack examples/example-pack/
```

### CI/CD Integration

```bash
#!/bin/bash
# validate-and-build.sh

set -e

echo "Validating game assets..."
cardinal-cli validate rules rules.toml
cardinal-cli validate pack my-pack/

echo "Building pack..."
cardinal-cli compile pack my-pack/ dist/game.ccpack --verbose

echo "Testing pack..."
cardinal-cli test pack dist/game.ccpack

echo "✓ Build successful!"
```

### Development Loop

```bash
# Edit your cards
vim cards/new_card.toml

# Validate
cardinal-cli validate card cards/new_card.toml

# Test in context
cardinal-cli validate cards-dir cards/

# Rebuild pack
cardinal-cli compile pack my-pack/ output/latest.ccpack --verbose

# Test the game
cardinal-cli test game --rules rules.toml --verbose
```

## Error Handling

All commands provide clear error messages:

**Validation errors:**
```
Asset Validation Report
============================================================

❌ Errors (2):
  1. Card ID cannot be empty
  2. Phase 'main' has no steps

⚠️  Warnings (1):
  1. Starting life is 0

============================================================
❌ Validation failed with 2 error(s)
```

**Compilation errors:**
```
Compilation error: Pack validation failed with 1 error(s)
```

## Tips

1. **Always validate before compiling** - Use `compile pack` instead of `build-pack` for automatic validation
2. **Use verbose mode during development** - Add `--verbose` to see detailed information
3. **Test deterministically** - Use `--seed` for reproducible test results
4. **Integrate into CI/CD** - Add validation and testing to your build pipeline
5. **Version your packs** - Update version in `pack.toml` for each release

## Next Steps

- Read [PACK_SYSTEM.md](../PACK_SYSTEM.md) for pack format details
- See [SCRIPTING_GUIDE.md](../SCRIPTING_GUIDE.md) for Rhai scripting
- Check [README.md](../README.md) for general Cardinal documentation
