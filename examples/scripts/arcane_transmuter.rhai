// Arcane Transmuter
// A complex card demonstrating multiple helper functions
// 
// When played:
// - Mill 2 cards from opponent's deck
// - For each card milled, gain 1 mana
// - Draw a card
// - Grant this creature flying
// - Add a +1/+1 counter

fn execute_ability() {
    [
        // Mill opponent's deck
        mill_cards(1, 2),
        
        // Gain mana as a ritual effect
        gain_resource(controller, "mana", 2),
        
        // Draw a card (cantrip)
        draw_cards(controller, 1),
        
        // Grant flying keyword
        grant_keyword(source_card, "flying"),
        
        // Add a +1/+1 counter
        add_counter(source_card, "+1/+1", 1)
    ]
}
