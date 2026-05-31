---
title: "Chests"
status: ready-for-agent
---

## What to build

Add chest entities to rooms that players can open for loot.

**Chest rendering:**
- Programmatic Phaser Graphics (no sprites).
- Rounded rectangle with gold gradient: linear gradient (`#8b7355` lightened 30 → `#8b7355`), margin 16% of tile.
- 1px outline darkened 40.
- "?" symbol in `#f0e8d8` centered, font size 34% of tile.

**Chest behavior:**
- Spawned in rooms alongside creatures. Number per room: implementation decision (suggest 0–3, random).
- Spawn on random floor tiles, same 4-tile exclusion from entry door.
- Not affected by aggro/combat — creatures can walk past chests.
- Player walks up to chest and interacts (press `[Enter]` on adjacent tile, or walk into it).
- Opening: visual feedback (brief highlight or flash), then loot popup appears (reusing the same loot popup from slice 8).
- Loot content: generated the same way as creature drops. Uses `GAME_CONFIG.lootDrops` tables? Chest-only loot table, or same pool as creatures — implementation decision.
- Chest remains open after looting (visual change: "?" → empty, or chest appears opened). Does not respawn on re-entry? Or does respawn? Implementation decision (suggest: chests respawn on room re-entry like creatures).

**Interaction:**
- When player is adjacent to a chest (N/S/E/W), a subtle prompt or highlight indicates it's interactable.
- `[Enter]` or click/tap to open.
- If a creature is aggro'd and chasing, player can still open chests (creature may catch them during loot).

## Prototype references

- `demos/dungeon-crawler/prototypes/room-layout-and-assets/room-layout-and-assets.html` — chest rendering (`drawChest()`) with the gold gradient rounded rectangle, outline, and centered "?" symbol. Translate directly from Canvas 2D to Phaser Graphics.

## Acceptance criteria

- [ ] Chest entity renders with gold gradient rounded rect and "?" symbol
- [ ] 0–3 chests spawn per room on random floor tiles
- [ ] Player can open chest by walking into it or pressing Enter adjacent
- [ ] Opening chest shows loot popup with generated items
- [ ] Chest appears opened after looting

## Blocked by

08-loot-system
