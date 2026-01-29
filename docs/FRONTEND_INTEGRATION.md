# Frontend Integration Guide

This guide demonstrates how any frontend can integrate with Cardinal Codex using the enhanced CLI tooling.

## Overview

Cardinal Codex is now production-ready with comprehensive CLI tools that make it easy for any frontend (web, mobile, desktop, AI) to integrate without requiring custom build tools or validation logic.

## Quick Start for Frontend Developers

### 1. Install Cardinal CLI

```bash
# Clone the repository
git clone https://github.com/Big-Sky-Tech/Cardinal-Codex.git
cd Cardinal-Codex

# Build the CLI
cargo build --release --bin cardinal-cli

# CLI is now available at target/release/cardinal-cli
```

### 2. Prepare Your Game Assets

Create your game pack structure:

```
my-game/
├── pack.toml         # Pack metadata
├── cards/            # Card definitions
│   ├── card1.toml
│   ├── card2.toml
│   └── ...
└── scripts/          # Optional Rhai scripts
    ├── card1.rhai
    ├── card2.rhai
    └── ...
```

### 3. Validate Assets

Before compiling, validate your assets to catch errors:

```bash
# Validate the entire pack
cardinal-cli validate pack my-game/

# Or validate individual components
cardinal-cli validate rules rules.toml
cardinal-cli validate cards-dir my-game/cards/
cardinal-cli validate script my-game/scripts/fireball.rhai
```

**Expected output (success):**
```
Validating pack directory: my-game/
✓ Asset validation passed
```

**Expected output (with errors):**
```
Asset Validation Report
============================================================

❌ Errors (2):
  1. Card ID cannot be empty
  2. Duplicate card ID: 'goblin_001'

⚠️  Warnings (1):
  1. Script file not found: scripts/missing.rhai

============================================================
❌ Validation failed with 2 error(s)
```

### 4. Compile Game Artifact

Create a production-ready `.ccpack` file:

```bash
# Compile with validation
cardinal-cli compile pack my-game/ dist/game.ccpack --verbose

# Skip validation for faster builds (not recommended for production)
cardinal-cli compile pack my-game/ dist/game.ccpack --no-validate
```

**Output:**
```
Compiling pack...
  Input: my-game/
  Output: dist/game.ccpack

Validating pack...
  ✓ Validation passed

Building pack...
✓ Pack built successfully: dist/game.ccpack
  Pack ID: my-game
  Version: 1.0.0
  Files: 15
```

### 5. Test the Artifact

Verify the pack loads correctly:

```bash
cardinal-cli test pack dist/game.ccpack --verbose
```

**Output:**
```
Testing pack loading...
  Pack: dist/game.ccpack
  ✓ Pack loaded successfully
  Pack ID: my-game
  Version: 1.0.0
  Files: 15

Pack test completed!
  Cards: 10
  Scripts: 5
  Total files: 15
```

## Frontend Integration Patterns

### Pattern 1: Web Frontend (JavaScript/TypeScript)

```javascript
// 1. Load the .ccpack file
const response = await fetch('game.ccpack');
const packBuffer = await response.arrayBuffer();

// 2. Use Cardinal WASM/JS bindings (when available)
import { loadPack, GameEngine } from 'cardinal-wasm';

const pack = loadPack(packBuffer);
const engine = new GameEngine(pack, seed);

// 3. Start the game
engine.startGame(deck1, deck2);

// 4. Apply actions and handle events
const result = engine.applyAction(playerId, action);
for (const event of result.events) {
  // Update UI based on events
  updateUI(event);
}
```

### Pattern 2: Mobile App (React Native / Flutter)

```dart
// Flutter example
import 'package:cardinal_bindings/cardinal.dart';

// 1. Load pack from assets
final packData = await rootBundle.load('assets/game.ccpack');

// 2. Initialize engine
final engine = CardinalEngine.fromPack(packData, seed: 42);

// 3. Game loop
void playCard(int cardId) {
  final result = engine.applyAction(
    playerId,
    PlayCardAction(cardId: cardId),
  );
  
  setState(() {
    // Update game state based on events
    for (var event in result.events) {
      handleEvent(event);
    }
  });
}
```

### Pattern 3: Desktop App (Electron / Tauri)

```rust
// Tauri backend example
use cardinal_kernel::{GameEngine, load_game_config, CardSource};
use std::path::PathBuf;

#[tauri::command]
fn load_game(pack_path: String) -> Result<GameState, String> {
    // Load from .ccpack file
    let source = CardSource::Pack(PathBuf::from(pack_path));
    let ruleset = load_game_config("rules.toml", Some(vec![source]))
        .map_err(|e| e.to_string())?;
    
    let state = GameState::from_ruleset(&ruleset);
    Ok(state)
}

#[tauri::command]
fn apply_action(
    engine: State<GameEngine>,
    player_id: u8,
    action: Action,
) -> Result<Vec<Event>, String> {
    let result = engine.apply_action(
        PlayerId(player_id),
        action,
    ).map_err(|e| format!("{:?}", e))?;
    
    Ok(result.events)
}
```

### Pattern 4: AI/Bot Integration

