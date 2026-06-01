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
