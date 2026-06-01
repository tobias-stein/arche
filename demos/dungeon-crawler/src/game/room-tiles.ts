import { GAME_CONFIG } from '../config/game-config';
import { Dungeon } from './dungeon-generator';

export const TILE = { WALL: 0, FLOOR: 1, DOOR_N: 2, DOOR_S: 3, DOOR_E: 4, DOOR_W: 5 } as const;
export type TileType = (typeof TILE)[keyof typeof TILE];

export function generateRoomTiles(dungeon: Dungeon, roomId: number): TileType[][] {
  const rw = GAME_CONFIG.room.width;
  const rh = GAME_CONFIG.room.height;
  const tiles: TileType[][] = Array.from({ length: rh }, () => Array(rw).fill(TILE.FLOOR));

  for (let x = 0; x < rw; x++) {
    tiles[0][x] = TILE.WALL;
    tiles[rh - 1][x] = TILE.WALL;
  }
  for (let y = 0; y < rh; y++) {
    tiles[y][0] = TILE.WALL;
    tiles[y][rw - 1] = TILE.WALL;
  }

  const cx = Math.floor(rw / 2);
  const cy = Math.floor(rh / 2);

  if (dungeon.hasDoor(roomId, 0)) tiles[0][cx] = TILE.DOOR_N;
  if (dungeon.hasDoor(roomId, 1)) tiles[rh - 1][cx] = TILE.DOOR_S;
  if (dungeon.hasDoor(roomId, 2)) tiles[cy][0] = TILE.DOOR_W;
  if (dungeon.hasDoor(roomId, 3)) tiles[cy][rw - 1] = TILE.DOOR_E;

  return tiles;
}

export function getPlayerSpawn(): { x: number; y: number } {
  return { x: Math.floor(GAME_CONFIG.room.width / 2), y: Math.floor(GAME_CONFIG.room.height / 2) };
}

export function getDoorPos(dir: number): { x: number; y: number } {
  const cx = Math.floor(GAME_CONFIG.room.width / 2);
  const cy = Math.floor(GAME_CONFIG.room.height / 2);
  switch (dir) {
    case 0: return { x: cx, y: 1 };
    case 1: return { x: cx, y: GAME_CONFIG.room.height - 2 };
    case 2: return { x: 1, y: cy };
    case 3: return { x: GAME_CONFIG.room.width - 2, y: cy };
    default: return { x: cx, y: 1 };
  }
}

export function oppositeDir(d: number): number {
  return d ^ 1;
}

const DIRS_TILE: [number, number][] = [[0, -1], [0, 1], [-1, 0], [1, 0]];

export function bfsPathfind(
  tiles: TileType[][],
  fx: number,
  fy: number,
  tx: number,
  ty: number,
): { x: number; y: number }[] | null {
  const rh = tiles.length;
  const rw = tiles[0]?.length ?? 0;
  if (tx < 0 || tx >= rw || ty < 0 || ty >= rh) return null;
  if (tiles[ty][tx] === TILE.WALL) return null;
  if (fx === tx && fy === ty) return [];

  const dist = Array.from({ length: rh }, () => Array(rw).fill(-1));
  const prev = Array.from({ length: rh }, () => Array(rw).fill(null)) as ({ x: number; y: number } | null)[][];
  const q: { x: number; y: number }[] = [{ x: fx, y: fy }];
  dist[fy][fx] = 0;

  for (let qi = 0; qi < q.length; qi++) {
    const cur = q[qi];
    if (cur.x === tx && cur.y === ty) break;

    for (const [dx, dy] of DIRS_TILE) {
      const nx = cur.x + dx;
      const ny = cur.y + dy;
      if (nx < 0 || nx >= rw || ny < 0 || ny >= rh) continue;
      if (dist[ny][nx] !== -1) continue;
      if (tiles[ny][nx] === TILE.WALL) continue;

      dist[ny][nx] = dist[cur.y][cur.x] + 1;
      prev[ny][nx] = { x: cur.x, y: cur.y };
      q.push({ x: nx, y: ny });
    }
  }

  if (dist[ty][tx] === -1) return null;

  const path: { x: number; y: number }[] = [];
  let c: { x: number; y: number } | null = { x: tx, y: ty };
  while (c !== null && (c.x !== fx || c.y !== fy)) {
    path.unshift(c);
    c = prev[c.y][c.x];
  }
  return path;
}

export function getDirFromDelta(dx: number, dy: number): number {
  if (dx === 0 && dy === -1) return 0;
  if (dx === 0 && dy === 1) return 1;
  if (dx === -1 && dy === 0) return 2;
  if (dx === 1 && dy === 0) return 3;
  return -1;
}

export function getAllDoorPositions(tiles: TileType[][]): { x: number; y: number }[] {
  const doors: { x: number; y: number }[] = [];
  for (let y = 0; y < tiles.length; y++) {
    for (let x = 0; x < tiles[y].length; x++) {
      const t = tiles[y][x];
      if (t === TILE.DOOR_N || t === TILE.DOOR_S || t === TILE.DOOR_E || t === TILE.DOOR_W) {
        doors.push({ x, y });
      }
    }
  }
  return doors;
}

export function getTileRegion(
  tiles: TileType[][],
  center: { x: number; y: number },
  radius: number,
): { x: number; y: number }[] {
  const positions: { x: number; y: number }[] = [];
  const rh = tiles.length;
  const rw = tiles[0]?.length ?? 0;
  for (let dy = -radius; dy <= radius; dy++) {
    for (let dx = -radius; dx <= radius; dx++) {
      const x = center.x + dx;
      const y = center.y + dy;
      if (x >= 0 && x < rw && y >= 0 && y < rh) {
        if (tiles[y][x] === TILE.FLOOR) {
          positions.push({ x, y });
        }
      }
    }
  }
  return positions;
}
