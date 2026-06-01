---
title: "Title screen + player card"
status: completed
---

## What to build

Implement the title screen and the persistent player card HUD element.

**Title screen:**
- Full-screen overlay on initial load (before game starts).
- Game title (large, centered) using Pixel Game font in green `#60e060` with text-shadow glow.
- Subtitle/tagline.
- "Press Enter to Start" or tap-to-play prompt (pulsing opacity animation).
- Background: static dungeon-themed pattern or animated graphic using prototype colors (dark green `#1a3a2a` → `#0a1a10` gradient).
- On start: cross-fade to dungeon, dungeon generation happens during transition.

**Player card:**
- Top-left, always visible. Follows the prototype exactly.
- Layout: avatar icon + name/level header, HP bar (red gradient, numeric value right-aligned), MP bar (blue gradient, numeric value right-aligned), stats row (ATK / DEF).
- HP bar colors: linear gradient `#d04040` → `#ff6060`, empty track `#1a3a2a`.
- MP bar colors: linear gradient `#4060d0` → `#6080ff`.
- XP bar below stats (smaller, green `#60e060`).
- Card style: `#2d4d38` background, `#4a7a58` border, pixel-art 3D shadow (`4px 4px 0 rgba(0,0,0,.3)`).
- Click/tap on player card opens the inventory dialog (handled in slice 10 — wire up the event but the handler can be a placeholder).
- Player stats read from GameState.player, update reactively via event subscription.

**Values from config:**
- Base HP: `GAME_CONFIG.player.baseHp` (100)
- Base MP: `GAME_CONFIG.player.baseMp` (30)
- Base ATK: `GAME_CONFIG.player.baseAttack` (10)
- Base DEF: `GAME_CONFIG.player.baseDefense` (5)
- Starting level: `GAME_CONFIG.player.startingLevel` (1)

## Prototype references

- `demos/dungeon-crawler/prototypes/HUD/hud-dungeon-view.html` — the title screen (`#title`) with its pulsing prompt and dark green gradient background. The player card (`#player-card`) with all CSS — exact bar styling, avatar circle, stats row, shadow effects. Copy the HTML/CSS directly, translate to React.
- `demos/dungeon-crawler/prototypes/HUD/hud-battle-view.html` — same player card, shown in the combat context (top-left, same position). Confirms it's persistent across all game phases.

## Acceptance criteria

- [ ] Title screen shows on load with game title and "Press Enter" prompt
- [ ] Pressing Enter/tap transitions to dungeon view
- [ ] Player card visible at top-left after game starts
- [ ] HP/MP bars reflect GameState values and animate on change
- [ ] ATK/DEF/Lv/XP displayed correctly
- [ ] Clicking player card emits event for inventory dialog (handler can be placeholder)

## Blocked by

01-project-scaffold
