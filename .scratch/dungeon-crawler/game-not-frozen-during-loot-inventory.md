---
title: Enemy movement continues while inventory or loot dialogs are open
status: ready-for-agent
---

## Summary

When the player opens a chest (loot dialog) or presses `I` (inventory dialog), the game world does not freeze. Enemy creatures continue chasing and moving toward the player during these interactions. The game should freeze all enemy movement and game-loop updates while any loot/inventory dialog is visible.

## Expected behavior

Opening the inventory or loot dialog should behave like the encounter/combat pause:
- Enemies should not move or chase the player
- The player should not be able to move
- Only the inventory/loot interaction (drag-and-drop, equipment changes) should be active
- When the dialogs are closed, enemies resume their normal behavior

## Current implementation

In `DungeonScene.update()` (line 683–694), the game only checks for `combatActive` or `encounterActive`:

```typescript
update(_time: number, delta: number): void {
    this.elapsed += delta;
    if (this.gameState.combatActive || this.gameState.encounterActive) {
      this.drawCurrentRoom();
      return;  // ← freeze only for combat/encounter
    }
    this.updateAdjacentChest();
    this.processInput();
    this.updateMovement();
    this.updateCreatures();
    this.drawCurrentRoom();
}
```

There is no `inventoryOpen` or `lootOpen` flag in `GameState` that the `DungeonScene` can check to freeze the game loop.

## Fix options

**Option A (recommended): Add a `dialogOpen` flag to `GameState`**
- Add `inventoryOpen: boolean` and `lootOpen: boolean` (or a single `dialogOpen`) to `GameState`
- Set `inventoryOpen` when the inventory dialog opens/closes (via `App.tsx` state or a new event)
- Set `lootOpen` when `loot:show` / `chest:loot-show` fires, and clear on `combat:ended` / `chest:loot-dismissed`
- In `DungeonScene.update()`, also check `this.gameState.inventoryOpen || this.gameState.lootOpen` alongside the existing combat/encounter checks

**Option B: Use events** — Add `inventory:opened` / `inventory:closed` and `loot:opened` / `loot:closed` events to `GameState`, and have `DungeonScene` subscribe to them to toggle an internal freeze flag.

## Files to modify

- `src/GameState.ts` — add `inventoryOpen` / `lootOpen` state fields, setters, and/or events
- `src/App.tsx` — set the GameState flags when toggling `showInventory` and when loot visibility changes
- `src/game/DungeonScene.ts` — check the new flags in `update()` alongside the existing combat/encounter checks
