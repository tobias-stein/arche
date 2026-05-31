import { describe, it, expect, beforeEach, vi } from 'vitest'
import { calculatePlayerDamage, calculateCreatureDamage } from './combat'
import { GAME_CONFIG } from '../config'
import type { PlayerState, CreatureState } from '../types'

let player: PlayerState
let creature: CreatureState

beforeEach(() => {
  player = {
    name: 'Hero',
    level: 1,
    xp: { current: 0, next: 10 },
    hp: { current: 100, max: 100 },
    mp: { current: 30, max: 30 },
    attack: 10,
    defense: 5,
    position: { x: 0, y: 0 },
  }
  creature = {
    id: 'creature_1',
    name: 'Rat',
    archeType: 'creature',
    subtype: 'monster',
    difficulty: 'normal',
    level: 1,
    hp: { current: 35, max: 35 },
    attack: 6,
    defense: 4,
    xpReward: 15,
    position: { x: 5, y: 3 },
    aggroRange: 1,
    aggro: false,
    stunned: false,
    stunTimer: 0,
  }
})

describe('calculatePlayerDamage', () => {
  it('deals at least minDamage', () => {
    const lowPlayer = { ...player, attack: 1 }
    const highDef = { ...creature, defense: 99 }
    const dmg = calculatePlayerDamage(lowPlayer, highDef)
    expect(dmg).toBe(GAME_CONFIG.combat.minDamage)
  })

  it('scales with attack minus defense plus random roll', () => {
    vi.spyOn(Math, 'random').mockReturnValue(0)
    const dmg = calculatePlayerDamage(player, creature)
    const expected = Math.max(GAME_CONFIG.combat.minDamage, player.attack - creature.defense + 0)
    expect(dmg).toBe(expected)
    vi.restoreAllMocks()
  })

  it('adds random roll up to playerDamageRollMax', () => {
    vi.spyOn(Math, 'random').mockReturnValue(0.999)
    const dmg = calculatePlayerDamage(player, creature)
    const expected = Math.max(GAME_CONFIG.combat.minDamage, player.attack - creature.defense + GAME_CONFIG.combat.playerDamageRollMax)
    expect(dmg).toBe(expected)
    vi.restoreAllMocks()
  })
})

describe('calculateCreatureDamage', () => {
  it('deals at least minDamage', () => {
    const lowCreature = { ...creature, attack: 1 }
    const dmg = calculateCreatureDamage(player, lowCreature)
    expect(dmg).toBe(GAME_CONFIG.combat.minDamage)
  })

  it('subtracts creatureAttackPenalty', () => {
    vi.spyOn(Math, 'random').mockReturnValue(0)
    const dmg = calculateCreatureDamage(player, creature)
    const expected = Math.max(
      GAME_CONFIG.combat.minDamage,
      creature.attack - player.defense + 0 - GAME_CONFIG.combat.creatureAttackPenalty,
    )
    expect(dmg).toBe(expected)
    vi.restoreAllMocks()
  })

  it('uses creatureDamageRollMax for random roll', () => {
    vi.spyOn(Math, 'random').mockReturnValue(0.999)
    const dmg = calculateCreatureDamage(player, creature)
    const expected = Math.max(
      GAME_CONFIG.combat.minDamage,
      creature.attack - player.defense + GAME_CONFIG.combat.creatureDamageRollMax - GAME_CONFIG.combat.creatureAttackPenalty,
    )
    expect(dmg).toBe(expected)
    vi.restoreAllMocks()
  })
})
