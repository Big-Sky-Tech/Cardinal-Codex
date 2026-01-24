use colored::Colorize;
use crate::{
    state::gamestate::GameState,
    ids::{PlayerId, CardId},
    engine::cards::CardRegistry,
};

/// Game log entry for tracking what happened
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub turn: u32,
    pub phase: String,
    pub step: String,
    pub message: String,
}

/// Complete game display with formatting
pub struct GameDisplay {
    pub game_log: Vec<LogEntry>,
}

impl GameDisplay {
    pub fn new() -> Self {
        Self {
            game_log: Vec::new(),
        }
    }

    pub fn log(&mut self, turn: u32, phase: &str, step: &str, message: String) {
        self.game_log.push(LogEntry {
            turn,
            phase: phase.to_string(),
            step: step.to_string(),
            message,
        });
    }

    /// Render the complete game state
    pub fn render_game(
        &self,
        state: &GameState,
        cards: &CardRegistry,
        viewer: PlayerId,
    ) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&self.render_header(state, viewer));
        output.push('\n');

        // Your field
        output.push_str(&self.render_zone(
            state,
            cards,
            viewer,
            "field",
            "Your Field",
            false,
        ));
        output.push('\n');

        // Opponent field
        let opponent = if viewer.0 == 0 { PlayerId(1) } else { PlayerId(0) };
        output.push_str(&self.render_zone(
            state,
            cards,
            opponent,
            "field",
            "Opponent Field",
            true,
        ));
        output.push('\n');

        // Stack
        output.push_str(&self.render_stack(state));
        output.push('\n');

        // Hand
        output.push_str(&self.render_hand(state, cards, viewer));

        output
    }

    fn render_header(&self, state: &GameState, viewer: PlayerId) -> String {
        let mut output = String::new();

        // Turn info
        let turn_info = format!(
            "Turn {} | Phase: {} | Step: {} | Priority: {:?}",
            state.turn.number, state.turn.phase.0, state.turn.step.0, state.turn.priority_player
        );
        output.push_str(&turn_info.bold().to_string());
        output.push('\n');

        // Life totals
        let your_player = viewer;
        let your_life = state.players.iter()
            .find(|p| p.id == your_player)
            .map(|p| p.life)
            .unwrap_or(0);

        let opponent = if viewer.0 == 0 { PlayerId(1) } else { PlayerId(0) };
        let opponent_life = state.players.iter()
            .find(|p| p.id == opponent)
            .map(|p| p.life)
            .unwrap_or(0);

        let your_life_str = if your_life > 10 {
            format!("{}", your_life).green()
        } else if your_life > 5 {
            format!("{}", your_life).yellow()
        } else {
            format!("{}", your_life).red()
        };

        let opponent_life_str = if opponent_life > 10 {
            format!("{}", opponent_life).green()
        } else if opponent_life > 5 {
            format!("{}", opponent_life).yellow()
        } else {
            format!("{}", opponent_life).red()
        };

        output.push_str(&format!("Your Life: {} ♥  |  Opponent Life: {} ♥", your_life_str, opponent_life_str));
        output.push('\n');

        output
    }

    fn render_zone(
        &self,
        state: &GameState,
        cards: &CardRegistry,
        player: PlayerId,
        zone_name: &str,
        display_name: &str,
        hide_cards: bool,
    ) -> String {
        let zone_id_str = format!("{}@{}", zone_name, player.0);
        let zone = state.zones.iter().find(|z| z.id.0 == zone_id_str);

        let mut output = String::new();
        output.push_str(&format!("{}\n", display_name.bold().cyan()));

        if let Some(zone) = zone {
            if zone.cards.is_empty() {
                output.push_str("  (empty)\n");
            } else {
                for (idx, card_id) in zone.cards.iter().enumerate() {
                    if hide_cards {
                        output.push_str(&format!("  [{}] Mystery Card\n", idx + 1));
                    } else if let Some(card_def) = cards.get(&card_id.0) {
                        let card_str = format!("[{}] {} ({})", idx + 1, card_def.name, card_def.card_type);
                        output.push_str(&format!("  {}\n", card_str.yellow()));
                    } else {
                        output.push_str(&format!("  [{}] Card #{}\n", idx + 1, card_id.0));
                    }
                }
            }
        } else {
            output.push_str("  (zone not found)\n");
        }

        output
    }

    fn render_hand(
        &self,
        state: &GameState,
        cards: &CardRegistry,
        player: PlayerId,
    ) -> String {
        let hand_id_str = format!("hand@{}", player.0);
        let hand_zone = state.zones.iter().find(|z| z.id.0 == hand_id_str);

        let mut output = String::new();
        output.push_str(&format!("{}\n", "Your Hand".bold().cyan()));

        if let Some(hand) = hand_zone {
            if hand.cards.is_empty() {
                output.push_str("  (empty)\n");
            } else {
                for (idx, card_id) in hand.cards.iter().enumerate() {
                    if let Some(card_def) = cards.get(&card_id.0) {
                        let cost_str = card_def.cost.as_deref().unwrap_or("—");
                        let card_str = format!(
                            "[{}] {} ({}) [{}]",
                            idx + 1, card_def.name, card_def.card_type, cost_str
                        );
                        output.push_str(&format!("  {}\n", card_str.yellow()));
                    } else {
                        output.push_str(&format!("  [{}] Card #{}\n", idx + 1, card_id.0));
                    }
                }
            }
        }

        output
    }

    fn render_stack(&self, state: &GameState) -> String {
        let mut output = String::new();
        output.push_str(&format!("{}\n", "Stack".bold().cyan()));

        if state.stack.is_empty() {
            output.push_str("  (empty)\n");
        } else {
            for (idx, item) in state.stack.iter().enumerate() {
                let source_str = item.source
                    .map(|id| format!("Card #{}", id.0))
                    .unwrap_or_else(|| "Unknown".to_string());
                
                let effect_str = match &item.effect {
                    crate::model::command::EffectRef::Builtin(name) => name.to_string(),
                    crate::model::command::EffectRef::Scripted(name) => name.clone(),
                };
                
                output.push_str(&format!(
                    "  [{}] {} (source: {}, controller: {:?})\n",
                    idx + 1, effect_str, source_str, item.controller
                ));
            }
        }

        output
    }

    /// Render game log (last N entries)
    pub fn render_log(&self, limit: usize) -> String {
        let mut output = String::new();
        output.push_str(&format!("{}\n", "Game Log".bold().cyan()));

        let start = if self.game_log.len() > limit {
            self.game_log.len() - limit
        } else {
            0
        };

        for entry in &self.game_log[start..] {
            let header = format!(
                "[Turn {}, {} - {}]",
                entry.turn, entry.phase, entry.step
            ).dimmed();
            output.push_str(&format!("{} {}\n", header, entry.message));
        }

        output
    }

    /// Show the main menu
    pub fn render_menu(&self, is_active_player: bool, is_priority_player: bool) -> String {
        let mut output = String::new();
        output.push_str(&format!("{}\n", "Available Actions".bold().green()));

        if is_active_player && is_priority_player {
            output.push_str("  [1] Play card from hand\n");
            output.push_str("  [2] View hand (detailed)\n");
            output.push_str("  [3] View your field\n");
            output.push_str("  [4] View opponent's field\n");
            output.push_str("  [5] View game log\n");
            output.push_str("  [6] Pass priority\n");
            output.push_str("  [7] Concede\n");
        } else if is_priority_player {
            output.push_str("  [2] View hand (detailed)\n");
            output.push_str("  [3] View your field\n");
            output.push_str("  [4] View opponent's field\n");
            output.push_str("  [5] View game log\n");
            output.push_str("  [6] Pass priority\n");
            output.push_str("  [7] Concede\n");
        } else {
            output.push_str("  [2] View hand (detailed)\n");
            output.push_str("  [3] View your field\n");
            output.push_str("  [4] View opponent's field\n");
            output.push_str("  [5] View game log\n");
            output.push_str("  [7] Concede\n");
            output.push_str("\n  Waiting for opponent's priority...\n");
        }

        output
    }

    /// Render a single card in detail
    pub fn render_card_detail(&self, cards: &CardRegistry, card_id: CardId) -> String {
        let mut output = String::new();

        if let Some(card_def) = cards.get(&card_id.0) {
            output.push_str(&format!("{}", "┌─────────────────────────┐\n".bright_black()));
            output.push_str(&format!(
                "{} {} [{}] {}\n",
                "│".bright_black(),
                card_def.name.bold().yellow(),
                card_def.cost.as_deref().unwrap_or("—"),
                "│".bright_black()
            ));
            output.push_str(&format!("{}", "├─────────────────────────┤\n".bright_black()));
            output.push_str(&format!(
                "{} {} {}\n",
                "│".bright_black(),
                card_def.card_type.cyan(),
                "│".bright_black()
            ));
            output.push_str(&format!("{}", "│                         │\n".bright_black()));

            if let Some(desc) = &card_def.description {
                for line in desc.lines() {
                    output.push_str(&format!(
                        "{} {:<23} {}\n",
                        "│".bright_black(),
                        line,
                        "│".bright_black()
                    ));
                }
                output.push_str(&format!("{}", "│                         │\n".bright_black()));
            }

            for ability in &card_def.abilities {
                let ability_text = format!(
                    "{} ({})",
                    ability.trigger.green(), ability.effect.magenta()
                );
                output.push_str(&format!(
                    "{} {:<23} {}\n",
                    "│".bright_black(),
                    &ability_text.to_string()[..std::cmp::min(23, ability_text.len())],
                    "│".bright_black()
                ));
            }

            output.push_str(&format!("{}", "└─────────────────────────┘\n".bright_black()));
        } else {
            output.push_str(&format!("Card #{} not found\n", card_id.0).red().to_string());
        }

        output
    }
}
