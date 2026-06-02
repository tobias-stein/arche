import { GAME_CONFIG } from '../config/game-config';
import { TILE, type TileType } from './room-tiles';
import type { Difficulty, CreatureState } from '../types';
import { generate, buildCreatureRequest, buildBossRequest, parseCreature } from '../api';
import { randInt, pickRandom, getAvailableFloorTiles } from './dungeon-utils';

let nextId = 1;

const DIFFICULTIES: Difficulty[] = ['normal', 'champion', 'elite'];

const CREATURE_STATS: Record<Difficulty, { hp: number; attack: number; defense: number; xp: number }> = {
  normal: { hp: 30, attack: 6, defense: 4, xp: 10 },
  champion: { hp: 50, attack: 10, defense: 7, xp: 25 },
  elite: { hp: 80, attack: 14, defense: 10, xp: 50 },
  boss: { hp: 150, attack: 20, defense: 15, xp: 100 },
};

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

export function pickSpawnPositions(
  tiles: TileType[][],
  count: number,
  excludedPositions: { x: number; y: number }[],
): { x: number; y: number }[] {
  const floorTiles = getAvailableFloorTiles(tiles, excludedPositions, GAME_CONFIG.creatureSpawn.entryExclusionRadius);

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
  excludedPositions: { x: number; y: number }[],
): Promise<CreatureState[]> {
  let count: number;
  if (isBoss) {
    count = 1;
  } else {
    count = randInt(GAME_CONFIG.creatureSpawn.minPerRoom, GAME_CONFIG.creatureSpawn.maxPerRoom);
  }

  const positions = pickSpawnPositions(tiles, count, excludedPositions);
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


