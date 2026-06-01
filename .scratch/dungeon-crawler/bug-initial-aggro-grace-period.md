---
title: "Enemies aggro immediately on game start and room entry with no grace period"
status: completed
---

## Description

When the game starts (first room spawn) or when entering a new room, creatures within aggro range immediately aggro and start chasing the player. There is no grace period for the player to orient themselves or move away before creatures engage.

## Current behaviour

In `DungeonScene.ts:501-509`, `updateCreatures()` checks aggro range every frame:

```typescript
if (!c.aggro) {
  const dist = Math.max(
    Math.abs(c.position.x - this.playerX),
    Math.abs(c.position.y - this.playerY),
  );
  if (dist <= c.aggroRange) {
    c.aggro = true;
  }
}
```

This runs immediately on the first frame after `spawnCreatures()`. If a creature spawns within aggro range of the player's entry position, it immediately aggros and starts chasing. The player may not have even taken their first step.

Note: `aggroRange` defaults are `1` for normal, `2` for champion, `3` for elite/boss. Combined with the fact that `entryExclusionRadius` in both spawners uses the north door position (not the player's actual entry position) means creatures can spawn very close to where the player actually enters the room (see also the related stacking/spawn-position bugs).

## Expected behaviour

- On game start (first room), apply a grace period of 1-2 seconds where no creatures aggro.
- On room entry (via door transition), apply a grace period of 500-1000ms where no creatures aggro.
- During the grace period, creatures should ignore aggro range checks.
- Consider adding a config field for the grace period duration: `GAME_CONFIG.creature.aggroGracePeriodMs` (default `1500` for initial spawn, `800` for room transitions).

## Acceptance criteria

- [ ] Player has ~1.5s to move before any creature aggroes on game start
- [ ] Player has ~800ms after entering a new room before creatures aggro
- [ ] Grace period timer resets on room transition
