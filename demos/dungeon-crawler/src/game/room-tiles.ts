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
