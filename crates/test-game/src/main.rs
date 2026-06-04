use cardinal::ids::PlayerId;
use cardinal::{Action, GameRuntime};

fn main() {
    let rules_path = "crates/test-game/rules.toml";
    let seed = 42;

    println!("====================================");
    println!("Cardinal Minimal Runtime Test");
    println!("====================================\n");
    println!("Loading rules from: {}", rules_path);

    let mut runtime = match GameRuntime::load(rules_path, None, seed) {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("ERROR: Failed to load game config: {:?}", error);
            eprintln!("\nMake sure you're running from the repository root:");
            eprintln!("  cargo run --bin test-game");
            return;
        }
    };

    let decks = match runtime.build_mirror_decks(5) {
        Ok(decks) => decks,
        Err(error) => {
            eprintln!("ERROR: Failed to build demo decks: {:?}", error);
            return;
        }
    };

    if let Err(error) = runtime.start_game(decks) {
        eprintln!("ERROR: Failed to start game: {:?}", error);
        return;
    }

    let state = runtime.public_state();
    println!("✓ Game loaded");
    println!("✓ Players: {}", state.players.len());
    println!("✓ Turn: {}", state.turn.number);
    println!("✓ Phase: {}", state.turn.phase.0);
    println!("✓ Step: {}", state.turn.step.0);
    println!("✓ Active Player: Player {}", state.turn.active_player.0);
    println!();

    let player = state.turn.priority_player;
    let legal_actions = runtime.legal_actions(player);
    println!("Legal actions for Player {}:", player.0);
    for (index, action) in legal_actions.iter().enumerate() {
        println!("  [{}] {}", index + 1, describe_action(action));
    }
    println!();

    let chosen_action = legal_actions.into_iter()
        .find(|action| !matches!(action, Action::Concede))
        .unwrap_or(Action::Concede);

    println!("Applying action: {}", describe_action(&chosen_action));
    match runtime.apply_action(player, chosen_action) {
        Ok(result) => {
            if result.events.is_empty() {
                println!("No events emitted.");
            } else {
                println!("Events:");
                for event in result.events {
                    println!("  - {:?}", event);
                }
            }
        }
        Err(error) => {
            eprintln!("ERROR: Failed to apply action: {:?}", error);
        }
    }

    println!("\nPlayer 0 view:");
    print_player_view(&runtime.player_view(PlayerId(0)));
}

fn describe_action(action: &Action) -> String {
    match action {
        Action::PassPriority => "PassPriority".to_string(),
        Action::Concede => "Concede".to_string(),
        Action::PlayCard { card, from } => format!("PlayCard(card={}, from={})", card.0, from.0),
        Action::ChooseTarget { choice_id, .. } => format!("ChooseTarget(choice_id={})", choice_id),
    }
}

fn print_player_view(state: &cardinal::GameState) {
    println!("  Turn: {}", state.turn.number);
    println!("  Phase: {}", state.turn.phase.0);
    println!("  Step: {}", state.turn.step.0);
    println!("  Active Player: Player {}", state.turn.active_player.0);

    for player in &state.players {
        println!("  Player {} life: {}", player.id.0, player.life);
    }

    for zone in &state.zones {
        println!("  Zone {} has {} visible cards", zone.id.0, zone.cards.len());
    }
}
