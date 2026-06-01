import { GAME_CONFIG } from '../config/game-config';
import { TILE, type TileType } from './room-tiles';
import type { Difficulty, CreatureState } from '../types';
import { generate, buildCreatureRequest, buildBossRequest, parseCreature } from '../api';

let nextId = 1;

const DIFFICULTIES: Difficulty[] = ['normal', 'champion', 'elite'];

const CREATURE_STATS: Record<Difficulty, { hp: number; attack: number; defense: number; xp: number }> = {
  normal: { hp: 30, attack: 6, defense: 4, xp: 10 },
  champion: { hp: 50, attack: 10, defense: 7, xp: 25 },
  elite: { hp: 80, attack: 14, defense: 10, xp: 50 },
  boss: { hp: 150, attack: 20, defense: 15, xp: 100 },
};

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

  const stats = CREATURE_STATS[difficulty];
  const hpMax = stats.hp + level * 5;
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
    attack: stats.attack + level * 2,
    defense: stats.defense + level,
    xpReward: stats.xp + level * 5,
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
  exclusionCenters: { x: number; y: number }[],
): { x: number; y: number }[] {
  const rw = tiles[0]?.length ?? 0;
  const rh = tiles.length;
  const exclusion = GAME_CONFIG.creatureSpawn.entryExclusionRadius;

  const floorTiles: { x: number; y: number }[] = [];
  for (let y = 0; y < rh; y++) {
    for (let x = 0; x < rw; x++) {
      if (tiles[y][x] === TILE.FLOOR) {
        const isExcluded = exclusionCenters.some(
          center => chebyshev({ x, y }, center) < exclusion,
        );
        if (!isExcluded) {
          floorTiles.push({ x, y });
        }
      }
    }
  }

  const positions: { x: number; y: number }[] = [];
  const available = [...floorTiles];

  for (let i = 0; i < count && available.length > 0; i++) {
    const idx = Math.floor(Math.random() * available.length);
    const pos = available[idx];
    available.splice(idx, 1);
    positions.push(pos);
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

export async function generateCreaturesForRoom(
  tiles: TileType[][],
  roomId: number,
  isBoss: boolean,
  playerLevel: number,
  exclusionCenters: { x: number; y: number }[],
): Promise<CreatureState[]> {
  let count: number;
  if (isBoss) {
    count = 1;
  } else {
    count = randInt(GAME_CONFIG.creatureSpawn.minPerRoom, GAME_CONFIG.creatureSpawn.maxPerRoom);
  }

  const positions = pickSpawnPositions(tiles, count, exclusionCenters);
  const difficulties = pickDifficulties(positions.length, isBoss);

  try {
    const request = isBoss ? buildBossRequest(playerLevel) : buildCreatureRequest(playerLevel);
    const response = await generate(request);
    const creatures: CreatureState[] = [];
    const usedThings = response.things.slice(0, positions.length);
    for (let i = 0; i < usedThings.length; i++) {
      creatures.push(parseCreature(usedThings[i], positions[i].x, positions[i].y));
    }
    return creatures;
  } catch {
    // Fallback to mock data
  }

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
