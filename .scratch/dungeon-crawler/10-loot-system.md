---
title: "Loot system"
status: ready-for-agent
---

## What to build

Implement loot drops after creature defeat, the loot popup, and item generation.

**Drop counts by difficulty (from config):**
- Normal: `rand(GAME_CONFIG.lootDrops.normal.min, GAME_CONFIG.lootDrops.normal.max)` = 0–1
- Champion: `rand(GAME_CONFIG.lootDrops.champion.min, GAME_CONFIG.lootDrops.champion.max)` = 1–3
- Elite: `rand(GAME_CONFIG.lootDrops.elite.min, GAME_CONFIG.lootDrops.elite.max)` = 3–5
- Boss: `rand(GAME_CONFIG.lootDrops.boss.min, GAME_CONFIG.lootDrops.boss.max)` = 5–7

**Rarity distribution:** NOT rolled game-side. Arche's blueprint weights determine rarity. The game requests items with a level constraint only (`creatureLevel ± GAME_CONFIG.generationWindow.itemLevelVariance`) — no archetype filter, no rarity filter. The game reads `rarity` from the Arche response for visual styling (common `#888`, uncommon `#4488ff`, rare `#44cc66`, legendary `#aa44ff`).

**Item generation:** Items are generated via `POST /api/generate` (see issue 06). Arche integration in slice 06 is separate — for now generate mock items with random subtype, a placeholder name like "Iron Sword", and stats matching a randomly chosen rarity. Item level: `creatureLevel ± GAME_CONFIG.generationWindow.itemLevelVariance` (default 2).

**Loot popup:**
- Centered card after victory. Header: "VICTORY!" with XP gain and level-up notification badge.
- "LOOT" label with a grid of dropped items (4 columns).
- Each item: icon (FontAwesome matching item type) + name + rarity badge (colored triangle corner).
- Actions: `[↑/↓]` navigate, `[Enter]` take selected, `[Space]` take all, `[Esc]` leave all, `[I]` open inventory dialog.
- Take applies auto-equip (if equipment slot empty → equip directly; else → goes to inventory).
- Popup stays open until all items taken or Esc pressed.
- Inventory dialog remains accessible while loot popup is open (player can manage space).

**XP display:** The XP gained from the kill displayed in the victory header.

**Generation flow:** Each drop slot calls Arche individually with only a level constraint. Arche picks a random item blueprint via weighted selection — the game does NOT pre-roll archetype or rarity. This means the same generate call could return a weapon, armor, potion, spell, or accessory depending on blueprint weights.

## Prototype references

- `demos/dungeon-crawler/prototypes/HUD/hud-battle-view.html` — loot dialog (`#loot-dialog`) with the victory header, XP display, loot grid (4 columns), rarity corner triangles (`::after` pseudo-element with `data-rarity` attribute), "Take All" button. The prototype also has drag-from-loot-to-inventory DnD which is relevant for slice 10. The loot items UI and interaction patterns are directly reusable.
- `demos/dungeon-crawler/prototypes/HUD/hud-dungeon-view.html` — same loot dialog, shown in the dungeon context. Confirms loot works identically in both exploration and combat phases.

## Acceptance criteria

- [ ] Creature defeat shows loot popup with correct item count per difficulty
- [ ] Items have correct rarity colors and icons
- [ ] Taking an item auto-equips if slot is empty, otherwise goes to inventory
- [ ] "Take All" takes all items, "Leave All" dismisses popup
- [ ] Inventory dialog accessible during loot
- [ ] XP gain displayed in victory header
- [ ] Items use FontAwesome icons matching their type

## Blocked by

08-combat-core