```python
# Python example (using potential Python bindings)
import cardinal

# Load the game
engine = cardinal.GameEngine.from_pack("game.ccpack", seed=42)

# AI decision loop
while not game_over:
    # Get game state
    state = engine.get_state()
    
    # AI decides action
    action = ai_agent.decide(state)
    
    # Apply action
    result = engine.apply_action(player_id, action)
    
    # Learn from events
    for event in result.events:
        ai_agent.learn(event)
```

## CI/CD Integration

Add Cardinal validation to your CI/CD pipeline:

```yaml
# .github/workflows/validate-game.yml
name: Validate Game Assets

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Build Cardinal CLI
        run: cargo build --release --bin cardinal-cli
      
      - name: Validate Game Assets
        run: |
          ./target/release/cardinal-cli validate rules rules.toml
          ./target/release/cardinal-cli validate pack my-game/
      
      - name: Compile Pack
        run: |
          ./target/release/cardinal-cli compile pack my-game/ dist/game.ccpack --verbose
      
      - name: Test Pack
        run: |
          ./target/release/cardinal-cli test pack dist/game.ccpack
      
      - name: Upload Artifact
        uses: actions/upload-artifact@v3
        with:
          name: game-pack
          path: dist/game.ccpack
```

## Development Workflow

### Iterative Development

```bash
# 1. Make changes to cards
vim cards/new_card.toml

# 2. Quick validation
cardinal-cli validate card cards/new_card.toml

# 3. Test in context
cardinal-cli validate cards-dir cards/

# 4. Rebuild pack
cardinal-cli compile pack my-game/ dist/game.ccpack --verbose

# 5. Test changes
cardinal-cli test game --rules rules.toml
```

### Debug Failed Validation

When validation fails:

```bash
# Get detailed error report
cardinal-cli validate pack my-game/
```

Example error output helps you fix issues:
```
Asset Validation Report
============================================================

❌ Errors (1):
  1. Duplicate card ID 'goblin_scout' found: 'Goblin Scout' and 'Goblin Scout v2'

⚠️  Warnings (1):
  1. Creature card 'Dragon' missing toughness stat

============================================================
❌ Validation failed with 1 error(s)
```

Fix the issues and re-validate.

## Pack Distribution

### Option 1: Direct File Distribution

Simply distribute the `.ccpack` file:

```bash
# Upload to CDN
aws s3 cp dist/game.ccpack s3://my-bucket/games/game-v1.0.0.ccpack

# Frontend loads directly
fetch('https://cdn.example.com/games/game-v1.0.0.ccpack')
```

### Option 2: Embedded in Application

Include the `.ccpack` in your app bundle:

```javascript
// Web: Include as asset
import gamePack from './assets/game.ccpack';

// Mobile: Add to assets/resources
// Desktop: Bundle with app
```

### Option 3: API Endpoint

Serve packs via API:

```javascript
// Backend
app.get('/api/packs/:packId', async (req, res) => {
  const pack = await loadPack(req.params.packId);
  res.sendFile(pack.path);
});

// Frontend
const response = await fetch('/api/packs/starter-set');
const packData = await response.arrayBuffer();
```

## Best Practices

### 1. Always Validate Before Compiling

```bash
# Good: Validate first
cardinal-cli validate pack my-game/
cardinal-cli compile pack my-game/ dist/game.ccpack

# Bad: Skip validation and discover errors at runtime
cardinal-cli compile pack my-game/ dist/game.ccpack --no-validate
```

### 2. Use Versioning

```toml
# pack.toml
pack_id = "my-game"
version = "1.2.3"  # Semantic versioning
```

### 3. Test After Compilation

```bash
# Always test the compiled pack
cardinal-cli test pack dist/game.ccpack
```

### 4. Automate with Scripts

```bash
#!/bin/bash
# scripts/build-and-test.sh

set -e

echo "Validating..."
cardinal-cli validate pack my-game/

echo "Compiling..."
cardinal-cli compile pack my-game/ dist/game.ccpack --verbose

echo "Testing..."
cardinal-cli test pack dist/game.ccpack

echo "✓ Build successful!"
```

### 5. Use in CI/CD

Integrate validation into your CI pipeline to catch errors early.

## Troubleshooting

### "Pack validation failed"

Run detailed validation to see specific errors:
```bash
cardinal-cli validate pack my-game/
```

### "Script compilation failed"

Check script syntax:
```bash
cardinal-cli validate script my-game/scripts/problematic.rhai
```

### "Duplicate card ID"

Search for duplicate IDs in your cards:
```bash
grep -r "^id = " my-game/cards/
```

### "File not found"

Ensure all referenced script files exist:
```bash
# Check script_path references in card files
grep -r "script_path" my-game/cards/
```

## Summary

With the enhanced Cardinal CLI tooling, frontend developers can:

1. ✅ **Validate** all game assets before integration
2. ✅ **Compile** optimized `.ccpack` artifacts
3. ✅ **Test** game functionality independently
4. ✅ **Distribute** single-file game bundles
5. ✅ **Integrate** with any frontend technology

No custom tooling required - just use the Cardinal CLI!

## Next Steps

- See [CLI_USAGE_GUIDE.md](CLI_USAGE_GUIDE.md) for complete CLI reference
- Read [PACK_SYSTEM.md](../PACK_SYSTEM.md) for pack format details
- Check [SCRIPTING_GUIDE.md](../SCRIPTING_GUIDE.md) for card scripting
- Review [ARCHITECTURE.md](../ARCHITECTURE.md) for engine design
