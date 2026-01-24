# Cardinal CLI: Interactive Game Tutorial

The **cardinal-cli** is a terminal-based game interface that demonstrates Cardinal's capabilities. It's a working example of how to build a client that uses the Cardinal game engine.

## What is the CLI?

Think of it like this:

```
┌──────────────────────────────┐
│     You (Terminal User)       │
│   "I'll play card #1"         │
└──────────────┬───────────────┘
               │
               ▼
        ┌─────────────┐
        │ Cardinal-CLI│  Takes your input
        │  (Terminal) │  Renders game to screen
        └──────┬──────┘  Reads Cardinal's events
               │
               ▼
        ┌─────────────┐
        │   Cardinal  │  Validates your action
        │   Engine    │  Updates game state
        └──────┬──────┘  Emits events
               │
               ▼
        ┌─────────────┐
        │   Events    │  "Card was played"
        │  & State    │  "Creature entered field"
        └─────────────┘
```

**Cardinal-cli does NOT make decisions.** It's just a window into the Cardinal engine. All game logic happens in Cardinal; the CLI just shows it to you.

---

## Running the CLI

```bash
cargo run --bin cardinal-cli
```

You'll see something like:

```
Welcome to Cardinal - A Rules Engine TCG!

✓ Rules loaded: My Cool TCG
✓ Game state created
✓ Test decks populated
✓ Game initialized

═══════════════════════════════════════════════════════════
Game starting! You are Player 0
═══════════════════════════════════════════════════════════

Turn 1 | Phase: start | Step: untap | Priority: PlayerId(1)
Your Life: 20 ♥  |  Opponent Life: 20 ♥

Your Hand
  [1] Goblin Scout (creature) [1R]
  [2] Knight of Valor (creature) [2W]
  [3] Inspiration (spell) [1U]
  ...

Available Actions
  [1] Play card
  [2] View hand (detailed)
  [3] View your field
  [4] View opponent's field
  [5] View game log
  [6] Pass priority
  [7] Concede

  > _
```

Type a number to choose an action.

---

## Understanding the Display

### The Header

```
Turn 1 | Phase: start | Step: untap | Priority: PlayerId(1)
Your Life: 20 ♥  |  Opponent Life: 20 ♥
```

- **Turn 1** — First turn of the game
- **Phase: start** — We're in the "start" phase (untapping, upkeeping, drawing)
- **Step: untap** — Currently in the "untap" step (permanents untap)
- **Priority: PlayerId(1)** — Player 1 (the opponent) has priority (can act)
- **Life totals** — You have 20 health; opponent has 20 health

### Your Hand

```
Your Hand
  [1] Goblin Scout (creature) [1R]
  [2] Knight of Valor (creature) [2W]
  [3] Inspiration (spell) [1U]
  [4] Card #0
  [5] Fireball (spell) [2R]
```

- **[1]** — Card number (press this to play it)
- **Goblin Scout** — Card name
- **(creature)** — Card type
- **[1R]** — Mana cost (1 generic + 1 red)

### Your Field

```
Your Field
  [1] Goblin Scout (1/1 creature)
  [2] Knight of Valor (1/1 creature)
```

Cards in play on your side of the battlefield.

### Opponent Field

```
Opponent Field
  (cards are hidden)
```

You can't see opponent cards (hidden from you).

### Stack

```
Stack
  (empty)

or

Stack
  [1] Deal 1 damage to opponent
  [2] Draw 1 card
```

The **stack** is where spells and abilities wait to resolve. Items resolve in order (last in, first out).

### Game Log

```
Turn 1, start/untap: Game started
Turn 1, start/draw: You drew 1 card
Turn 1, main: You played: Goblin Scout
Turn 1, main: Goblin Scout dealt 1 damage to opponent
```

A record of what happened. Useful for replays.

---

## Game Actions

### [1] Play Card

Plays a card from your hand.

```
> 1
Which card to play? (1-5): 1
→ You played: Goblin Scout
→ Card entered play
→ Ability triggered!
→ Opponent took 1 damage
```

The game validates:
- Do you own the card?
- Is it in your hand?
- Is it your turn?
- Is the game in a phase where playing is allowed?
- Do you have enough mana?

If any check fails:
```
> 1
Which card to play? (1-5): 3
→ Cannot play card: NotYourTurn
```

