---
title: "Encounter prompt + game-over/victory/controls screens"
status: ready-for-agent
---

## What to build

Implement the encounter prompt card and all full-screen overlay screens.

**Encounter prompt:**
- Centered card on dimmed dungeon background (`rgba(10,25,15,.85)`).
- Creature name (bold, red `#ff7070`) with difficulty badge (colored tag per difficulty).
- Brief stats (HP, ATK, DEF) in an enemy-card mini layout.
- Two buttons: "Engage" (`[Enter]`, red `#4a1a1a` bg) and "Back away" (`[Esc]`, green `#2a4a38` bg).
- Back away: player steps back one tile from the creature, creature remains at its position, dungeon un-dimms.

**Game Over screen:**
- Full-screen overlay on player death. Background dims to red.
- Large "GAME OVER" text (Pixel Game font).
- Stats: level reached, rooms explored, enemies slain, time survived.
- "Play Again" button (`[Enter]`) — restart from beginning.
- "Quit" button (`[Q]`) — return to title screen.

**Victory screen:**
- Full-screen overlay when boss is slain.
- Large "YOU WIN!" text.
- Stats: final level, rooms explored, enemies slain, items collected.
- "Play Again" button (`[Enter]`) — restart.
- "Quit" button (`[Q]`) — return to title screen.

**Controls overlay:**
- Full-screen overlay. Toggle with `[H]` or `[?]` key, or clickable help button in HUD.
- Dark background with two-column grid of key bindings: Movement (WASD/Arrows), Combat (arrows/Enter), Inventory (`[I]`), Map (`[M]`), Log (`[L]`), General (Esc/Q), Loot (Enter/Space/Esc).
- Close with any key, tap, or × button.

All screens are React components reading GameState.

## Prototype references

- `demos/dungeon-crawler/prototypes/HUD/hud-battle-view.html` — encounter prompt (`#encounter`) with dimmed background, creature card mini-layout, Engage/Back away buttons. Styling for the encounter card reuses the enemy card CSS patterns.
- `demos/dungeon-crawler/prototypes/HUD/hud-dungeon-view.html` — controls overlay (`#controls`) with the two-column keyboard grid layout. Also the controls close button and help toggle pattern.
- Game Over and Victory screens exist only as specs in the PRD and DESIGN.md — no prototype visual. Use the title screen's full-screen overlay pattern as a template.

## Acceptance criteria

- [ ] Creature touch shows encounter card with dimmed background
- [ ] Engage fades into combat scene, Back away steps player back
- [ ] Game Over overlay shows on player death with stats
- [ ] Victory overlay shows on boss kill with stats
- [ ] Controls overlay opens/closes with `[H]`/`[?]`
- [ ] All overlays work with keyboard, mouse, and touch
- [ ] Restart returns to title screen

## Blocked by

06-combat-core
