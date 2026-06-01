---
title: "Creature chase speed should be configurable with a step interval delay"
status: needs-triage
---

## Description

Enemies move (chase the player) at their full chase speed immediately when they aggro, with no human-noticeable delay between steps. The player has no time to react or flee.

## Current behaviour

In `DungeonScene.ts:525`, the single config value `GAME_CONFIG.creatureChaseSpeed` (currently `0.5`) is used as an accumulator per frame — it adds to `chaseTickAccum` every frame and triggers a creature step whenever the accumulator reaches `1.0`. On a 60fps frame rate, this means creatures move roughly every 33ms (i.e., almost every frame). This is far too fast.

```typescript
// DungeonScene.ts:524-531
const chaseSpeed = GAME_CONFIG.creatureChaseSpeed;
this.chaseTickAccum += chaseSpeed;
while (this.chaseTickAccum >= 1) {
  this.chaseTickAccum -= 1;
  this.moveCreatureToward(this.chaseTarget, this.playerX, this.playerY);
}
```

- The config only has this single `creatureChaseSpeed` value, with no per-difficulty override support.
- The name `Speed` is misleading — it's currently an accumulator rate, not a delay/interval.

## Expected behaviour

- Add a new config field `creature.stepIntervalMs` (or similar) under a `creature` section in `GAME_CONFIG`, defaulting to `1000` (1 second).
- The `stepIntervalMs` should specify how long a creature must wait between each AI step (pathfinding move toward player).
- Optionally, different values per difficulty (`normal`, `champion`, `elite`, `boss`) so elites can be faster than normals.
- The old `creatureChaseSpeed` config value should be removed or deprecated (it's at `config/game-config.ts:52` and `config.ts:74`).

## Implementation notes

Replace the frame-rate-coupled accumulator approach with a time-based interval check:

```typescript
// Pseudocode — use Phaser's this.time.now or delta accumulation
if (now - lastStepTime >= creature.stepIntervalMs) {
  moveCreatureToward(...);
  lastStepTime = now;
}
```

This makes creature speed predictable regardless of frame rate and gives players a visible window to react.

## Acceptance criteria

- [ ] `GAME_CONFIG.creature` has a `stepIntervalMs` field (default `1000`)
- [ ] Normal champions move roughly every 1 second, elites every ~800ms, bosses every ~600ms (or whatever config dictates)
- [ ] The old `creatureChaseSpeed` field is removed
- [ ] Player can visibly outrun creatures when moving away
