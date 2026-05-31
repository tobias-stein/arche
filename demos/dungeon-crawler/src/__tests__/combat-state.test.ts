import { describe, it, expect, beforeEach, vi } from 'vitest'
import { GameState, resetGameState } from '../GameState'
import { GAME_CONFIG } from '../config'
import type { CreatureState, ItemState } from '../types'

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
    it('damages creature and switches to enemy turn', async () => {
      const creature = makeCreature({ hp: { current: 35, max: 35 } })
      gs.startCombat(creature)

      const damageEvents: unknown[] = []
      const turnEvents: string[] = []
      gs.on('combat:creature-damaged', (dmg, c) => damageEvents.push([dmg, c.hp.current]))
      gs.on('combat:turn-changed', (turn) => turnEvents.push(turn as string))

      await gs.processPlayerAttack(10)

      expect(creature.hp.current).toBe(25)
      expect(damageEvents).toEqual([[10, 25]])
      expect(turnEvents).toEqual(['enemy'])
      expect(gs.combatTurn).toBe('enemy')
    })

    it('resolves victory when creature HP reaches 0', async () => {
      const creature = makeCreature({ hp: { current: 5, max: 35 }, xpReward: 15 })
      gs.startCombat(creature)

      const victoryEvents: unknown[] = []
      gs.on('combat:victory', (...args) => victoryEvents.push(args))

      await gs.processPlayerAttack(10)

      expect(creature.hp.current).toBe(0)
      expect(victoryEvents.length).toBe(1)
      expect(victoryEvents[0]).toEqual(['creature_1', 15])
      expect(gs.combatVictory).toBe(true)
      expect(gs.combatTurn).toBeNull()
      expect(gs.victory).toBe(false)
    })

    it('triggers game:victory for boss kills', async () => {
      const creature = makeCreature({
        hp: { current: 5, max: 35 },
        xpReward: 15,
        difficulty: 'boss',
      })
      gs.startCombat(creature)

      const gameVictoryEvents: unknown[] = []
      gs.on('game:victory', () => gameVictoryEvents.push('game:victory'))

      await gs.processPlayerAttack(10)

      expect(gameVictoryEvents).toEqual(['game:victory'])
      expect(gs.victory).toBe(true)
    })

    it('applies recovery after victory', async () => {
      gs.player.hp.current = 50
      gs.player.mp.current = 10
      const creature = makeCreature({ hp: { current: 5, max: 35 } })
      gs.startCombat(creature)

      await gs.processPlayerAttack(10)

      const hpRecovery = Math.round(gs.player.hp.max * GAME_CONFIG.recovery.hpPercent)
      const mpRecovery = Math.round(gs.player.mp.max * GAME_CONFIG.recovery.mpPercent)
      expect(gs.player.hp.current).toBe(50 + hpRecovery)
      expect(gs.player.mp.current).toBe(10 + mpRecovery)
    })

    it('grants XP after victory', async () => {
      const creature = makeCreature({ hp: { current: 5, max: 35 }, xpReward: 15 })
      gs.startCombat(creature)

      await gs.processPlayerAttack(10)

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
      const gameOverEvents: unknown[] = []
      gs.on('combat:defeat', () => defeatEvents.push('defeat'))
      gs.on('game:over', () => gameOverEvents.push('game:over'))

      gs.processEnemyTurn(10)

      expect(gs.player.hp.current).toBe(0)
      expect(defeatEvents).toEqual(['defeat'])
      expect(gameOverEvents).toEqual(['game:over'])
      expect(gs.combatDefeat).toBe(true)
      expect(gs.gameOver).toBe(true)
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

  describe('loot generation', () => {
    it('generates loot after victory and emits loot:show', async () => {
      const creature = makeCreature({ difficulty: 'elite', level: 5, hp: { current: 5, max: 100 }, xpReward: 50 })
      gs.startCombat(creature)

      const lootEvents: unknown[] = []
      gs.on('loot:show', (items) => lootEvents.push(items))

      await gs.processPlayerAttack(50)

      expect(gs.lootItems.length).toBeGreaterThanOrEqual(3)
      expect(gs.lootItems.length).toBeLessThanOrEqual(5)
      expect(lootEvents.length).toBe(1)
    })

    it('generates correct item level range', async () => {
      const creature = makeCreature({ difficulty: 'normal', level: 10, hp: { current: 5, max: 100 } })
      gs.startCombat(creature)
      await gs.processPlayerAttack(50)

      for (const item of gs.lootItems) {
        expect(item.level).toBeGreaterThanOrEqual(8)
        expect(item.level).toBeLessThanOrEqual(12)
      }
    })
  })

  describe('takeLootItem', () => {
    it('removes item from loot and adds to inventory or equipment', async () => {
      const creature = makeCreature({ difficulty: 'elite', level: 5, hp: { current: 5, max: 100 } })
      gs.startCombat(creature)
      await gs.processPlayerAttack(50)

      const item = gs.lootItems[0]
      gs.takeLootItem(item.id)

      expect(gs.lootItems.find(i => i.id === item.id)).toBeUndefined()
      const inInventory = gs.inventory.some(slot => slot?.id === item.id)
      const inEquipment = Object.values(gs.equipment).some(eq => eq?.id === item.id)
      expect(inInventory || inEquipment).toBe(true)
    })

    it('emits loot:items-changed after taking', async () => {
      const creature = makeCreature({ difficulty: 'elite', level: 5, hp: { current: 5, max: 100 } })
      gs.startCombat(creature)
      await gs.processPlayerAttack(50)

      const changedEvents: unknown[] = []
      gs.on('loot:items-changed', (items) => changedEvents.push(items))

      gs.takeLootItem(gs.lootItems[0].id)

      expect(changedEvents.length).toBe(1)
    })

    it('auto-equips to empty equipment slot', async () => {
      const creature = makeCreature({ difficulty: 'boss', level: 5, hp: { current: 5, max: 200 } })
      gs.startCombat(creature)
      await gs.processPlayerAttack(200)

      const equipItem = gs.lootItems.find(i => i.equipSlot)
      if (equipItem) {
        const slot = equipItem.equipSlot!
        gs.takeLootItem(equipItem.id)
        expect(gs.equipment[slot]).toBe(equipItem)
      }
    })
  })

  describe('takeAllLoot', () => {
    it('takes all loot items', async () => {
      const creature = makeCreature({ difficulty: 'elite', level: 5, hp: { current: 5, max: 100 } })
      gs.startCombat(creature)
      await gs.processPlayerAttack(50)

      const prevCount = gs.lootItems.length
      gs.takeAllLoot()

      expect(gs.lootItems.length).toBe(0)
      const inventoryCount = gs.inventory.filter(s => s !== null).length
      const equipCount = Object.keys(gs.equipment).length
      expect(inventoryCount + equipCount).toBeGreaterThanOrEqual(prevCount)
    })

    it('emits loot:items-changed with empty array', async () => {
      const creature = makeCreature({ difficulty: 'elite', level: 5, hp: { current: 5, max: 100 } })
      gs.startCombat(creature)
      await gs.processPlayerAttack(50)

      const changedEvents: unknown[] = []
      gs.on('loot:items-changed', (items) => changedEvents.push(items))

      gs.takeAllLoot()

      expect(changedEvents.length).toBe(1)
      expect(changedEvents[0]).toEqual([])
    })
  })

  describe('dismissLoot', () => {
    it('clears loot items', async () => {
      const creature = makeCreature({ difficulty: 'elite', level: 5, hp: { current: 5, max: 100 } })
      gs.startCombat(creature)
      await gs.processPlayerAttack(50)

      gs.dismissLoot()

      expect(gs.lootItems).toEqual([])
    })

    it('emits loot:items-changed with empty array', async () => {
      const creature = makeCreature({ difficulty: 'elite', level: 5, hp: { current: 5, max: 100 } })
      gs.startCombat(creature)
      await gs.processPlayerAttack(50)

      const changedEvents: unknown[] = []
      gs.on('loot:items-changed', (items) => changedEvents.push(items))

      gs.dismissLoot()

      expect(changedEvents.length).toBe(1)
      expect(changedEvents[0]).toEqual([])
    })
  })

  describe('inventory capacity', () => {
    it('has 18 inventory slots', () => {
      expect(gs.inventory.length).toBe(GAME_CONFIG.capacity.inventorySlots)
    })

    it('takeAllLoot moves all items out of loot', async () => {
      const creature = makeCreature({ difficulty: 'elite', level: 5, hp: { current: 5, max: 100 } })
      gs.startCombat(creature)
      await gs.processPlayerAttack(50)

      gs.takeAllLoot()

      expect(gs.lootItems.length).toBe(0)
    })
  })
})
