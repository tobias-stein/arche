import { describe, it, expect, beforeEach, vi } from 'vitest'
import { GameState } from '../GameState'
import { GAME_CONFIG } from '../config'
import type { CreatureState } from '../types'

function makeCreature(overrides?: Partial<CreatureState>): CreatureState {
  return {
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
    ...overrides,
  }
}

describe('GameState combat', () => {
  let gs: GameState

  beforeEach(() => {
    gs = new GameState()
  })

  describe('startCombat', () => {
    it('sets combat state and emits combat:started', () => {
      const creature = makeCreature()
      const events: unknown[] = []
      gs.on('combat:started', (c) => events.push(c))

      gs.startCombat(creature)

      expect(gs.combatActive).toBe(true)
      expect(gs.combatCreature).toBe(creature)
      expect(gs.combatTurn).toBe('player')
      expect(gs.combatVictory).toBe(false)
      expect(gs.combatDefeat).toBe(false)
      expect(events).toEqual([creature])
    })
  })

  describe('processPlayerAttack', () => {
    it('damages creature and switches to enemy turn', () => {
      const creature = makeCreature({ hp: { current: 35, max: 35 } })
      gs.startCombat(creature)

      const damageEvents: unknown[] = []
      const turnEvents: string[] = []
      gs.on('combat:creature-damaged', (dmg, c) => damageEvents.push([dmg, c.hp.current]))
      gs.on('combat:turn-changed', (turn) => turnEvents.push(turn as string))

      gs.processPlayerAttack(10)

      expect(creature.hp.current).toBe(25)
      expect(damageEvents).toEqual([[10, 25]])
      expect(turnEvents).toEqual(['enemy'])
      expect(gs.combatTurn).toBe('enemy')
    })

    it('resolves victory when creature HP reaches 0', () => {
      const creature = makeCreature({ hp: { current: 5, max: 35 }, xpReward: 15 })
      gs.startCombat(creature)

      const victoryEvents: unknown[] = []
      gs.on('combat:victory', (...args) => victoryEvents.push(args))

      gs.processPlayerAttack(10)

      expect(creature.hp.current).toBe(0)
      expect(victoryEvents.length).toBe(1)
      expect(victoryEvents[0]).toEqual(['creature_1', 15])
      expect(gs.combatVictory).toBe(true)
      expect(gs.combatTurn).toBeNull()
    })

    it('applies recovery after victory', () => {
      gs.player.hp.current = 50
      gs.player.mp.current = 10
      const creature = makeCreature({ hp: { current: 5, max: 35 } })
      gs.startCombat(creature)

      gs.processPlayerAttack(10)

      const hpRecovery = Math.round(gs.player.hp.max * GAME_CONFIG.recovery.hpPercent)
      const mpRecovery = Math.round(gs.player.mp.max * GAME_CONFIG.recovery.mpPercent)
      expect(gs.player.hp.current).toBe(50 + hpRecovery)
      expect(gs.player.mp.current).toBe(10 + mpRecovery)
    })

    it('grants XP after victory', () => {
      const creature = makeCreature({ hp: { current: 5, max: 35 }, xpReward: 15 })
      gs.startCombat(creature)

      gs.processPlayerAttack(10)

      expect(gs.player.xp.current).toBe(5)
    })
  })

  describe('processEnemyTurn', () => {
    it('damages player and switches to player turn', () => {
      const creature = makeCreature()
      gs.startCombat(creature)

      const damageEvents: unknown[] = []
      const turnEvents: string[] = []
      gs.on('combat:player-damaged', (dmg, p) => damageEvents.push([dmg, p.hp.current]))
      gs.on('combat:turn-changed', (turn) => turnEvents.push(turn as string))

      gs.processEnemyTurn(8)

      expect(gs.player.hp.current).toBe(92)
      expect(damageEvents).toEqual([[8, 92]])
      expect(turnEvents).toEqual(['player'])
      expect(gs.combatTurn).toBe('player')
    })

    it('resolves defeat when player HP reaches 0', () => {
      gs.player.hp.current = 5
      const creature = makeCreature()
      gs.startCombat(creature)

      const defeatEvents: unknown[] = []
      gs.on('combat:defeat', () => defeatEvents.push('defeat'))

      gs.processEnemyTurn(10)

      expect(gs.player.hp.current).toBe(0)
      expect(defeatEvents).toEqual(['defeat'])
      expect(gs.combatDefeat).toBe(true)
      expect(gs.combatTurn).toBeNull()
    })
  })

  describe('fleeCombat', () => {
    it('emits combat:fled and ends combat', () => {
      const creature = makeCreature()
      gs.startCombat(creature)

      const fleeEvents: string[] = []
      const endEvents: string[] = []
      gs.on('combat:fled', (id) => fleeEvents.push(id as string))
      gs.on('combat:ended', () => endEvents.push('ended'))

      gs.fleeCombat()

      expect(fleeEvents).toEqual(['creature_1'])
      expect(endEvents).toEqual(['ended'])
      expect(gs.combatActive).toBe(false)
      expect(gs.combatCreature).toBeNull()
    })
  })

  describe('endCombat', () => {
    it('resets combat state and emits combat:ended', () => {
      const creature = makeCreature()
      gs.startCombat(creature)

      const endEvents: string[] = []
      gs.on('combat:ended', () => endEvents.push('ended'))

      gs.endCombat()

      expect(gs.combatActive).toBe(false)
      expect(gs.combatCreature).toBeNull()
      expect(gs.combatTurn).toBeNull()
      expect(endEvents).toEqual(['ended'])
    })
  })
})
