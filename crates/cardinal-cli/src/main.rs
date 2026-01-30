use cardinal::*;
use cardinal::ids::PlayerId;
use cardinal::model::action::Action;
use clap::{Parser, Subcommand};
use std::io::{self, BufRead};

#[derive(Parser)]
#[command(name = "cardinal")]
#[command(about = "Cardinal - A Rules Engine TCG", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Play the game interactively
    Play {
        /// Path to rules.toml file
        #[arg(short, long, default_value = "./rules.toml")]
        rules: String,
    },
    /// Build a .ccpack file from a directory
    BuildPack {
        /// Input directory containing pack.toml
        input: String,
        /// Output .ccpack file path
        output: String,
    },
    /// List contents of a .ccpack file
    ListPack {
        /// Path to .ccpack file
        pack: String,
    },
    /// Unpack a .ccpack file to a directory
    UnpackPack {
        /// Path to .ccpack file
        pack: String,
        /// Output directory
        output: String,
    },
    /// Validate game assets (rules, cards, scripts, or packs)
    Validate {
        #[command(subcommand)]
        target: ValidateTarget,
    },
    /// Compile game assets into optimized artifacts
    Compile {
        #[command(subcommand)]
        target: CompileTarget,
    },
    /// Test and simulate game scenarios
    Test {
        #[command(subcommand)]
        target: TestTarget,
    },
}

#[derive(Subcommand)]
enum ValidateTarget {
    /// Validate a rules.toml file
    Rules {
        /// Path to rules.toml file
        path: String,
    },
    /// Validate a single card TOML file
    Card {
        /// Path to card .toml file
        path: String,
    },
    /// Validate a cards directory
    CardsDir {
        /// Path to cards directory
        path: String,
    },
    /// Validate a cards.toml file
    CardsFile {
        /// Path to cards.toml file
        path: String,
    },
    /// Validate a Rhai script file
    Script {
        /// Path to .rhai script file
        path: String,
    },
    /// Validate a pack directory before building
    Pack {
        /// Path to pack directory
        path: String,
    },
}

#[derive(Subcommand)]
enum CompileTarget {
    /// Compile a pack directory into a .ccpack file (with validation)
    Pack {
        /// Input directory containing pack.toml
        input: String,
        /// Output .ccpack file path
        output: String,
        /// Skip validation before compiling
        #[arg(long)]
        no_validate: bool,
        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },
}

