use cardinal::*;
use cardinal::ids::{CardId, PlayerId};
use cardinal_kernel as cardinal;

fn load_test_rules() -> cardinal::Ruleset {
    cardinal::load_game_config("../../rules.toml", None).expect("load game config")
}

fn build_demo_decks(rules: &cardinal::Ruleset, deck_size: usize) -> Vec<Vec<CardId>> {
    let card_ids: Vec<CardId> = rules.cards.iter()
        .filter_map(|card| card.id.parse::<u32>().ok().map(CardId))
        .collect();

    assert!(!card_ids.is_empty(), "expected numeric test card IDs");

    (0..rules.players.min_players)
        .map(|_| {
            (0..deck_size)
                .map(|index| card_ids[index % card_ids.len()])
                .collect()
        })
        .collect()
}

fn build_single_card_decks(rules: &cardinal::Ruleset, card_id: CardId, deck_size: usize) -> Vec<Vec<CardId>> {
    (0..rules.players.min_players)
        .map(|_| vec![card_id; deck_size])
        .collect()
}

fn start_engine(rules: cardinal::Ruleset, seed: u64, decks: Vec<Vec<CardId>>) -> GameEngine {
    let mut engine = GameEngine::new(rules, seed);
    engine.start_game(decks).expect("start game");
    engine
}

fn advance_to_play_card(engine: &mut GameEngine) -> (PlayerId, Action) {
    for _ in 0..64 {
        let player = engine.state().turn.priority_player;
        let actions = engine.legal_actions(player);
        if let Some(action) = actions.into_iter().find(|action| matches!(action, Action::PlayCard { .. })) {
            return (player, action);
        }

        engine.apply_action(player, Action::PassPriority)
            .expect("advance priority");
    }

    panic!("no playable card action found");
}

fn hand_zone_card_count(state: &GameState, player: PlayerId) -> usize {
    let hand_zone_id = format!("hand@{}", player.0);
    state.zones.iter()
        .find(|zone| zone.id.0 == hand_zone_id)
        .map(|zone| zone.cards.len())
        .unwrap_or(0)
}

#[test]
fn build_engine_from_rules() {
    let rules = load_test_rules();
    let engine = GameEngine::new(rules, 42);
    assert!(!engine.state().players.is_empty());
}

#[test]
fn start_game_draws_hands_and_sets_priority() {
    let rules = load_test_rules();
    let engine = start_engine(rules.clone(), 42, build_demo_decks(&rules, 10));
    let state = engine.state();

    assert_eq!(state.turn.active_player, state.turn.priority_player);

    let first_player = state.turn.active_player;
    for player in &state.players {
        let hand_size = hand_zone_card_count(state, player.id);
        if player.id == first_player && rules.turn.skip_first_turn_draw_for_first_player {
            assert_eq!(hand_size, 0);
        } else {
            assert_eq!(hand_size, rules.players.starting_hand_size);
        }
    }
}

#[test]
fn deterministic_startup_uses_seed() {
    let rules = load_test_rules();
    let decks = build_demo_decks(&rules, 10);

    let engine_a = start_engine(rules.clone(), 42, decks.clone());
    let engine_b = start_engine(rules.clone(), 42, decks.clone());
    let engine_c = start_engine(rules, 99, decks);

    assert_eq!(engine_a.state().turn.active_player, engine_b.state().turn.active_player);
    assert_eq!(engine_a.player_view(PlayerId(0)).zones.len(), engine_b.player_view(PlayerId(0)).zones.len());
    assert!(engine_c.state().turn.active_player.0 < 2);
}

#[test]
fn public_state_hides_private_zones() {
    let rules = load_test_rules();
    let engine = start_engine(rules.clone(), 42, build_demo_decks(&rules, 10));
    let public_state = engine.public_state();

    for zone in public_state.zones.iter().filter(|zone| zone.id.0.starts_with("hand@") || zone.id.0.starts_with("deck@")) {
        assert!(zone.cards.is_empty(), "private zones should be hidden in public state");
    }
}

