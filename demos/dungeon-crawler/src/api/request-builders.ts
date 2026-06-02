import { GAME_CONFIG } from '../config'
import type { GenerateRequest } from './types'

function computeBand(level: number): { gte: number; lte: number } {
  const variance = GAME_CONFIG.generationWindow.creatureLevelVariance
  return {
    gte: Math.max(1, level - variance),
    lte: Math.min(100, level + variance),
  }
}

export function buildCreatureRequest(playerLevel: number): GenerateRequest {
  const band = computeBand(playerLevel)
  return {
    archetype: 'creature',
    constraints: {
      level: { gte: band.gte, lte: band.lte },
      difficulty: { in: ['normal', 'champion', 'elite'] },
    },
  }
}

export function buildLootRequest(creatureLevel: number): GenerateRequest {
  const band = computeBand(creatureLevel)
  return {
    constraints: {
      level: { gte: band.gte, lte: band.lte },
    },
  }
}

export function buildBossRequest(playerLevel: number): GenerateRequest {
  const band = computeBand(playerLevel)
  return {
    archetype: 'creature',
    constraints: {
      level: { gte: band.gte, lte: band.lte },
      difficulty: { in: ['boss'] },
    },
  }
}
