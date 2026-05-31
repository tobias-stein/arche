import { describe, it, expect, beforeAll } from 'vitest';
import { Dungeon } from './dungeon-generator';
import { generateRoomTiles, bfsPathfind, getDirFromDelta, TILE } from './room-tiles';

describe('DungeonGenerator', () => {
  let dungeon: Dungeon;

  beforeAll(() => {
    dungeon = new Dungeon();
  });

  it('generates rooms', () => {
    expect(dungeon.rooms.length).toBeGreaterThan(0);
  });

  it('has an entrance room at index 0', () => {
    expect(dungeon.entranceRoom).toBe(0);
  });

  it('has a boss room that is different from entrance', () => {
    expect(dungeon.bossRoom).not.toBe(dungeon.entranceRoom);
  });

  it('has tree edges connecting rooms', () => {
    expect(dungeon.treeEdges.length).toBeGreaterThanOrEqual(dungeon.rooms.length - 1);
  });

  it('all rooms are reachable via connection set (BFS)', () => {
    const n = dungeon.rooms.length;
    const visited = new Set<number>();
    const queue = [0];
    visited.add(0);

    while (queue.length > 0) {
      const cur = queue.shift()!;
      for (let nb = 0; nb < n; nb++) {
        const key = `${cur},${nb}`;
        if (dungeon.connectionSet.has(key) && !visited.has(nb)) {
          visited.add(nb);
          queue.push(nb);
        }
      }
    }

    expect(visited.size).toBe(n);
  });

  it('boss room has greatest graph distance from entrance', () => {
    const n = dungeon.rooms.length;
    const dist = Array(n).fill(-1);
    const q = [0];
    dist[0] = 0;
    for (let qi = 0; qi < q.length; qi++) {
      const cur = q[qi];
      for (let nb = 0; nb < n; nb++) {
        const key = `${cur},${nb}`;
        if (dist[nb] === -1 && dungeon.connectionSet.has(key)) {
          dist[nb] = dist[cur] + 1;
          q.push(nb);
        }
      }
    }

    const bossDist = dist[dungeon.bossRoom];
    for (let i = 0; i < n; i++) {
      expect(dist[i]).toBeLessThanOrEqual(bossDist);
    }
  });
});

describe('RoomTiles', () => {
  let dungeon: Dungeon;

  beforeAll(() => {
    dungeon = new Dungeon();
  });

  it('generates room tiles with correct dimensions', () => {
    const tiles = generateRoomTiles(dungeon, 0);
    expect(tiles.length).toBe(11);
    expect(tiles[0].length).toBe(15);
  });

  it('has walls on outer ring (except door positions)', () => {
    const tiles = generateRoomTiles(dungeon, 0);
    const cx = 7;
    const cy = 5;
    for (let x = 0; x < 15; x++) {
      if (x === cx) {
        expect([TILE.WALL, TILE.DOOR_N, TILE.DOOR_S]).toContain(tiles[0][x]);
        expect([TILE.WALL, TILE.DOOR_N, TILE.DOOR_S]).toContain(tiles[10][x]);
      } else {
        expect(tiles[0][x]).toBe(TILE.WALL);
        expect(tiles[10][x]).toBe(TILE.WALL);
      }
    }
    for (let y = 0; y < 11; y++) {
      if (y === cy) {
        expect([TILE.WALL, TILE.DOOR_W, TILE.DOOR_E]).toContain(tiles[y][0]);
        expect([TILE.WALL, TILE.DOOR_W, TILE.DOOR_E]).toContain(tiles[y][14]);
      } else {
        expect(tiles[y][0]).toBe(TILE.WALL);
        expect(tiles[y][14]).toBe(TILE.WALL);
      }
    }
  });

  it('has floor tiles in interior', () => {
    const tiles = generateRoomTiles(dungeon, 0);
    expect(tiles[1][1]).toBe(TILE.FLOOR);
    expect(tiles[5][7]).toBe(TILE.FLOOR);
    expect(tiles[9][13]).toBe(TILE.FLOOR);
  });

  it('has doors at midpoints matching connection set', () => {
    const tiles = generateRoomTiles(dungeon, 0);
    const cx = 7;
    const cy = 5;

    if (tiles[0][cx] === TILE.DOOR_N) {
      expect(dungeon.hasDoor(0, 0)).toBe(true);
    }
    if (tiles[10][cx] === TILE.DOOR_S) {
      expect(dungeon.hasDoor(0, 1)).toBe(true);
    }
    if (tiles[cy][0] === TILE.DOOR_W) {
      expect(dungeon.hasDoor(0, 2)).toBe(true);
    }
    if (tiles[cy][14] === TILE.DOOR_E) {
      expect(dungeon.hasDoor(0, 3)).toBe(true);
    }
  });
});

describe('BfsPathfind', () => {
  const tiles: (typeof TILE)[keyof typeof TILE][][] = [
    [TILE.WALL, TILE.WALL, TILE.WALL, TILE.WALL, TILE.WALL],
    [TILE.WALL, TILE.FLOOR, TILE.FLOOR, TILE.FLOOR, TILE.WALL],
    [TILE.WALL, TILE.FLOOR, TILE.WALL, TILE.FLOOR, TILE.WALL],
    [TILE.WALL, TILE.FLOOR, TILE.FLOOR, TILE.FLOOR, TILE.WALL],
    [TILE.WALL, TILE.WALL, TILE.WALL, TILE.WALL, TILE.WALL],
  ];

  it('finds path on open floor', () => {
    const path = bfsPathfind(tiles, 1, 1, 3, 3);
    expect(path).not.toBeNull();
    expect(path!.length).toBeGreaterThan(0);
    expect(path![path!.length - 1]).toEqual({ x: 3, y: 3 });
  });

  it('returns null for wall target', () => {
    expect(bfsPathfind(tiles, 1, 1, 0, 0)).toBeNull();
  });

  it('returns empty array for same position', () => {
    expect(bfsPathfind(tiles, 1, 1, 1, 1)).toEqual([]);
  });

  it('finds path between reachable tiles', () => {
    const path = bfsPathfind(tiles, 1, 1, 3, 1);
    expect(path).not.toBeNull();
  });

  it('returns null for out of bounds target', () => {
    expect(bfsPathfind(tiles, 1, 1, -1, 1)).toBeNull();
    expect(bfsPathfind(tiles, 1, 1, 1, 10)).toBeNull();
  });

  it('returns only floor tiles in the path', () => {
    const path = bfsPathfind(tiles, 1, 1, 3, 1);
    expect(path).not.toBeNull();
    for (const step of path!) {
      expect(tiles[step.y][step.x]).not.toBe(TILE.WALL);
    }
  });
});

describe('getDirFromDelta', () => {
  it('returns 0 for north', () => {
    expect(getDirFromDelta(0, -1)).toBe(0);
  });
  it('returns 1 for south', () => {
    expect(getDirFromDelta(0, 1)).toBe(1);
  });
  it('returns 2 for west', () => {
    expect(getDirFromDelta(-1, 0)).toBe(2);
  });
  it('returns 3 for east', () => {
    expect(getDirFromDelta(1, 0)).toBe(3);
  });
  it('returns -1 for invalid delta', () => {
    expect(getDirFromDelta(1, 1)).toBe(-1);
  });
});
