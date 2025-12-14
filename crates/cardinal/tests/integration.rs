use cardinal::*;

#[test]
fn build_engine_from_rules() {
    let rules = load_rules("../../rules.toml").expect("load rules");
    let engine = GameEngine::from_ruleset(rules, 42);
    // basic sanity: at least one player present
    assert!(!engine.state.players.is_empty());
}

