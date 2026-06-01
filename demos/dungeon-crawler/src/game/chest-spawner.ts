import { GAME_CONFIG } from '../config/game-config';
import { TILE, type TileType } from './room-tiles';
import type { ChestState } from '../types';

let nextChestId = 1;

function randInt(min: number, max: number): number {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}

function chebyshev(a: { x: number; y: number }, b: { x: number; y: number }): number {
  return Math.max(Math.abs(a.x - b.x), Math.abs(a.y - b.y));
}

export function generateChestsForRoom(
  tiles: TileType[][],
  entryTile: { x: number; y: number },
): ChestState[] {
  const count = randInt(
    GAME_CONFIG.chestSpawn.minPerRoom,
    GAME_CONFIG.chestSpawn.maxPerRoom,
  );
  const rw = tiles[0]?.length ?? 0;
  const rh = tiles.length;
  const exclusion = GAME_CONFIG.chestSpawn.entryExclusionRadius;

  const floorTiles: { x: number; y: number }[] = [];
  for (let y = 0; y < rh; y++) {
    for (let x = 0; x < rw; x++) {
      if (tiles[y][x] === TILE.FLOOR) {
        if (chebyshev({ x, y }, entryTile) >= exclusion) {
          floorTiles.push({ x, y });
        }
      }
    }
  }

  const chests: ChestState[] = [];
  const available = [...floorTiles];
  for (let i = 0; i < count && available.length > 0; i++) {
    const idx = Math.floor(Math.random() * available.length);
    const pos = available[idx];
    available.splice(idx, 1);
    chests.push({
      id: `chest_${nextChestId++}`,
      position: { x: pos.x, y: pos.y },
      opened: false,
    });
  }

  return chests;
}

export function resetChestIdCounter(): void {
  nextChestId = 1;
}
