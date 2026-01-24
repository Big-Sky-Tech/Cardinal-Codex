use crate::{
    state::gamestate::GameState,
    util::rng::GameRng,
    ids::{CardId, PlayerId},
    rules::schema::Ruleset,
};

/// Initialize a game by:
/// 1. Shuffling each player's deck
/// 2. Drawing starting hands
/// 3. Determining the first player
/// 4. Setting up the initial turn state
pub fn initialize_game(
    mut state: GameState,
    rules: &Ruleset,
    seed: u64,
) -> GameState {
    let mut rng = GameRng::new(seed);
    let num_players = state.players.len() as u32;

    // 1. Shuffle each player's deck
    for i in 0..num_players {
        let player_id = PlayerId(i as u8);
        shuffle_player_deck(&mut state, player_id, &mut rng, rules);
    }

    // 2. Determine first player based on rule
    let first_player = determine_first_player(&rules.players.first_player_rule, num_players, &mut rng);
    state.turn.active_player = first_player;
    state.turn.priority_player = first_player;

    // 3. Draw starting hands
    let skip_first_draw = rules.turn.skip_first_turn_draw_for_first_player;
    for i in 0..num_players {
        let player_id = PlayerId(i as u8);
        let should_skip = skip_first_draw && player_id == first_player;
        
        if !should_skip {
            draw_cards(&mut state, player_id, rules.players.starting_hand_size as u32, rules);
        }
    }

    state
}

/// Shuffle a player's deck in-place using the provided RNG
fn shuffle_player_deck(
    state: &mut GameState,
    player: PlayerId,
    rng: &mut GameRng,
    _rules: &Ruleset,
) {
    // Find the deck zone for this player
    let deck_zone_id_string = format!("deck@{}", player.0);
    let deck_zone = state.zones.iter_mut()
        .find(|z| z.id.0 == deck_zone_id_string);

    if let Some(zone) = deck_zone {
        // Fisher-Yates shuffle
        for i in (1..zone.cards.len()).rev() {
            let j: usize = rng.generate::<u32>() as usize % (i + 1);
            zone.cards.swap(i, j);
        }
    }
}

/// Determine which player goes first based on the rule
fn determine_first_player(
    rule: &str,
    num_players: u32,
    rng: &mut GameRng,
) -> PlayerId {
    match rule {
        "random" => {
            let idx: u32 = rng.generate::<u32>() % num_players;
            PlayerId(idx as u8)
        }
        "player_0" | "first" => PlayerId(0),
        "player_1" | "second" => {
            if num_players > 1 {
                PlayerId(1)
            } else {
                PlayerId(0)
            }
        }
        _ => {
            // Default to random if unknown rule
            let idx: u32 = rng.generate::<u32>() % num_players;
            PlayerId(idx as u8)
        }
    }
}

/// Draw `count` cards from a player's deck to their hand
fn draw_cards(
    state: &mut GameState,
    player: PlayerId,
    count: u32,
    rules: &Ruleset,
) {
    let deck_zone_id_string = format!("deck@{}", player.0);
    let hand_zone_id_string = format!("hand@{}", player.0);

    // Find deck and hand zones
    let deck_cards: Vec<CardId> = state.zones.iter()
        .find(|z| z.id.0 == deck_zone_id_string)
        .map(|z| z.cards.clone())
        .unwrap_or_default();

    // Draw from the top of deck (first card in the Vec)
    let cards_to_draw = deck_cards.iter()
        .take(count as usize)
        .cloned()
        .collect::<Vec<_>>();

    // Remove from deck
    if let Some(deck_zone) = state.zones.iter_mut()
        .find(|z| z.id.0 == deck_zone_id_string)
    {
        for _ in 0..cards_to_draw.len() {
            if !deck_zone.cards.is_empty() {
                deck_zone.cards.remove(0);
            }
        }
    }

    // Add to hand (respecting max hand size)
    if let Some(hand_zone) = state.zones.iter_mut()
        .find(|z| z.id.0 == hand_zone_id_string)
    {
        for card in cards_to_draw {
            if hand_zone.cards.len() < rules.players.max_hand_size {
                hand_zone.cards.push(card);
            }
        }
    }
}
