---
title: "Combat system (core)"
status: ready-for-agent
---

## What to build

Implement the turn-based 1v1 combat system with Attack action, enemy AI, and post-victory recovery.

**Combat flow:**
1. Encounter triggered → combat scene fades in (solid dark background `#0a0a0a` with radial gradient).
2. Player card remains top-left. Enemy card appears bottom-right.
3. Action menu (2×2 grid) centered: Attack, Cast Spell, Use Item, Flee.
4. Player turn: select action. Attack resolves immediately.
5. Enemy turn after player action (1s delay for readability).
6. Repeat until victory (creature HP ≤ 0) or defeat (player HP ≤ 0) or flee.

**Damage formulas (from config):**
- Player attack: `max(GAME_CONFIG.combat.minDamage, playerAttack - creatureDefense + rand(0, GAME_CONFIG.combat.playerDamageRollMax))`
- Creature attack: `max(GAME_CONFIG.combat.minDamage, creatureAttack - playerDefense + rand(0, GAME_CONFIG.combat.creatureDamageRollMax) - GAME_CONFIG.combat.creatureAttackPenalty)`

**Victory:**
- Brief flash → loot popup (placeholder — wired in slice 8).
- Post-victory recovery: restore `GAME_CONFIG.recovery.hpPercent` (15%) of max HP and `GAME_CONFIG.recovery.mpPercent` (15%) of max MP.
- XP gain: creature's `xpReward` value. Check if player levels up using `GAME_CONFIG.xpThresholds` array.
- Creature removed from room.

**Defeat:**
- Screen dims to red → Game Over overlay (placeholder — wired in slice 7).

**Enemy card:**
- Bottom-right, fixed during combat. Shows creature name (with difficulty badge colored per difficulty), HP bar (red gradient), ATK/DEF stats.
- Card style: dark red tones (`#3a2020` bg, `#6a3a3a` border) matching the prototype.

**Action menu:**
- 2×2 grid centered. Each cell: icon + label, 150×80px, green card style (`#2d4d38` bg, `#4a7a58` border) with pixel-art 3D shadow.
- Attack: `fa-crosshairs`, Cast Spell: `fa-wand-sparkles`, Use Item: `fa-flask`, Flee: `fa-person-running`.
- Keyboard: Arrow keys navigate grid, Enter confirms. Mouse/touch: click/tap.
- Hover: lift effect (translateY -2px, stronger shadow). Active: press effect (translateY +2px, shallow shadow).
- Cast Spell and Use Item show placeholder alerts for now (wired in slice 9).

## Prototype references

- `demos/dungeon-crawler/prototypes/HUD/hud-battle-view.html` — THE prototype for this slice. Contains:
  - Combat background (`combat-bg`, `combat-floor`, `combat-vs`)
  - Enemy card (`#enemy-card`) with exact CSS for positioning, colors, HP bar, stats
  - Action menu (`#action-menu`) with 2×2 grid, button styles, hover/active effects, FontAwesome icons
  - Player card (`#player-card`) — same as used in dungeon view
  - The entire HTML/CSS can be ported directly to React components
- `demos/dungeon-crawler/docs/PRD.md` — Game Configuration section for the config values.

## Acceptance criteria

- [ ] Combat scene with dark background fades in on encounter
- [ ] Player card stays visible, enemy card appears bottom-right
- [ ] Attack action deals damage using the config formula
- [ ] Enemy AI attacks back on its turn
- [ ] Victory applies 15% HP/MP recovery and grants XP
- [ ] Defeat shows game over placeholder
- [ ] Victory removes creature from room
- [ ] All combat event states update correctly in GameState

## Blocked by

06-creature-spawning
