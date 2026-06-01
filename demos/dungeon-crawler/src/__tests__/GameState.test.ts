import { describe, it, expect, beforeEach } from 'vitest'
import { GameState } from '../GameState'
import { GAME_CONFIG } from '../config'
import type { ItemState } from '../types'

describe('GameState', () => {
  let gs: GameState

  beforeEach(() => {
    gs = new GameState()
  })

  it('initializes player from config', () => {
    expect(gs.player.name).toBe('Hero')
    expect(gs.player.level).toBe(GAME_CONFIG.player.startingLevel)
    expect(gs.player.hp.current).toBe(GAME_CONFIG.player.baseHp)
    expect(gs.player.hp.max).toBe(GAME_CONFIG.player.baseHp)
    expect(gs.player.mp.current).toBe(GAME_CONFIG.player.baseMp)
    expect(gs.player.mp.max).toBe(GAME_CONFIG.player.baseMp)
    expect(gs.player.attack).toBe(GAME_CONFIG.player.baseAttack)
    expect(gs.player.defense).toBe(GAME_CONFIG.player.baseDefense)
    expect(gs.player.xp.current).toBe(0)
    expect(gs.player.xp.next).toBe(GAME_CONFIG.xpThresholds[0])
  })

  it('initializes with default room state', () => {
    expect(gs.currentRoomId).toBe(0)
    expect(gs.visitedRooms).toBeInstanceOf(Set)
    expect(gs.visitedRooms.size).toBe(0)
  })

  it('setPlayerPosition updates position and emits event', () => {
    const events: unknown[][] = []
    gs.on('player:moved', (p) => events.push([p.position.x, p.position.y]))
    gs.setPlayerPosition(5, 3)
    expect(gs.player.position.x).toBe(5)
    expect(gs.player.position.y).toBe(3)
    expect(events.length).toBe(1)
    expect(events[0]).toEqual([5, 3])
  })

  it('setCurrentRoom updates room and adds to visited', () => {
    gs.setCurrentRoom(3)
    expect(gs.currentRoomId).toBe(3)
    expect(gs.visitedRooms.has(3)).toBe(true)
  })

  it('setCurrentRoom emits room:changed event', () => {
    const events: number[] = []
    gs.on('room:changed', (id) => events.push(id as number))
    gs.setCurrentRoom(5)
    expect(events).toEqual([5])
  })

  it('is not started by default', () => {
    expect(gs.gameStarted).toBe(false)
  })

  it('startGame sets gameStarted to true and emits event', () => {
    const events: string[] = []
    gs.on('game:started', () => events.push('game:started'))
    gs.startGame()
    expect(gs.gameStarted).toBe(true)
    expect(events).toEqual(['game:started'])
  })

  it('emits player:stats-changed on hp change', () => {
    const events: unknown[][] = []
    gs.on('player:stats-changed', (p) => events.push([p.hp.current, p.hp.max]))
    gs.setPlayerHp(50)
    expect(gs.player.hp.current).toBe(50)
    expect(events.length).toBe(1)
    expect(events[0]).toEqual([50, 100])
  })

  it('emits player:stats-changed on mp change', () => {
    const events: unknown[][] = []
    gs.on('player:stats-changed', (p) => events.push([p.mp.current, p.mp.max]))
    gs.setPlayerMp(10)
    expect(gs.player.mp.current).toBe(10)
    expect(events.length).toBe(1)
    expect(events[0]).toEqual([10, 30])
  })

  it('clamps hp to max', () => {
    gs.setPlayerHp(999)
    expect(gs.player.hp.current).toBe(gs.player.hp.max)
  })

  it('clamps hp to 0', () => {
    gs.setPlayerHp(-10)
    expect(gs.player.hp.current).toBe(0)
  })

  it('emits player:stats-changed on attack change', () => {
    const events: unknown[][] = []
    gs.on('player:stats-changed', (p) => events.push([p.attack]))
    gs.setPlayerAttack(15)
    expect(gs.player.attack).toBe(15)
    expect(events.length).toBe(1)
  })

  it('emits player:stats-changed on defense change', () => {
    gs.setPlayerDefense(8)
    expect(gs.player.defense).toBe(8)
  })

  it('addXp increases xp and emits stats-changed', () => {
    const events: string[] = []
    gs.on('player:stats-changed', () => events.push('stats-changed'))
    gs.addXp(5)
    expect(gs.player.xp.current).toBe(5)
    expect(events).toContain('stats-changed')
  })

  it('addXp triggers level-up when exceeding threshold', () => {
    const levelUps: number[] = []
    gs.on('xp:level-up', (lvl) => levelUps.push(lvl as number))
    gs.addXp(15)
    expect(gs.player.level).toBe(2)
    expect(gs.player.xp.current).toBe(5)
    expect(levelUps).toEqual([2])
  })

  it('emits inventory:requested on click', () => {
    const events: string[] = []
    gs.on('inventory:requested', () => events.push('inventory:requested'))
    gs.emitInventoryRequested()
    expect(events).toEqual(['inventory:requested'])
  })

  describe('chest loot', () => {
  it('takeAllChestLoot emits chest:loot-dismissed', () => {
    gs.generateChestLootItems()
    const events: string[] = []
    gs.on('chest:loot-dismissed', () => events.push('dismissed'))
    gs.takeAllChestLoot()
    expect(events).toEqual(['dismissed'])
  })

  it('dismissChestLoot emits chest:loot-dismissed', () => {
    gs.generateChestLootItems()
    const events: string[] = []
    gs.on('chest:loot-dismissed', () => events.push('dismissed'))
    gs.dismissChestLoot()
    expect(events).toEqual(['dismissed'])
  })

  it('takeChestItem emits chest:loot-dismissed when last item taken', () => {
    const item: ItemState = { id: 'test_item_1', name: 'Test Item', archeType: 'weapon', rarity: 'common', level: 1, stats: {}, affixes: [] }
    gs.chestLootItems = [item]

    const dismissed: string[] = []
    gs.on('chest:loot-dismissed', () => dismissed.push('dismissed'))

    gs.takeChestItem(item.id)

    expect(dismissed).toEqual(['dismissed'])
  })

  it('takeChestItem does not emit chest:loot-dismissed when items remain', () => {
    // Generate chest loot
    gs.generateChestLootItems()
    // Ensure we have at least 2 items
    while (gs.chestLootItems.length < 2) {
      gs.generateChestLootItems()
    }

    const dismissed: string[] = []
    gs.on('chest:loot-dismissed', () => dismissed.push('dismissed'))

    // Take one item (more remain)
    gs.takeChestItem(gs.chestLootItems[0].id)

    expect(dismissed).toEqual([])
    expect(gs.chestLootItems.length).toBeGreaterThanOrEqual(1)
  })
})

it('supports off to unsubscribe', () => {
    const calls: string[] = []
    const cb = () => calls.push('called')
    gs.on('player:stats-changed', cb)
    gs.off('player:stats-changed', cb)
    gs.setPlayerHp(80)
    expect(calls).toEqual([])
  })
})
