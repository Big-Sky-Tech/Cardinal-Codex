# Copilot Instructions — Cardinal

## Project Overview
**Cardinal** is a headless, deterministic rules engine for turn-based, TCG-like games.

It is designed as a **rules kernel**, not a game client.
Rendering, UI, networking, and AI live outside this crate.

The engine must support:
- Config-driven rules (TOML)
- Swappable mechanics (via modules / scripting)
- Deterministic simulation
- Event-based state changes
- Future plugin / scripting integration (Rust-script / WASM)

## Core Design Principles (DO NOT VIOLATE)

1. **Determinism**
    - Same inputs + same seed = same outputs
    - No system time, threads, randomness, or IO in core logic
    - All randomness must come from the engine-owned RNG

2. **Headless Architecture**
    - No rendering, UI, audio, or engine-specific assumptions
    - The library must be embeddable in any frontend

3. **Actions In, Events Out**
    - External callers submit `Action`
    - Engine validates and applies them
    - Engine emits `Event` records describing what happened

4. **State Is Authoritative**
    - `GameState` is the single source of truth
    - No hidden global state
    - No side effects outside controlled reducers

5. **No Direct State Mutation from Rules**
    - Rules/modules/scripts must return `Command` values
    - The core engine validates and commits commands
    - Never allow plugins to mutate `GameState` directly

6. **Pure-ish Reducers**
    - `apply_action` and command application should be predictable
    - Avoid implicit behavior or “magic” mutations

## Crate Responsibilities

### `cardinal` (the engine crate in `crates/cardinal`)
Responsible for:
- Game state representation
- Turn / phase / priority progression
- Action legality checks
- Command application
- Stack handling
- Event emission
- Pending player choices
- Rules schema interpretation

Not responsible for:
- UI
- Input handling
- Networking
- Persistence
- AI heuristics
- Asset loading

## Code Style Rules

- Prefer **explicit structs and enums** over clever abstractions
- Favor **readability and correctness** over brevity
- Avoid macros unless absolutely necessary
- Avoid `unwrap()` and `expect()` in engine logic
- Errors must be represented as explicit error types
- Prefer newtype IDs (`PlayerId`, `CardId`, etc.) over raw primitives

## Public API Stability

Assume that:
- `Action`, `Event`, `Command`, and `GameState` are public contracts
- Breaking changes to these types are costly
- Add fields instead of removing them where possible

## Action Handling Rules

When implementing new actions:
1. Validate legality in `engine::legality`
2. Apply effects in `engine::reducer`
3. Emit events for **every** meaningful state change
4. Never skip validation
5. Never mutate state silently

## Events

Events are:
- Required for UI animation
- Required for replays
- Required for debugging

If state changes and no event is emitted, that is a bug.

## Pending Choices

When player input is required:
- Engine must enter a paused state
- Emit a `ChoiceRequested` event
- Reject unrelated actions until resolved
- Resume execution only after valid `Choose*` action

## Rules / Plugin Integration

Assume rules may eventually be:
- Native Rust modules
- Rust-script (EXTREMELY LIKELY)
- WASM plugins

Therefore:
- Expose clear, minimal interfaces
- Avoid tight coupling between engine internals and rule logic
- Treat rules as untrusted inputs

## Testing Expectations

Prefer:
- Small, deterministic unit tests
- State + action → expected events
- Tests that simulate entire turns or phases
- No reliance on execution order side effects

## What Copilot Should Do

- Generate conservative, explicit Rust code
- Follow existing architectural patterns
- Extend enums and structs carefully
- Ask for missing context by leaving TODO comments
- Prefer correctness over cleverness

## What Copilot Should NOT Do

- Invent new gameplay rules
- Add UI concepts
- Assume a specific TCG (Magic, Yugioh, etc.)
- Bypass validation logic
- Introduce hidden global state
- Add async or threading primitives

---

**Cardinal treats rules like law.**
Code should read like a statute, not a spell.
