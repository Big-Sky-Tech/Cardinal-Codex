crates/cardinal/src/
        lib.rs
        error.rs
        ids.rs
    state/
        mod.rs
        gamestate.rs
        zones.rs
    rules/
        mod.rs
        schema.rs        # loaded from TOML (phases/zones/action defs)
        query.rs         # what engine asks rules/mods
    engine/
        mod.rs
        core.rs          # GameEngine struct
        reducer.rs       # apply action -> state changes
        legality.rs      # validate actions
        events.rs
    model/
        mod.rs
        action.rs
        event.rs
        command.rs
        choice.rs
        card.rs
    util/
        rng.rs
        log.rs
