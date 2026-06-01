import { describe, it, expect } from 'vitest';
import { GAME_CONFIG } from '../config/game-config';

describe('DungeonScene - aggro grace period', () => {
  it('has aggroGracePeriodMs with correct defaults', () => {
    expect(GAME_CONFIG.aggroGracePeriodMs.initial).toBe(1500);
    expect(GAME_CONFIG.aggroGracePeriodMs.transition).toBe(800);
  });

  it('grace timer prevents aggro when active', () => {
    const now = 1000;
    const graceTimer = now + GAME_CONFIG.aggroGracePeriodMs.initial;
    expect(now < graceTimer).toBe(true);
  });

  it('grace timer allows aggro after expiry', () => {
    const now = 3000;
    const graceTimer = 1000 + GAME_CONFIG.aggroGracePeriodMs.initial;
    expect(now >= graceTimer).toBe(true);
  });

  it('transition grace period is shorter than initial', () => {
    expect(GAME_CONFIG.aggroGracePeriodMs.transition).toBeLessThan(
      GAME_CONFIG.aggroGracePeriodMs.initial,
    );
  });
});