#[test]
fn player_view_hides_opponent_private_zones() {
    let rules = load_test_rules();
    let engine = start_engine(rules.clone(), 42, build_demo_decks(&rules, 10));

    let player_zero = engine.player_view(PlayerId(0));
    let player_one = engine.player_view(PlayerId(1));

    assert_eq!(hand_zone_card_count(&player_zero, PlayerId(1)), 0);
    assert_eq!(hand_zone_card_count(&player_one, PlayerId(0)), 0);
}

#[test]
fn legal_actions_include_pass_for_priority_player() {
    let rules = load_test_rules();
    let engine = start_engine(rules.clone(), 42, build_demo_decks(&rules, 10));
    let priority_player = engine.state().turn.priority_player;

    assert!(engine.legal_actions(priority_player).iter().any(|action| matches!(action, Action::PassPriority)));
}

#[test]
fn legal_actions_exclude_pass_for_non_priority_player() {
    let rules = load_test_rules();
    let engine = start_engine(rules.clone(), 42, build_demo_decks(&rules, 10));
    let non_priority_player = if engine.state().turn.priority_player == PlayerId(0) {
        PlayerId(1)
    } else {
        PlayerId(0)
    };

    assert!(!engine.legal_actions(non_priority_player).iter().any(|action| matches!(action, Action::PassPriority)));
}

#[test]
fn legal_actions_include_play_card_when_available() {
    let rules = load_test_rules();
    let mut engine = start_engine(rules.clone(), 42, build_demo_decks(&rules, 10));

    let (player, action) = advance_to_play_card(&mut engine);
    assert!(engine.legal_actions(player).iter().any(|candidate| matches!(
        (candidate, &action),
        (Action::PlayCard { card: lhs_card, from: lhs_from }, Action::PlayCard { card: rhs_card, from: rhs_from })
            if lhs_card == rhs_card && lhs_from == rhs_from
    )));
}

#[test]
fn inactive_player_cannot_play_card() {
    let rules = load_test_rules();
    let engine = start_engine(rules.clone(), 42, build_demo_decks(&rules, 10));
    let inactive_player = if engine.state().turn.active_player == PlayerId(0) {
        PlayerId(1)
    } else {
        PlayerId(0)
    };
    let inactive_view = engine.player_view(inactive_player);
    let hand_zone = inactive_view.zones.iter()
        .find(|zone| zone.id.0 == format!("hand@{}", inactive_player.0))
        .expect("inactive hand zone");

    let result = {
        let card = hand_zone.cards.first().copied().unwrap_or(CardId(999));
        let action = Action::PlayCard { card, from: hand_zone.id.clone() };
        let mut engine = engine;
        engine.apply_action(inactive_player, action)
    };

    assert!(result.is_err(), "inactive player should not be able to play cards");
}

#[test]
fn cannot_play_from_opponents_zone() {
    let rules = load_test_rules();
    let engine = start_engine(rules.clone(), 42, build_demo_decks(&rules, 10));
    let active_player = engine.state().turn.active_player;
    let opponent = if active_player == PlayerId(0) { PlayerId(1) } else { PlayerId(0) };
    let opponent_view = engine.player_view(opponent);
    let opponent_hand = opponent_view.zones.iter()
        .find(|zone| zone.id.0 == format!("hand@{}", opponent.0))
        .expect("opponent hand zone");

    let result = {
        let action = Action::PlayCard { card: CardId(999), from: opponent_hand.id.clone() };
        let mut engine = engine;
        engine.apply_action(active_player, action)
    };

    assert!(result.is_err(), "active player cannot play from opponent zones");
}

