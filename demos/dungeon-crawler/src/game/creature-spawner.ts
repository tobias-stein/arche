import { GAME_CONFIG } from '../config/game-config';
import { TILE, type TileType } from './room-tiles';
import type { Difficulty, CreatureState } from '../types';

let nextId = 1;

const DIFFICULTIES: Difficulty[] = ['normal', 'champion', 'elite'];

function randInt(min: number, max: number): number {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}

function pickRandom<T>(arr: T[]): T {
  return arr[Math.floor(Math.random() * arr.length)];
}

function chebyshev(a: { x: number; y: number }, b: { x: number; y: number }): number {
  return Math.max(Math.abs(a.x - b.x), Math.abs(a.y - b.y));
}

export function generateMockCreature(
  difficulty: Difficulty,
  playerLevel: number,
  x: number,
  y: number,
): CreatureState {
  const levelVariance = GAME_CONFIG.generationWindow?.creatureLevelVariance ?? 2;
  const level = Math.max(1, playerLevel + randInt(-levelVariance, levelVariance));

  const hpBase = difficulty === 'boss' ? 150 : difficulty === 'elite' ? 80 : difficulty === 'champion' ? 50 : 30;
  const attackBase = difficulty === 'boss' ? 20 : difficulty === 'elite' ? 14 : difficulty === 'champion' ? 10 : 6;
  const defenseBase = difficulty === 'boss' ? 15 : difficulty === 'elite' ? 10 : difficulty === 'champion' ? 7 : 4;
  const xpBase = difficulty === 'boss' ? 100 : difficulty === 'elite' ? 50 : difficulty === 'champion' ? 25 : 10;

  const hpMax = hpBase + level * 5;
  const names: Record<Difficulty, string[]> = {
    normal: ['Rat', 'Spider', 'Slime', 'Skeleton'],
    champion: ['Dire Rat', 'Shadow Spider', 'Acid Slime', 'Bone Warrior'],
    elite: ['Plague Rat', 'Night Weaver', 'Elder Slime', 'Death Knight'],
    boss: ['Vermin King', 'Arachna', 'The Blob', 'Bone Lord'],
  };

  return {
    id: `creature_${nextId++}`,
    name: pickRandom(names[difficulty]),
    archeType: 'creature',
    subtype: 'monster',
    difficulty,
    level,
    hp: { current: hpMax, max: hpMax },
    attack: attackBase + level * 2,
    defense: defenseBase + level,
    xpReward: xpBase + level * 5,
    position: { x, y },
    aggroRange: GAME_CONFIG.aggroRanges[difficulty],
    aggro: false,
    stunned: false,
    stunTimer: 0,
  };
}

export interface SpawnSlot {
  x: number;
  y: number;
  difficulty: Difficulty;
}

export function pickSpawnPositions(
  tiles: TileType[][],
  count: number,
  entryTile: { x: number; y: number },
): { x: number; y: number }[] {
  const rw = tiles[0]?.length ?? 0;
  const rh = tiles.length;
  const exclusion = GAME_CONFIG.creatureSpawn.entryExclusionRadius;

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

  const positions: { x: number; y: number }[] = [];
  const available = [...floorTiles];
  const taken = new Set<string>();

  for (let i = 0; i < count && available.length > 0; i++) {
    const idx = Math.floor(Math.random() * available.length);
    const pos = available[idx];
    available.splice(idx, 1);
    positions.push(pos);
    taken.add(`${pos.x},${pos.y}`);
  }

  return positions;
}

export function pickDifficulties(
  count: number,
  isBoss: boolean,
): Difficulty[] {
  if (isBoss) {
    return ['boss'];
  }
  const diffs: Difficulty[] = [];
  for (let i = 0; i < count; i++) {
    diffs.push(pickRandom(DIFFICULTIES));
  }
  return diffs;
}

export function generateCreaturesForRoom(
  tiles: TileType[][],
  roomId: number,
  isBoss: boolean,
  playerLevel: number,
  entryTile: { x: number; y: number },
): CreatureState[] {
  let count: number;
  if (isBoss) {
    count = 1;
  } else {
    count = randInt(GAME_CONFIG.creatureSpawn.minPerRoom, GAME_CONFIG.creatureSpawn.maxPerRoom);
  }

  const positions = pickSpawnPositions(tiles, count, entryTile);
  const difficulties = pickDifficulties(positions.length, isBoss);

  const creatures: CreatureState[] = [];
  for (let i = 0; i < positions.length; i++) {
    const { x, y } = positions[i];
    const diff = difficulties[i] ?? 'normal';
    creatures.push(generateMockCreature(diff, playerLevel, x, y));
  }

  return creatures;
}

export function getEntryTile(tiles: TileType[][]): { x: number; y: number } {
  const rw = tiles[0]?.length ?? 0;
  const rh = tiles.length;
  for (let y = 0; y < rh; y++) {
    for (let x = 0; x < rw; x++) {
      if (tiles[y][x] === TILE.DOOR_N) return { x, y };
    }
  }
  return { x: Math.floor(rw / 2), y: Math.floor(rh / 2) };
}
