## Plan: Hybrid Card System (Rhai + Data-Driven)

**Goal**: Enable cards to be defined either as pure TOML data or via Rhai scripts, allowing both non-coders and scripters to create custom effects.

### Steps

1. **Create scripting infrastructure**
   - Add `rhai = "1.x"` to `Cargo.toml` (with `no_std`/deterministic options as feasible)
   - Add `crates/cardinal/src/engine/scripting.rs` with a `RhaiEngine` wrapper that owns a shared `rhai::Engine` and runtime context
   - Expose `register_card_script()` and `execute_card_script()` helpers

2. **Extend effect system to support scripted effects**
   - Keep `EffectRef` enum and ensure `Scripted(String)` is used for scripted effects
   - Introduce an `EffectSource` marker (Builtin | DataDriven | Scripted) if helpful for clarity
   - Update `effect_to_command()` to build `EffectRef::Scripted` for scripted abilities and pass through parsed params for data-driven effects

3. **Build effect execution engine**
   - Add `crates/cardinal/src/engine/effect_executor.rs` with `execute_effect()`
   - Execute three kinds: builtin (damage/draw/gain_life/pump), data-driven (TOML params), scripted (Rhai)
   - Wire into stack resolution so effects are actually applied (not just emitted as names); emit `StackResolved` after application

4. **Update card loading to support Rhai scripts**
   - In `schema.rs`, add optional `script_path` to `CardDef`
   - In `build_registry()`, if `script_path` is present, load & compile the Rhai script once, register into `RhaiEngine`, and mark the card as scripted; TOML-only cards stay as-is (backward compatible)

5. **Define Rhai card API**
   - Script entrypoint: `fn execute_ability(ctx) -> Vec<Command>` (or similar)
   - Expose safe helpers to Rhai: `deal_damage(target, amount)`, `draw_cards(player, count)`, `gain_life(player, amount)`, `pump_creature(card, p, t)`
   - Provide read-only accessors to game state and card info needed to decide targets/values

6. **Improve triggers & choices**
   - Align trigger matching with `trigger_kinds` (no hard-coded strings); map engine events to trigger keys
   - Flesh out choice/target pipeline so scripted/data-driven effects can request and validate targets before execution

7. **Documentation**
   - Add a "Scripting Guide" section to `crates/cardinal/README.md` with Rhai examples
   - Update `ARCHITECTURE.md` to describe the hybrid flow (TOML vs Rhai cards)
   - Add TOML examples showing `script_path` and data-only cards

### Considerations

- **Determinism & Safety**: Configure Rhai to disallow non-deterministic ops; set operation limits; no IO/threads/time; scripts validated at load.
- **Performance**: Scripted abilities cost more than builtins; data-driven remains as fast as current. Acceptable trade for flexibility.
- **Backward compatibility**: Existing TOML cards continue to work; `script_path` is optional.
- **Phasing**: Deliver in two phases—(A) solidify effect execution for data-driven cards; (B) add Rhai scripting once execution pipeline is stable.
- **Migration**: None required; cards choose TOML-only or TOML + Rhai per card.
