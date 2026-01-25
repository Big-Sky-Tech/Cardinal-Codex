// Storm Elemental: A creature with flying that deals damage on ETB
//
// Demonstrates using stats in scripted card effects

fn execute_ability() {
    // Deal 2 damage to opponent when this enters the battlefield
    deal_damage(1, 2)
}
