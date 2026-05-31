import type { GenerateRequest } from './types'

const LEVEL_BANDS = [
  { min: 1, max: 10 },
  { min: 5, max: 20 },
  { min: 15, max: 35 },
  { min: 30, max: 50 },
  { min: 45, max: 70 },
  { min: 60, max: 85 },
  { min: 80, max: 100 },
]

function findBand(level: number): { min: number; max: number } {
  for (const band of LEVEL_BANDS) {
    if (level >= band.min && level <= band.max) {
      return band
    }
  }
  return LEVEL_BANDS[LEVEL_BANDS.length - 1]
}

export function buildCreatureRequest(
  playerLevel: number,
  excludeBoss?: boolean,
): GenerateRequest {
  const band = findBand(playerLevel)
  return {
    archetype: 'creature',
    constraints: {
      level: { gte: band.min, lte: band.max },
    },
    affixes: {
      min_prefixes: 0,
      max_prefixes: 1,
      min_suffixes: 0,
      max_suffixes: 1,
    },
  }
}

export function buildLootRequest(creatureLevel: number): GenerateRequest {
  const band = findBand(creatureLevel)
  return {
    archetype: 'item',
    constraints: {
      level: { gte: band.min, lte: band.max },
    },
    affixes: {
      min_prefixes: 0,
      max_prefixes: 2,
      min_suffixes: 0,
      max_suffixes: 1,
    },
  }
}

export function buildBossRequest(playerLevel: number): GenerateRequest {
  const band = findBand(playerLevel)
  return {
    archetype: 'creature',
    constraints: {
      level: { gte: band.min, lte: band.max },
    },
    affixes: {
      min_prefixes: 2,
      max_prefixes: 2,
      min_suffixes: 1,
      max_suffixes: 1,
    },
  }
}
