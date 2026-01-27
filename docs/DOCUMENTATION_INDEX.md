# Cardinal Documentation Index

A complete guide to all documentation for the Cardinal game engine project.

## Main Documentation Files

### 📘 README.md
**Purpose**: Project overview and quick navigation  
**Audience**: Everyone  
**Length**: ~150 lines  
**Read time**: 5 minutes  

**Contains**:
- Project features and benefits
- Quick start (CLI, tests, code)
- Core 4 principles
- Key concepts summary
- Links to detailed docs

**Start here if**: You just discovered Cardinal

---

### 📗 README_DETAILED.md
**Purpose**: Complete beginner's guide to Cardinal  
**Audience**: Everyone (technical and non-technical)  
**Length**: ~350 lines  
**Read time**: 30-45 minutes  

**Contains**:
- What Cardinal is (with analogies)
- Why it's designed this way
- Basic game loop explanation
- Game concepts (zones, turns, phases, cards)
- Complete card play example (8 steps)
- Project structure
- Configuration guide
- Learning path
- FAQ and common questions

**Start here if**: You want to understand how Cardinal works

---

### 📕 ARCHITECTURE.md
**Purpose**: Deep technical dive into design  
**Audience**: Developers, architects, contributors  
**Length**: ~700 lines  
**Read time**: 60-90 minutes  

**Contains**:
- Design philosophy and why it matters
- Four immutable principles (detailed)
- How the game loop works (detailed)
- GameState structure and all fields
- Cards: definition vs execution
- Turns: complete flow diagram
- Zones: purpose and usage
- Actions: what players can do
- Events: what happened
- Commands: validation layer
- Trigger system: detailed explanation
- How CLI works internally
- Key data structures table
- Testing strategy
- Summary of concepts

**Start here if**: You want deep understanding or plan to contribute

---

### 📚 crates/cardinal/README.md
**Purpose**: Cardinal library API documentation  
**Audience**: API users, integrators  
**Length**: ~500 lines  
**Read time**: 40-60 minutes  

**Contains**:
- What Cardinal does (6 responsibilities)
- Using Cardinal: basic loop
- Four core principles (developer perspective)
- Game structure (turns, phases, steps, priority)
- How cards work (with TOML example)
- Zones: reference table
- Actions: complete reference
- Events: complete reference
- Triggers: types and examples
- Commands: intermediate effects
- Integration example (full walkthrough)
- File organization
- Key concepts summary table
- How to use in your project
- Testing information
- Common questions

**Start here if**: You want to integrate Cardinal into your code

---

### 📖 crates/cardinal-cli/README.md
**Purpose**: Interactive game tutorial and guide  
**Audience**: Players, CLI developers  
**Length**: ~400 lines  
**Read time**: 30-45 minutes  

