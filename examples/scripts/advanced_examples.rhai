// Advanced Card Script Examples
// Demonstrates the expanded helper functions in Cardinal

// ==============================================
// Example 1: Life Drain with Type Helper
// ==============================================
fn example_life_drain() {
    // Use the 'drain' type helper for common damage + heal pattern
    drain(1, 3, controller)
}

// ==============================================
// Example 2: Resource Manipulation
// ==============================================
fn example_ritual() {
    // Gain 3 mana when this spell resolves
    gain_resource(controller, "mana", 3)
}

// ==============================================
// Example 3: Card Movement (Mill)
// ==============================================
fn example_mill() {
    // Mill 3 cards from opponent's deck to graveyard
    mill_cards(1, 3)
}

// ==============================================
// Example 4: Stat Modification
// ==============================================
fn example_pump_spell() {
    // Target creature gets +2/+2 until end of turn
    pump_creature(source_card, 2, 2)
}

// ==============================================
// Example 5: Keyword Granting
// ==============================================
fn example_grant_flying() {
    // Grant flying to target creature
    grant_keyword(source_card, "flying")
}

// ==============================================
// Example 6: Token Creation
// ==============================================
fn example_create_soldier() {
    // Create a 1/1 soldier token on the field
    create_token(controller, "soldier_token", "field")
}

// ==============================================
// Example 7: Counter Manipulation
// ==============================================
fn example_add_plus_one_counters() {
    // Add two +1/+1 counters to target creature
    add_counter(source_card, "+1/+1", 2)
}

// ==============================================
// Example 8: Multiple Effects (Cantrip)
// ==============================================
fn example_cantrip_bolt() {
    // Deal 2 damage, then draw a card
    cantrip(controller, bolt(1, 2))
}

// ==============================================
// Example 9: Complex Multi-Step Effect
// ==============================================
fn example_complex_card() {
    // This card does multiple things:
    // 1. Deal 2 damage to opponent
    // 2. Gain 2 life
    // 3. Draw a card
    // 4. Grant controller 1 mana
    [
        deal_damage(1, 2),
        gain_life(controller, 2),
        draw_cards(controller, 1),
        gain_resource(controller, "mana", 1)
    ]
}

// ==============================================
// Example 10: Conditional Logic (Future)
// ==============================================
// Note: Game state queries not yet implemented
// This is a preview of future functionality
//
// fn example_conditional() {
//     if turn_number > 5 {
//         bolt(1, 5)  // Late game: deal 5 damage
//     } else {
//         bolt(1, 2)  // Early game: deal 2 damage
//     }
// }

// ==============================================
// Example 11: Stat Setting (Not Delta)
// ==============================================
fn example_set_stats() {
    // Set creature to exactly 3/3 (ignores previous stats)
    set_stats(source_card, 3, 3)
}

// ==============================================
// Example 12: Generic Stat Modification
// ==============================================
fn example_modify_custom_stat() {
    // Modify a custom stat (e.g., "range" or "element")
    modify_stat(source_card, "range", 1)  // Increase range by 1
}

// ==============================================
// Example 13: Discard Cards
// ==============================================
fn example_discard() {
    // Opponent discards 2 cards
    discard_cards(1, 2)
}

// ==============================================
// Example 14: Shuffle Deck
// ==============================================
fn example_shuffle() {
    // Shuffle controller's deck
    shuffle_zone(controller, "deck")
}

// ==============================================
// Example 15: Life Loss (Not Damage)
// ==============================================
fn example_life_loss() {
    // Opponent loses 3 life (doesn't trigger damage effects)
    lose_life(1, 3)
}

// ==============================================
// Example 16: Set Life Total
// ==============================================
fn example_set_life() {
    // Set opponent's life to exactly 10
    set_life(1, 10)
}

// ==============================================
// Example 17: Resource Spending
// ==============================================
fn example_spend_resources() {
    // Spend 2 action points to activate this ability
    spend_resource(controller, "action_points", 2)
}

// ==============================================
// Example 18: Remove Keyword
// ==============================================
fn example_remove_flying() {
    // Remove flying from target creature
    remove_keyword(source_card, "flying")
}

// ==============================================
// Example 19: Remove Counters
// ==============================================
fn example_remove_counters() {
    // Remove one +1/+1 counter
    remove_counter(source_card, "+1/+1", 1)
}

// ==============================================
// Example 20: Set Custom Stat
// ==============================================
fn example_set_custom_stat() {
    // Set creature's element to "fire"
    set_stat(source_card, "element", "fire")
}
