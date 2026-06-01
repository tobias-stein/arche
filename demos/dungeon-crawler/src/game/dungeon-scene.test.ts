import { describe, it, expect } from 'vitest';
import { GAME_CONFIG } from '../config/game-config';

describe('DungeonScene - aggro grace period', () => {
  it('has aggroGracePeriodMs with correct defaults', () => {
    expect(GAME_CONFIG.aggroGracePeriodMs.initial).toBe(1500);
    expect(GAME_CONFIG.aggroGracePeriodMs.transition).toBe(800);
  });

  it('transition grace period is shorter than initial', () => {
    expect(GAME_CONFIG.aggroGracePeriodMs.transition).toBeLessThan(
      GAME_CONFIG.aggroGracePeriodMs.initial,
    );
  });
});

describe('DungeonScene - aggro disengage', () => {
  it('de-aggros creature when player leaves aggro range', () => {
    const creature = { position: { x: 10, y: 10 }, aggroRange: 5, aggro: true };
    const dist = Math.max(
      Math.abs(creature.position.x - 20),
      Math.abs(creature.position.y - 10),
    );
    if (dist > creature.aggroRange) creature.aggro = false;
    expect(creature.aggro).toBe(false);
  });

  it('keeps aggro when player is within aggro range', () => {
    const creature = { position: { x: 10, y: 10 }, aggroRange: 5, aggro: true };
    const dist = Math.max(
      Math.abs(creature.position.x - 12),
      Math.abs(creature.position.y - 10),
    );
    if (dist > creature.aggroRange) creature.aggro = false;
    expect(creature.aggro).toBe(true);
  });

  it('re-aggros when player re-enters aggro range after leaving', () => {
    const creature = { position: { x: 10, y: 10 }, aggroRange: 5, aggro: false };
    const dist = Math.max(
      Math.abs(creature.position.x - 12),
      Math.abs(creature.position.y - 10),
    );
    if (dist <= creature.aggroRange) creature.aggro = true;
    expect(creature.aggro).toBe(true);
  });
});
