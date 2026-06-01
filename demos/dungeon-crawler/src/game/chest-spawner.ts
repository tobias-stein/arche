import { GAME_CONFIG } from '../config/game-config';
import { type TileType } from './room-tiles';
import type { ChestState } from '../types';
import { randInt, getAvailableFloorTiles } from './dungeon-utils';

let nextChestId = 1;

export function generateChestsForRoom(
  tiles: TileType[][],
  exclusionCenters: { x: number; y: number }[],
): ChestState[] {
  const count = randInt(
    GAME_CONFIG.chestSpawn.minPerRoom,
    GAME_CONFIG.chestSpawn.maxPerRoom,
  );
  const floorTiles = getAvailableFloorTiles(tiles, exclusionCenters, GAME_CONFIG.chestSpawn.entryExclusionRadius);

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