### [2] View Hand (Detailed)

Shows each card in ASCII art with full details.

```
> 2

┌─────────────────────────┐
│ Goblin Scout [1R]       │
├─────────────────────────┤
│ creature                │
│                         │
│ A small but feisty      │
│ goblin.                 │
│                         │
│ etb (damage 1)          │
└─────────────────────────┘

┌─────────────────────────┐
│ Inspiration [1U]        │
├─────────────────────────┤
│ spell                   │
│                         │
│ Draw a card.            │
│                         │
│ on_play (draw)          │
└─────────────────────────┘

(more cards...)
```

Useful for understanding what you have.

### [3] View Your Field

Shows all your creatures and enchantments in play.

```
> 3

Your Field
  [1] Goblin Scout (1/1 creature)
  [2] Knight of Valor (1/1 creature)
  [3] Combat Trick (enchantment)

(empty)
```

### [4] View Opponent's Field

Shows opponent's creatures (hidden name, just a placeholder).

```
> 4

Opponent Field
  (cards are hidden)
```

You can't see what they have (information hiding for fairness).

### [5] View Game Log

Shows recent actions and events.

```
> 5

Recent Game Log (last 20 entries)
=====================================
Turn 1, start/untap: Game started
Turn 1, start/upkeep: (no upkeep events)
Turn 1, start/draw: You drew 1 card
Turn 1, main/main: You played: Goblin Scout
Turn 1, main/main: Goblin Scout entered the field
Turn 1, main/main: Ability triggered: deal 1 damage
Turn 1, main/main: Opponent took 1 damage
```

Useful for understanding the game state and replaying what happened.

### [6] Pass Priority

Pass your turn to the opponent.

```
> 6
→ You passed priority
→ Priority passed to opponent
```

If the opponent also passes, the phase ends and the next phase begins.

### [7] Concede

Give up.

```
> 7
→ You have conceded. Game over!
→ Thanks for playing!
```

Game ends. You lose.

---

## How the CLI Works Internally

### Main Game Loop

```rust
loop {
    // 1. Render current state
    println!("{}", display.render_game(&engine.state));
    
    // 2. Show available actions
    println!("{}", display.render_menu());
    
    // 3. Get player input
    let choice = read_line();
    
    // 4. Apply action
    match choice {
        1 => handle_play_card(&mut engine, ...),
        2 => handle_view_hand(&display, ...),
        ...
    }
    
    // 5. Process events from Cardinal
    for event in &result.events {
        game_log.add(event);
        println!("→ {}", format_event(event));
    }
    
    // Loop continues
}
```

### Input Flow

```
User types "1"
    ↓
CLI parses to: Action::PlayCard { ... }
    ↓
Passes to Cardinal: engine.apply_action(player, action)
    ↓
Cardinal validates and applies
    ↓
Returns StepResult { events: [...], ... }
    ↓
CLI reads events and prints them
    ↓
Back to step 1 (show state again)
```

### Display Rendering

The CLI uses the `GameDisplay` struct from Cardinal:

```rust
pub struct GameDisplay {
    game_log: Vec<LogEntry>,  // History of what happened
}

impl GameDisplay {
    pub fn render_game(&self, state: &GameState) -> String { ... }
    pub fn render_hand(&self, hand: &[Card]) -> String { ... }
    pub fn render_field(&self, field: &[Card]) -> String { ... }
    pub fn render_menu(&self) -> String { ... }
}
```

It formats the game state into readable text (with colors!).

---

## Using Colored Output

The CLI uses the **colored** crate to add colors to the terminal:

```
Turn 1 | Phase: main | Step: main | Priority: PlayerId(0)
Your Life: 20 ♥  |  Opponent Life: 18 ♥
     ↑ White          ↑ Green (healthy)    ↑ Red (damaged)
```

Life totals show in different colors:
- **Green** — You're healthy
- **Red** — You're hurt
- **Yellow** — You're very hurt

This makes it easy to see the game state at a glance.

---

## Test Decks

When you start the CLI, it automatically creates test decks:

```toml
# deck_0 (Your deck)
- Goblin Scout
- Knight of Valor
- Inspiration
- Fireball
- Bloated Toad

# deck_1 (Opponent's deck)
- Same as above
```

Both players start with identical test decks (5 cards each). You draw into them at the start of the game.