**Contains**:
- What the CLI is (and what it isn't)
- Running the game
- Understanding the display (header, hand, field, stack, log)
- Game actions (7 menu options with examples)
- Example game session
- How the CLI works internally
- Display rendering explanation
- Colored output guide
- Test decks information
- Using Cardinal from the CLI
- Extending the CLI (new features)
- Debugging tips
- Summary

**Start here if**: You want to play or modify the CLI

---

### 📋 crates/cardinal/explanation.md
**Purpose**: Design patterns and architecture overview  
**Audience**: Developers, code explorers  
**Length**: ~450 lines  
**Read time**: 30-45 minutes  

**Contains**:
- Core design rules
- Why each rule exists
- Recommended crate layout
- Minimal API surface
- Module organization
- Design patterns
- Key struct examples

**Start here if**: You want to understand code structure and patterns

---

### 📑 crates/cardinal/layout.md
**Purpose**: File organization reference  
**Audience**: Code navigators, contributors  
**Length**: ~50 lines  
**Read time**: 5 minutes  

**Contains**:
- Complete directory structure
- Module organization
- File purposes

**Start here if**: You're navigating the codebase

---

### 📄 DOCUMENTATION.md
**Purpose**: Index and summary of all documentation  
**Audience**: Everyone  
**Length**: ~300 lines  
**Read time**: 10-15 minutes  

**Contains**:
- Summary of all documents
- Document statistics
- Coverage overview
- For different audiences
- Quick navigation guide
- What each document teaches
- Recommended learning paths
- Documentation quality notes

**Start here if**: You're not sure which document to read

---

## Suggested Reading Paths

### Path 1: I Just Want to Play (20 min)
1. README.md (5 min)
2. crates/cardinal-cli/README.md - Display & Actions sections (15 min)
3. Run the game: `cargo run --bin cardinal-cli`

### Path 2: I Want to Understand the System (90 min)
1. README.md (5 min)
2. README_DETAILED.md (40 min)
3. crates/cardinal-cli/README.md (20 min)
4. Play a game (15 min)
5. Edit rules.toml and test changes (10 min)

### Path 3: I Want to Integrate Cardinal (120 min)
1. README.md (5 min)
2. README_DETAILED.md - Game Concepts section (15 min)
3. crates/cardinal/README.md (45 min)
4. ARCHITECTURE.md - Game Loop section (20 min)
5. Try integrating in a test project (35 min)

### Path 4: I Want to Understand the Code (180 min)
1. README.md (5 min)
2. README_DETAILED.md (35 min)
3. ARCHITECTURE.md (60 min)
4. crates/cardinal/explanation.md (25 min)
5. crates/cardinal/layout.md (5 min)
6. Browse crates/cardinal/src/ (50 min)

### Path 5: I Want to Contribute (240 min)
1. Complete Path 4 (180 min)
2. crates/cardinal/README.md - File Organization section (15 min)
3. Read copilot-instructions.md (design principles) (20 min)
4. Review existing code patterns (25 min)

---

## Documentation by Topic

### Understanding Cardinal
- Start: README_DETAILED.md
- Deep dive: ARCHITECTURE.md
- Reference: crates/cardinal/README.md

### Game Mechanics & Rules
- Start: README_DETAILED.md - Game Concepts section
- Reference: crates/cardinal/README.md - Zones, Actions, Events sections
- Deep dive: ARCHITECTURE.md - Turns, Phases, Priority section

### Card System
- Start: README_DETAILED.md - How Cards Work section
- Reference: crates/cardinal/README.md - How Cards Work section
- Deep dive: ARCHITECTURE.md - Cards section

### Triggers & Abilities
- Start: README_DETAILED.md - Card Abilities section
- Reference: crates/cardinal/README.md - Triggers section
- Deep dive: ARCHITECTURE.md - Trigger System section

### Architecture & Design
- Main: ARCHITECTURE.md
- Code: crates/cardinal/explanation.md
- Layout: crates/cardinal/layout.md

### Playing the Game
- Main: crates/cardinal-cli/README.md
- Quick start: README.md - Getting Started section
- Example: crates/cardinal-cli/README.md - Example Game Session section

### Integration & Usage
- Main: crates/cardinal/README.md - Using Cardinal section
- Examples: crates/cardinal/README.md - Integration section
- API: crates/cardinal/README.md throughout

### Configuration & Customization
- Start: README_DETAILED.md - Configuration section
- Details: crates/cardinal-cli/README.md - Extending the CLI section
- Rules: rules.toml (the file itself)

---

## Quick Reference

### For Different Questions

**"What is Cardinal?"**
→ README.md + README_DETAILED.md

**"How do I play?"**
→ crates/cardinal-cli/README.md

**"How do I use it in my project?"**
→ crates/cardinal/README.md + ARCHITECTURE.md

**"How do I modify the rules?"**
→ README_DETAILED.md (Configuration) + rules.toml

**"How does it work internally?"**
→ ARCHITECTURE.md + crates/cardinal/explanation.md

**"What goes where in the code?"**
→ crates/cardinal/layout.md + crates/cardinal/explanation.md

**"Why is it designed this way?"**
→ ARCHITECTURE.md (Principles section) + README_DETAILED.md

**"How do I debug?"**
→ crates/cardinal-cli/README.md (Debugging section) + ARCHITECTURE.md

---

## Documentation Statistics

| File | Type | Lines | Time | Topics |
|------|------|-------|------|--------|
| README.md | Overview | 150 | 5 min | 8 |
| README_DETAILED.md | Guide | 350+ | 40 min | 20 |
| ARCHITECTURE.md | Technical | 700+ | 75 min | 30 |
| crates/cardinal/README.md | API | 500+ | 50 min | 25 |
| crates/cardinal-cli/README.md | Tutorial | 400+ | 40 min | 20 |
| crates/cardinal/explanation.md | Design | 450+ | 35 min | 15 |
| crates/cardinal/layout.md | Reference | 50 | 5 min | 1 |
| **TOTAL** | | **2600+** | **250 min** | **120+** |

---

## Formats & Features

All documentation includes:

✅ **Text explanations** — Clear prose explaining concepts  
✅ **Diagrams** — ASCII flowcharts and structure visualizations  
✅ **Code examples** — Real Rust/TOML code samples  
✅ **Tables** — Reference tables for quick lookup  
✅ **Walkthroughs** — Step-by-step examples  
✅ **FAQs** — Common questions answered  
✅ **Links** — Cross-references between docs  
✅ **Progressive complexity** — Basic to advanced in order  

---

## Getting Started (Quick Guide)

1. **Want to play?**  
   `cargo run --bin cardinal-cli`

2. **Want to understand?**  
   Read README_DETAILED.md

3. **Want to integrate?**  
   Read crates/cardinal/README.md

4. **Want to contribute?**  
   Read ARCHITECTURE.md

5. **Lost?**  
   Read this file (DOCUMENTATION.md)

---

## Summary

Cardinal has **7 comprehensive documentation files** (2600+ lines, 50+ code examples) covering:

- **What it is** ← Start here
- **How it works** ← Understand this
- **How to use it** ← Apply it
- **How to modify it** ← Extend it
- **How it's designed** ← Know why
- **How to navigate the code** ← Find things
- **How to learn it** ← This guide

Find what you need using the Quick Reference or suggested paths above.

**Choose your starting point and begin!**

