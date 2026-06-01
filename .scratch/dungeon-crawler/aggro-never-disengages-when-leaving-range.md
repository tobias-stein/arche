---
title: Aggro never disengages when player leaves aggro range
status: completed
---

## Summary

Once a creature becomes aggro'd on the player, it remains aggro'd forever — even if the player moves far outside the creature's aggro range. Creatures should de-aggro and stop chasing when the player leaves their aggro range, and should only re-aggro if the player re-enters the range.

## Expected behavior

- A creature that is currently chasing the player should check each update whether the player is still within its aggro range
- If the player leaves the aggro range, the creature should de-aggro, stop chasing, and remain at its current position
- If the player re-enters the aggro range, the creature should re-aggro and resume chasing

## Current implementation

In `DungeonScene.updateCreatures()` (lines 534–581), aggro is set to `true` when the player enters range, but is **never set back to `false`**:

```typescript
if (!c.aggro) {
    const dist = Math.max(
        Math.abs(c.position.x - this.playerX),
        Math.abs(c.position.y - this.playerY),
    );
    if (dist <= c.aggroRange) {
        c.aggro = true;   // ← Set, but never cleared on exit
    }
}
```

The only place `aggro` is reset to `false` is in `stunCreature()` (line 628) after a flee, which is a completely separate scenario.

## Fix

In the `updateCreatures()` loop, add a check that de-aggros creatures when the player leaves the aggro range:

```typescript
if (c.aggro) {
    const dist = Math.max(
        Math.abs(c.position.x - this.playerX),
        Math.abs(c.position.y - this.playerY),
    );
    if (dist > c.aggroRange) {
        c.aggro = false;
    }
}
```

This should be added in the same loop that sets `aggro = true`, ideally after the `!c.aggro` branch.

## Files to modify

- `src/game/DungeonScene.ts` — add de-aggro check in `updateCreatures()` (around lines 549–558)