#[derive(Subcommand)]
enum TestTarget {
    /// Run a basic game simulation test
    Game {
        /// Path to rules.toml file
        #[arg(short, long, default_value = "./rules.toml")]
        rules: String,
        /// Random seed for deterministic testing
        #[arg(short, long, default_value = "42")]
        seed: u64,
        /// Number of cards in starting hand for testing
        #[arg(long, default_value = "5")]
        hand_size: usize,
        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Test loading a .ccpack file
    Pack {
        /// Path to .ccpack file
        pack: String,
        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Play { rules }) => {
            run_game(&rules);
        }
        Some(Commands::BuildPack { input, output }) => {
            if let Err(e) = cardinal::pack::build_pack(&input, &output) {
                eprintln!("Error building pack: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::ListPack { pack }) => {
            if let Err(e) = cardinal::pack::list_pack(&pack) {
                eprintln!("Error listing pack: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::UnpackPack { pack, output }) => {
            if let Err(e) = cardinal::pack::unpack_pack(&pack, &output) {
                eprintln!("Error unpacking: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Validate { target }) => {
            handle_validation(target);
        }
        Some(Commands::Compile { target }) => {
            handle_compilation(target);
        }
        Some(Commands::Test { target }) => {
            handle_testing(target);
        }
        None => {
            // Default: run the game with default rules
            run_game("./rules.toml");
        }
    }
}

fn run_game(rules_path: &str) {
    println!("Welcome to Cardinal - A Rules Engine TCG!");
    println!();

    // Load rules and cards
    let rules = match cardinal::load_game_config(rules_path, None) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to load game config: {:?}", e);
            return;
        }
    };

    println!("✓ Rules loaded: {}", rules.game.name);
    println!("✓ Cards loaded: {}", rules.cards.len());
    println!();

    // Create initial game state
    let initial_state = GameState::from_ruleset(&rules);
    println!("✓ Game state created");

    // Create test decks (for demo)
    let mut state = initial_state.clone();
    populate_test_decks(&mut state, 5);
    println!("✓ Test decks populated");

    // Initialize the game
    let state = cardinal::initialize_game(state, &rules, 42);
    println!("✓ Game initialized");
    println!();

    // Create game engine
    let mut engine = GameEngine::new(rules, 42, state);
    let mut display = GameDisplay::new();

    println!("═══════════════════════════════════════════════════════════");
    println!("Game starting! You are Player 0");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    // Game loop
    let stdin = io::stdin();
    let reader = stdin.lock();
    let mut lines = reader.lines();

    loop {
        // Render current game state
        let viewer = PlayerId(0);
        println!("{}", display.render_game(&engine.state, &engine.cards, viewer));
        println!();

        let is_active = engine.state.turn.active_player == viewer;
        let is_priority = engine.state.turn.priority_player == viewer;

        // Show menu
        println!("{}", display.render_menu(is_active, is_priority));
        println!();

        // Get input
        print!("> ");
        use std::io::Write;
        io::stdout().flush().unwrap();

        if let Some(Ok(input)) = lines.next() {
            let choice = input.trim();

            match choice {
                "1" => {
                    if !is_active || !is_priority {
                        println!("You cannot play cards right now.");
                        continue;
                    }
                    handle_play_card(&mut engine, &mut display, viewer);
                }
                "2" => {
                    handle_view_hand(&engine, &display, viewer);
                }
                "3" => {
                    handle_view_field(&engine, viewer);
                }
                "4" => {
                    handle_view_opponent_field(&engine, viewer);
                }
                "5" => {
                    handle_view_log(&display);
                }
                "6" => {
                    if !is_priority {
                        println!("You do not have priority.");
                        continue;
                    }
                    handle_pass_priority(&mut engine, &mut display, viewer);
                }
                "7" => {
                    println!("You have conceded. Game over!");
                    break;
                }
                _ => {
                    println!("Invalid choice. Try again.");
                }
            }

            println!();
        } else {
            break;
        }
    }

    println!();
    println!("Thanks for playing!");
}

fn populate_test_decks(state: &mut GameState, num_cards: usize) {
    let num_players = state.players.len() as u8;
    for player_idx in 0..num_players {
        let _player_id = PlayerId(player_idx);
        let deck_zone_id = format!("deck@{}", player_idx);

        if let Some(deck) = state.zones.iter_mut().find(|z| z.id.0 == deck_zone_id) {
            for i in 0..num_cards {
                let card_id = cardinal::ids::CardId((player_idx as u32 * 100) + i as u32);
                deck.cards.push(card_id);
            }
        }
    }
}

fn handle_play_card(engine: &mut GameEngine, display: &mut GameDisplay, player: PlayerId) {
    // Get hand zone
    let hand_zone_id = format!("hand@{}", player.0);
    let hand_cards: Vec<_> = engine.state.zones.iter()
        .find(|z| z.id.0 == hand_zone_id)
        .map(|z| z.cards.clone())
        .unwrap_or_default();

    if hand_cards.is_empty() {
        println!("Your hand is empty!");
        return;
    }

    println!("Select a card to play:");
    for (idx, card_id) in hand_cards.iter().enumerate() {
        if let Some(card_def) = engine.cards.get(&card_id.0) {
            println!("  [{}] {} ({})", idx + 1, card_def.name, card_def.card_type);
        } else {
            println!("  [{}] Card #{}", idx + 1, card_id.0);
        }
    }
    print!("> ");
    use std::io::Write;
    io::stdout().flush().unwrap();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        if let Ok(idx) = input.trim().parse::<usize>() {
            if idx > 0 && idx <= hand_cards.len() {
                let card_id = hand_cards[idx - 1];
                let from_zone = cardinal::ids::ZoneId(Box::leak(hand_zone_id.into_boxed_str()));

                match engine.apply_action(player, Action::PlayCard { card: card_id, from: from_zone }) {
                    Ok(result) => {
                        if let Some(card_def) = engine.cards.get(&card_id.0) {
                            display.log(
                                engine.state.turn.number,
                                &engine.state.turn.phase.0,
                                &engine.state.turn.step.0,
                                format!("You played: {}", card_def.name),
                            );
                        }
                        for event in &result.events {
                            match event {
                                Event::CardPlayed { .. } => {
                                    display.log(
                                        engine.state.turn.number,
                                        &engine.state.turn.phase.0,
                                        &engine.state.turn.step.0,
                                        "Card entered play".to_string(),
                                    );
                                }
                                Event::CardMoved { .. } => {
                                    display.log(
                                        engine.state.turn.number,
                                        &engine.state.turn.phase.0,
                                        &engine.state.turn.step.0,
                                        "Card moved to field".to_string(),
                                    );
                                }
                                Event::StackResolved { .. } => {
                                    display.log(
                                        engine.state.turn.number,
                                        &engine.state.turn.phase.0,
                                        &engine.state.turn.step.0,
                                        "Stack item resolved".to_string(),
                                    );
                                }
                                _ => {}
                            }
                        }
                        println!("Card played!");
                    }
                    Err(e) => {
                        println!("Cannot play card: {:?}", e);
                    }
                }
                return;
            }
        }
    }

    println!("Invalid selection.");
}

fn handle_view_hand(engine: &GameEngine, display: &GameDisplay, player: PlayerId) {
    let hand_zone_id = format!("hand@{}", player.0);
    let hand_cards: Vec<_> = engine.state.zones.iter()
        .find(|z| z.id.0 == hand_zone_id)
        .map(|z| z.cards.clone())
        .unwrap_or_default();

    if hand_cards.is_empty() {
        println!("Your hand is empty!");
        return;
    }

    println!();
    for (idx, card_id) in hand_cards.iter().enumerate() {
        println!("{}", display.render_card_detail(&engine.cards, *card_id));
        if idx < hand_cards.len() - 1 {
            println!();
        }
    }
    println!();
}

fn handle_view_field(engine: &GameEngine, player: PlayerId) {
    let field_zone_id = format!("field@{}", player.0);
    let field = engine.state.zones.iter()
        .find(|z| z.id.0 == field_zone_id);

    println!();
    if let Some(zone) = field {
        if zone.cards.is_empty() {
            println!("Your field is empty!");
        } else {
            for (idx, card_id) in zone.cards.iter().enumerate() {
                if let Some(card_def) = engine.cards.get(&card_id.0) {
                    println!("[{}] {} ({})", idx + 1, card_def.name, card_def.card_type);
                } else {
                    println!("[{}] Card #{}", idx + 1, card_id.0);
                }
            }
        }
    }
    println!();
}

fn handle_view_opponent_field(engine: &GameEngine, player: PlayerId) {
    let opponent = if player.0 == 0 { PlayerId(1) } else { PlayerId(0) };
    let field_zone_id = format!("field@{}", opponent.0);
    let field = engine.state.zones.iter()
        .find(|z| z.id.0 == field_zone_id);

    println!();
    if let Some(zone) = field {
        if zone.cards.is_empty() {
            println!("Opponent field is empty!");
        } else {
            for (idx, _) in zone.cards.iter().enumerate() {
                println!("[{}] Mystery Creature", idx + 1);
            }
        }
    }
    println!();
}

fn handle_view_log(display: &GameDisplay) {
    println!();
    println!("{}", display.render_log(20));
    println!();
}

fn handle_pass_priority(engine: &mut GameEngine, display: &mut GameDisplay, player: PlayerId) {
    match engine.apply_action(player, Action::PassPriority) {
        Ok(_result) => {
            display.log(
                engine.state.turn.number,
                &engine.state.turn.phase.0,
                &engine.state.turn.step.0,
                format!("Player {:?} passed priority", player),
            );
            println!("You passed priority.");
        }
        Err(e) => {
            println!("Cannot pass priority: {:?}", e);
        }
    }
}

fn handle_validation(target: ValidateTarget) {
    use cardinal::validation::*;

    let result = match target {
        ValidateTarget::Rules { path } => {
            println!("Validating rules file: {}", path);
            match validate_rules(&path) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Validation error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        ValidateTarget::Card { path } => {
            println!("Validating card file: {}", path);
            match validate_card(&path) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Validation error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        ValidateTarget::CardsDir { path } => {
            println!("Validating cards directory: {}", path);
            match validate_cards_dir(&path) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Validation error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        ValidateTarget::CardsFile { path } => {
            println!("Validating cards file: {}", path);
            match validate_cards_file(&path) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Validation error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        ValidateTarget::Script { path } => {
            println!("Validating script file: {}", path);
            match validate_script(&path) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Validation error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        ValidateTarget::Pack { path } => {
            println!("Validating pack directory: {}", path);
            match validate_pack(&path) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Validation error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    };

    print_validation_result(&result, "Asset");

    if !result.is_valid {
        std::process::exit(1);
    }
}

fn handle_compilation(target: CompileTarget) {
    use cardinal::compile::*;

    match target {
        CompileTarget::Pack { input, output, no_validate, verbose } => {
            let options = CompileOptions {
                validate: !no_validate,
                verbose,
            };

            if let Err(e) = compile_pack(&input, &output, options) {
                eprintln!("Compilation error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn handle_testing(target: TestTarget) {
    use cardinal::testing::*;

    match target {
        TestTarget::Game { rules, seed, hand_size, verbose } => {
            let options = TestOptions {
                seed,
                starting_hand_size: hand_size,
                verbose,
            };

            match run_basic_test(&rules, options) {
                Ok(summary) => {
                    println!("\n{}", summary);
                }
                Err(e) => {
                    eprintln!("Test error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        TestTarget::Pack { pack, verbose } => {
            match test_pack_loading(&pack, verbose) {
                Ok(summary) => {
                    println!("\n{}", summary);
                }
                Err(e) => {
                    eprintln!("Test error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}

