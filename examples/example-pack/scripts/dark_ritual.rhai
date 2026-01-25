// Dark Ritual: Multi-effect card
//
// This card demonstrates multiple effects in a single ability.
// It deals damage and then gains life, showing how to return an array of effects.

fn execute_ability() {
    // Return an array of effects to execute
    [
        deal_damage(1, 2),    // Deal 2 damage to opponent
        gain_life(0, 3)       // Gain 3 life for controller
    ]
}
