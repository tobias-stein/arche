---
title: "Chests should disappear from the map after looting"
status: needs-triage
---

## Description

When a chest is opened and looted, the chest remains visible on the dungeon map as an open chest graphic. It should disappear entirely after all loot is taken.

## Current behaviour

In `DungeonScene.ts:595-599`, `openChest()` sets `chest.opened = true` and emits the loot event. In `drawChests()` (lines 259-281), opened chests are drawn using `drawOpenChest()` — an open chest sprite remains on the tile permanently.

In `GameState.ts:406-418`, `takeAllChestLoot()` emits `chest:loot-dismissed`. The `DungeonScene` listens for this via `gameState.on('chest:loot-dismissed', ...)` — but there is no listener registered for this event in `DungeonScene.ts`. The chest remains drawn as "open" forever.

## Expected behaviour

- When all items are taken from a chest (via Take All or taking items one-by-one until empty), the chest graphic should be removed from the map.
- The `DungeonScene` should listen for `chest:loot-dismissed` and remove the chest from the `this.chests` array (or at minimum stop drawing it).
- Chests should also disappear if the player chooses "Leave" (Esc) after looting — the chest is now empty and should not show on the map.

## Acceptance criteria

- [ ] Chest disappears from the map after all loot is taken
- [ ] Chest disappears from the map after "Leave" is pressed
- [ ] The chest graphics object is properly cleaned up (no memory leak)
