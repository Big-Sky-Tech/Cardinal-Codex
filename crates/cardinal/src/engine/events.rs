use crate::state::gamestate::{GameState, CardInstanceData};
use crate::model::command::Command;
use crate::model::event::Event;
use crate::ids::{CardId, ZoneId};
use std::collections::HashMap;

/// Apply a batch of commands to the `GameState` and return emitted events.
/// Each command mutates the state and produces one or more events.
pub fn commit_commands(state: &mut GameState, commands: &[Command]) -> Vec<Event> {
    let mut events = Vec::new();

    for cmd in commands {
        match cmd {
            Command::MoveCard { card, from, to } => {
                // Remove card from source zone
                if let Some(zone) = state.zones.iter_mut().find(|z| z.id == *from) {
                    zone.cards.retain(|c| c != card);
                }
                // Add card to destination zone
                if let Some(zone) = state.zones.iter_mut().find(|z| z.id == *to) {
                    zone.cards.push(*card);
                }
                events.push(Event::CardMoved { card: *card, from: from.clone(), to: to.clone() });
            }
            Command::ChangeLife { player, delta } => {
                if let Some(p) = state.players.iter_mut().find(|pl| pl.id == *player) {
                    p.life += delta;
                }
                events.push(Event::LifeChanged { player: *player, delta: *delta });
            }
            Command::SetLife { player, amount } => {
                if let Some(p) = state.players.iter_mut().find(|pl| pl.id == *player) {
                    p.life = *amount;
                }
                events.push(Event::LifeSet { player: *player, amount: *amount });
            }
            Command::PushStack { item } => {
                let stack_id = item.id;
                state.stack.push(item.clone());
                events.push(Event::StackPushed { item_id: stack_id });
            }
            Command::RequestChoice { player, choice } => {
                state.pending_choice = Some(choice.clone());
                events.push(Event::ChoiceRequested { choice_id: choice.id, player: *player });
            }
            Command::ShuffleZone { zone: _ } => {
                // NOTE: ShuffleZone is intentionally left unimplemented here.
                // A correct implementation must:
                //   - Use the engine-owned RNG to deterministically reorder the cards
                //     in the target zone within `GameState`, and
                //   - Emit a corresponding `ZoneShuffled` event that accurately
                //     reflects the state change.
                //
                // Emitting `ZoneShuffled` without changing `GameState` would violate
                // the engine's design principles (state is authoritative; events
                // must describe real state changes). Until proper shuffling is wired
                // up, this command must not be used in live rules/effects.
                todo!("Command::ShuffleZone is not yet implemented: it must update GameState and use the engine RNG to shuffle the zone");
            }
            Command::ModifyStats { card, power, toughness } => {
                let instance = state.card_instances.entry(*card).or_insert_with(|| CardInstanceData {
                    stats: HashMap::new(),
                    stat_modifiers: HashMap::new(),
                    keywords: Vec::new(),
                    counters: HashMap::new(),
                });
                *instance.stat_modifiers.entry("power".to_string()).or_insert(0) += *power;
                *instance.stat_modifiers.entry("toughness".to_string()).or_insert(0) += *toughness;
                events.push(Event::StatsModified { card: *card, power: *power, toughness: *toughness });
            }
            Command::SetStats { card, power, toughness } => {
                let instance = state.card_instances.entry(*card).or_insert_with(|| CardInstanceData {
                    stats: HashMap::new(),
                    stat_modifiers: HashMap::new(),
                    keywords: Vec::new(),
                    counters: HashMap::new(),
                });
                instance.stats.insert("power".to_string(), power.to_string());
                instance.stats.insert("toughness".to_string(), toughness.to_string());
                events.push(Event::StatsSet { card: *card, power: *power, toughness: *toughness });
            }
            Command::ModifyStat { card, stat_name, delta } => {
                let instance = state.card_instances.entry(*card).or_insert_with(|| CardInstanceData {
                    stats: HashMap::new(),
                    stat_modifiers: HashMap::new(),
                    keywords: Vec::new(),
                    counters: HashMap::new(),
                });
                *instance.stat_modifiers.entry(stat_name.clone()).or_insert(0) += *delta;
                events.push(Event::StatModified { card: *card, stat_name: stat_name.clone(), delta: *delta });
            }
            Command::SetStat { card, stat_name, value } => {
                let instance = state.card_instances.entry(*card).or_insert_with(|| CardInstanceData {
                    stats: HashMap::new(),
                    stat_modifiers: HashMap::new(),
                    keywords: Vec::new(),
                    counters: HashMap::new(),
                });
                instance.stats.insert(stat_name.clone(), value.clone());
                events.push(Event::StatSet { card: *card, stat_name: stat_name.clone(), value: value.clone() });
            }
            Command::GrantKeyword { card, keyword } => {
                let instance = state.card_instances.entry(*card).or_insert_with(|| CardInstanceData {
                    stats: HashMap::new(),
                    stat_modifiers: HashMap::new(),
                    keywords: Vec::new(),
                    counters: HashMap::new(),
                });
                if !instance.keywords.contains(keyword) {
                    instance.keywords.push(keyword.clone());
                }
                events.push(Event::KeywordGranted { card: *card, keyword: keyword.clone() });
            }
            Command::RemoveKeyword { card, keyword } => {
                if let Some(instance) = state.card_instances.get_mut(card) {
                    instance.keywords.retain(|k| k != keyword);
                }
                events.push(Event::KeywordRemoved { card: *card, keyword: keyword.clone() });
            }
            Command::GainResource { player, resource, amount } => {
                if let Some(p) = state.players.iter_mut().find(|pl| pl.id == *player) {
                    *p.resources.entry(resource.clone()).or_insert(0) += *amount;
                }
                events.push(Event::ResourceGained { player: *player, resource: resource.clone(), amount: *amount });
            }
            Command::SpendResource { player, resource, amount } => {
                if let Some(p) = state.players.iter_mut().find(|pl| pl.id == *player) {
                    *p.resources.entry(resource.clone()).or_insert(0) -= *amount;
                }
                events.push(Event::ResourceSpent { player: *player, resource: resource.clone(), amount: *amount });
            }
            Command::SetResource { player, resource, amount } => {
                if let Some(p) = state.players.iter_mut().find(|pl| pl.id == *player) {
                    p.resources.insert(resource.clone(), *amount);
                }
                events.push(Event::ResourceSet { player: *player, resource: resource.clone(), amount: *amount });
            }
            Command::CreateToken { player, token_type, zone } => {
                // Generate a new card ID for the token
                // Use a simple approach: find max CardId and add 1
                let max_id = state.zones.iter()
                    .flat_map(|z| &z.cards)
                    .map(|c| c.0)
                    .max()
                    .unwrap_or(0);
                let token_id = CardId(max_id + 1);
                
                // Add token to the specified zone
                if let Some(z) = state.zones.iter_mut().find(|z| z.id == *zone) {
                    z.cards.push(token_id);
                }
                
                // Initialize token instance data
                let instance = CardInstanceData {
                    stats: HashMap::new(),
                    stat_modifiers: HashMap::new(),
                    keywords: Vec::new(),
                    counters: HashMap::new(),
                };
                state.card_instances.insert(token_id, instance);
                
                events.push(Event::TokenCreated { 
                    player: *player, 
                    token_type: token_type.clone(), 
                    card: token_id,
                    zone: zone.clone(),
                });
            }
            Command::AddCounter { card, counter_type, amount } => {
                let instance = state.card_instances.entry(*card).or_insert_with(|| CardInstanceData {
                    stats: HashMap::new(),
                    stat_modifiers: HashMap::new(),
                    keywords: Vec::new(),
                    counters: HashMap::new(),
                });
                *instance.counters.entry(counter_type.clone()).or_insert(0) += *amount;
                events.push(Event::CounterAdded { card: *card, counter_type: counter_type.clone(), amount: *amount });
            }
            Command::RemoveCounter { card, counter_type, amount } => {
                if let Some(instance) = state.card_instances.get_mut(card) {
                    let current = instance.counters.entry(counter_type.clone()).or_insert(0);
                    *current = (*current - *amount).max(0);
                }
                events.push(Event::CounterRemoved { card: *card, counter_type: counter_type.clone(), amount: *amount });
            }
        }
    }

    events
}

// Event handling logic
