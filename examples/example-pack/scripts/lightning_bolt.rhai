// Lightning Bolt: Deal 3 damage to target player
//
// This is an example of a simple scripted card effect.
// The execute_ability function is called when the card's ability triggers.

fn execute_ability() {
    // Deal 3 damage to player 1 (opponent)
    deal_damage(1, 3)
}
