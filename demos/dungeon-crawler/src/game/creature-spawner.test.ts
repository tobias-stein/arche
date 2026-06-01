import { describe, it, expect, beforeEach } from 'vitest';
import { TILE, type TileType } from './room-tiles';
import {
  generateMockCreature,
  pickSpawnPositions,
  pickDifficulties,
  generateCreaturesForRoom,
} from './creature-spawner';

function makeEmptyRoom(doors?: { n?: boolean; s?: boolean; w?: boolean; e?: boolean }): TileType[][] {
  const tiles: TileType[][] = Array.from({ length: 11 }, () => Array(15).fill(TILE.FLOOR));
  for (let x = 0; x < 15; x++) {
    tiles[0][x] = TILE.WALL;
    tiles[10][x] = TILE.WALL;
  }
  for (let y = 0; y < 11; y++) {
    tiles[y][0] = TILE.WALL;
    tiles[y][14] = TILE.WALL;
  }
  if (doors?.n) tiles[0][7] = TILE.DOOR_N;
  if (doors?.s) tiles[10][7] = TILE.DOOR_S;
  if (doors?.w) tiles[5][0] = TILE.DOOR_W;
  if (doors?.e) tiles[5][14] = TILE.DOOR_E;
  return tiles;
}

describe('generateMockCreature', () => {
  it('creates a creature with correct structure', () => {
    const c = generateMockCreature('normal', 1, 5, 3);
    expect(c.id).toMatch(/^creature_\d+$/);
    expect(c.name).toBeTruthy();
    expect(c.archeType).toBe('creature');
    expect(c.difficulty).toBe('normal');
    expect(c.level).toBeGreaterThanOrEqual(1);
    expect(c.hp.current).toBe(c.hp.max);
    expect(c.hp.max).toBeGreaterThan(0);
    expect(c.attack).toBeGreaterThan(0);
    expect(c.defense).toBeGreaterThan(0);
    expect(c.xpReward).toBeGreaterThan(0);
    expect(c.position).toEqual({ x: 5, y: 3 });
    expect(c.aggroRange).toBe(1);
    expect(c.aggro).toBe(false);
    expect(c.stunned).toBe(false);
    expect(c.stunTimer).toBe(0);
  });

  it('creates boss difficulty creatures', () => {
    const c = generateMockCreature('boss', 5, 0, 0);
    expect(c.difficulty).toBe('boss');
    expect(c.hp.max).toBeGreaterThan(100);
    expect(c.aggroRange).toBe(3);
  });

  it('adjusts level based on player level', () => {
    const c = generateMockCreature('elite', 10, 0, 0);
    expect(c.level).toBeGreaterThanOrEqual(8);
    expect(c.level).toBeLessThanOrEqual(12);
  });
});

describe('pickSpawnPositions', () => {
  let tiles: TileType[][];

  beforeEach(() => {
    tiles = makeEmptyRoom({ n: true });
  });

  it('returns correct number of positions', () => {
    const positions = pickSpawnPositions(tiles, 3, [{ x: 7, y: 0 }, { x: 7, y: 1 }]);
    expect(positions.length).toBe(3);
  });

  it('excludes specified positions', () => {
    const excluded = [{ x: 7, y: 0 }, { x: 7, y: 1 }, { x: 7, y: 2 }, { x: 7, y: 3 }];
    const positions = pickSpawnPositions(tiles, 10, excluded);
    for (const pos of positions) {
      expect(excluded.some(e => e.x === pos.x && e.y === pos.y)).toBe(false);
    }
  });

  it('returns only floor tiles', () => {
    const positions = pickSpawnPositions(tiles, 10, [{ x: 7, y: 0 }, { x: 7, y: 1 }]);
    for (const pos of positions) {
      expect(tiles[pos.y][pos.x]).toBe(TILE.FLOOR);
    }
  });

  it('returns fewer positions when floor tiles are limited', () => {
    const smallTiles: TileType[][] = [
      [TILE.WALL, TILE.WALL, TILE.WALL],
      [TILE.WALL, TILE.FLOOR, TILE.WALL],
      [TILE.WALL, TILE.FLOOR, TILE.WALL],
      [TILE.WALL, TILE.WALL, TILE.WALL],
    ];
    const positions = pickSpawnPositions(smallTiles, 5, []);
    expect(positions.length).toBeLessThanOrEqual(2);
  });
});

describe('pickDifficulties', () => {
  it('returns boss difficulty for boss rooms', () => {
    const diffs = pickDifficulties(1, true);
    expect(diffs).toEqual(['boss']);
  });

  it('returns non-boss difficulties for regular rooms', () => {
    const diffs = pickDifficulties(5, false);
    for (const d of diffs) {
      expect(['normal', 'champion', 'elite']).toContain(d);
    }
  });

  it('returns the requested count', () => {
    expect(pickDifficulties(3, false).length).toBe(3);
    expect(pickDifficulties(1, true).length).toBe(1);
  });
});

describe('generateCreaturesForRoom', () => {
  it('generates creatures for a non-boss room', async () => {
    const tiles = makeEmptyRoom({ n: true });
    const creatures = await generateCreaturesForRoom(tiles, 0, false, 1, [{ x: 7, y: 0 }, { x: 7, y: 1 }]);
    expect(creatures.length).toBeGreaterThanOrEqual(1);
    expect(creatures.length).toBeLessThanOrEqual(4);
    for (const c of creatures) {
      expect(c.difficulty).not.toBe('boss');
      expect(tiles[c.position.y][c.position.x]).toBe(TILE.FLOOR);
    }
  });

  it('generates exactly 1 boss creature for boss room', async () => {
    const tiles = makeEmptyRoom({ n: true });
    const creatures = await generateCreaturesForRoom(tiles, 0, true, 5, [{ x: 7, y: 0 }, { x: 7, y: 1 }]);
    expect(creatures.length).toBe(1);
    expect(creatures[0].difficulty).toBe('boss');
  });

  it('creatures have valid positions within room', async () => {
    const tiles = makeEmptyRoom({ n: true });
    const creatures = await generateCreaturesForRoom(tiles, 0, false, 1, [{ x: 7, y: 0 }, { x: 7, y: 1 }]);
    for (const c of creatures) {
      expect(c.position.x).toBeGreaterThanOrEqual(0);
      expect(c.position.x).toBeLessThan(15);
      expect(c.position.y).toBeGreaterThanOrEqual(0);
      expect(c.position.y).toBeLessThan(11);
    }
  });

  it('respawning generates new creatures', async () => {
    const tiles = makeEmptyRoom({ n: true });
    const c1 = await generateCreaturesForRoom(tiles, 0, false, 1, [{ x: 7, y: 0 }, { x: 7, y: 1 }]);
    const c2 = await generateCreaturesForRoom(tiles, 0, false, 1, [{ x: 7, y: 0 }, { x: 7, y: 1 }]);
    const ids1 = new Set(c1.map(c => c.id));
    const ids2 = new Set(c2.map(c => c.id));
    for (const id of ids1) {
      expect(ids2.has(id)).toBe(false);
    }
  });
});

