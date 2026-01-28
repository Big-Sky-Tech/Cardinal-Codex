// Power Surge
// Pump and add counters

fn execute_ability() {
    [
        pump_creature(source_card, 2, 2),
        add_counter(source_card, "+1/+1", 2)
    ]
}