To customize:
- Edit the `rules.toml` file
- Add more cards or change their abilities
- The CLI will use whatever's in `rules.toml`

---

## Example Game Session

### Turn 1 (You)

```
> 1
Which card to play? 1
→ You played: Goblin Scout
→ Goblin Scout entered the field
→ Ability triggered: etb - deal 1 damage
→ Opponent took 1 damage

Current state:
  Your Life: 20
  Opponent Life: 19

> 6
→ You passed priority
→ Priority now belongs to opponent
```

### Turn 1 (Opponent)

```
(Opponent's turn - you wait)
→ Opponent played a card
→ Opponent passed priority

(Back to you)
```

### Turn 2 (You)

```
> 2
(Viewing your hand)

> 1
Which card to play? 2
→ You played: Knight of Valor
→ Knight of Valor entered the field
→ Ability triggered: etb - gain 2 life
→ You gained 2 life

Current state:
  Your Life: 22
  Opponent Life: 19

> 6
→ Passed priority
```

And so on...

---

## Understanding Cardinal from the CLI

The CLI is a **mirror of what's happening in Cardinal**:

| What you see in CLI | What's happening in Cardinal |
|---|---|
| Card plays successfully | `apply_action()` returned Ok(...) |
| "Cannot play card" error | `apply_action()` returned Err(...) |
| Life totals change | State mutation happened; event emitted |
| Card in your field | Card moved from Hand zone to Field zone |
| Card in opponent's field | Hidden (you don't know what it is) |
| Game log entry | Event was emitted from Cardinal |
| Menu options change | `legal_actions()` returned different actions based on phase/priority |

The CLI is 100% deterministic based on Cardinal's output. It doesn't decide anything; it just shows you what Cardinal decided.

---

## Extending the CLI

Want to add features?

### New Menu Option

1. Add to `render_menu()` in [display.rs](../cardinal/src/display.rs)
2. Add a handler function in [main.rs](./src/main.rs)
3. Call the handler from the input match statement

Example: Adding a "draw card" debug command

```rust
// In render_menu()
println!("  [8] Debug: Draw a card");

// In input handler
8 => {
    let result = engine.apply_action(
        player,
        Action::DrawCards { count: 1 }  // if this action exists
    );
    // Process events...
}
```

### New Display Format

Modify [display.rs](../cardinal/src/display.rs):

```rust
impl GameDisplay {
    pub fn render_mana_pools(&self, player: &Player) -> String {
        // Custom formatting for mana pools
    }
}
```

Then call it in `render_game()`.

### Persistent Game State

Add file I/O to save/load games:

```rust
use std::fs;
use serde_json;

// Save
let state_json = serde_json::to_string(&engine.state)?;
fs::write("game.json", state_json)?;

// Load
let state_json = fs::read_to_string("game.json")?;
let state = serde_json::from_str(&state_json)?;
```

---

## Debugging the Game

### Check the Log

Press `[5]` to see the game log. It shows exactly what happened.

### Check the State

The CLI prints the full game state every turn. Look for:
- Your hand (did you draw?)
- Your field (did the card play?)
- Life totals (did abilities fire?)
- Stack (is something waiting to resolve?)

### Use Determinism

If you want to replay a game:

1. Note the seed used (printed at startup, or set your own)
2. Save all the actions taken
3. Run with the same seed + actions = same outcome

This is perfect for debugging.

---

## Summary

The **cardinal-cli** is:

- ✅ A working example of using Cardinal
- ✅ A terminal UI for playing the game
- ✅ A way to test the engine without a GUI
- ✅ Fully data-driven (uses rules.toml)
- ✅ Extensible (add features by modifying display.rs and main.rs)

The key insight: **The CLI is just a view. Cardinal is the controller.**

Everything the CLI does is: "Read Cardinal's state → Show it to user → Get user input → Feed to Cardinal → Repeat"

---

## Next Steps

- **Try playing a game** — Run `cargo run --bin cardinal-cli`
- **Read the code** — Check [src/main.rs](./src/main.rs) to see how it uses Cardinal
- **Modify a card** — Edit `rules.toml` and try a different card ability
- **Add a feature** — Implement a new menu option
- **Build your own UI** — Use the same Cardinal library in your own project

The CLI shows that Cardinal is **embeddable and reusable**. You can use it in a web game, mobile app, or anything else.