#[test]
fn play_card_emits_events_and_moves_card() {
    let rules = load_test_rules();
    let mut engine = start_engine(rules.clone(), 42, build_demo_decks(&rules, 10));
    let (player, action) = advance_to_play_card(&mut engine);

    let played_card = match action.clone() {
        Action::PlayCard { card, .. } => card,
        _ => panic!("expected play card action"),
    };

    let result = engine.apply_action(player, action).expect("play card");

    assert!(result.events.iter().any(|event| matches!(event, Event::CardPlayed { player: event_player, card } if *event_player == player && *card == played_card)));
    assert!(result.events.iter().any(|event| matches!(event, Event::CardMoved { card, .. } if *card == played_card)));
}

#[test]
fn concede_action_ends_game() {
    let rules = load_test_rules();
    let mut engine = start_engine(rules.clone(), 42, build_demo_decks(&rules, 10));

    let result = engine.apply_action(PlayerId(0), Action::Concede).expect("concede");

    assert!(result.events.iter().any(|event| matches!(event, Event::GameEnded { .. })));
    assert!(engine.state().ended.is_some());
}

#[test]
fn phase_advancement_requires_full_priority_round() {
    let rules = load_test_rules();
    let mut engine = start_engine(rules.clone(), 42, build_demo_decks(&rules, 10));
    let initial_phase = engine.state().turn.phase.clone();
    let initial_step = engine.state().turn.step.clone();

    for _ in 0..engine.state().players.len() {
        let player = engine.state().turn.priority_player;
        engine.apply_action(player, Action::PassPriority).expect("pass priority");
    }

    assert!(engine.state().turn.phase != initial_phase || engine.state().turn.step != initial_step);
}

#[test]
fn full_turn_progression_rotates_active_player() {
    let rules = load_test_rules();
    let mut engine = start_engine(rules.clone(), 42, build_demo_decks(&rules, 10));
    let starting_turn = engine.state().turn.number;
    let starting_player = engine.state().turn.active_player;
    let total_steps: usize = rules.turn.phases.iter().map(|phase| phase.steps.len()).sum();

    for _ in 0..(total_steps * engine.state().players.len() + 8) {
        let player = engine.state().turn.priority_player;
        engine.apply_action(player, Action::PassPriority).expect("pass priority");
        if engine.state().turn.number > starting_turn {
            break;
        }
    }

    assert_eq!(engine.state().turn.number, starting_turn + 1);
    assert_ne!(engine.state().turn.active_player, starting_player);
}

#[test]
fn goblin_scout_trigger_resolves_from_runtime_start() {
    let rules = load_test_rules();
    let goblin_id = CardId(1);
    let mut engine = start_engine(rules.clone(), 42, build_single_card_decks(&rules, goblin_id, 10));
    let (player, action) = advance_to_play_card(&mut engine);

    let result = engine.apply_action(player, action).expect("play goblin scout");

    assert!(result.events.iter().any(|event| matches!(event, Event::CardPlayed { card, .. } if *card == goblin_id)));
    assert!(result.events.iter().any(|event| matches!(event, Event::StackResolved { .. })));
}

#[test]
fn card_registry_lookup_works() {
    let rules = load_test_rules();
    let engine = GameEngine::new(rules, 42);
    let goblin = cardinal::engine::cards::get_card(engine.cards(), CardId(1)).expect("goblin scout");

    assert_eq!(goblin.name, "Goblin Scout");
    assert_eq!(goblin.card_type, "creature");
    assert!(!goblin.abilities.is_empty());
}

#[test]
fn runtime_wrapper_drives_one_action() {
    let mut runtime = GameRuntime::load("../../rules.toml", None, 42).expect("load runtime");
    let decks = runtime.build_mirror_decks(10).expect("build decks");
    runtime.start_game(decks).expect("start runtime game");

    let player = runtime.public_state().turn.priority_player;
    let action = runtime.legal_actions(player).into_iter()
        .find(|action| !matches!(action, Action::Concede))
        .expect("at least one non-concede action");

    let result = runtime.apply_action(player, action).expect("apply runtime action");
    assert!(!result.events.is_empty());
}
