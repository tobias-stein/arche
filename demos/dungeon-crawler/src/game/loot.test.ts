import { describe, it, expect, beforeEach } from 'vitest'
import { generateMockItems, generateMockItem, getRarityColor, resetItemIdCounter } from './loot'
import { GAME_CONFIG } from '../config'
import type { Rarity } from '../types'

beforeEach(() => {
  resetItemIdCounter()
})

describe('generateMockItems', () => {
  const variance = GAME_CONFIG.generationWindow.itemLevelVariance

  it('generates 0-1 items for normal difficulty', async () => {
    const items = await generateMockItems('normal', 5, variance)
    expect(items.length).toBeGreaterThanOrEqual(0)
    expect(items.length).toBeLessThanOrEqual(1)
  })

  it('generates 1-3 items for champion difficulty', async () => {
    // Run multiple times to cover range
    let min = Infinity
    let max = -Infinity
    for (let i = 0; i < 50; i++) {
      const items = await generateMockItems('champion', 5, variance)
      min = Math.min(min, items.length)
      max = Math.max(max, items.length)
    }
    expect(min).toBeGreaterThanOrEqual(1)
    expect(max).toBeLessThanOrEqual(3)
  })

  it('generates 3-5 items for elite difficulty', async () => {
    let min = Infinity
    let max = -Infinity
    for (let i = 0; i < 50; i++) {
      const items = await generateMockItems('elite', 5, variance)
      min = Math.min(min, items.length)
      max = Math.max(max, items.length)
    }
    expect(min).toBeGreaterThanOrEqual(3)
    expect(max).toBeLessThanOrEqual(5)
  })

  it('generates 5-7 items for boss difficulty', async () => {
    let min = Infinity
    let max = -Infinity
    for (let i = 0; i < 50; i++) {
      const items = await generateMockItems('boss', 10, variance)
      min = Math.min(min, items.length)
      max = Math.max(max, items.length)
    }
    expect(min).toBeGreaterThanOrEqual(5)
    expect(max).toBeLessThanOrEqual(7)
  })
})

describe('generateMockItem', () => {
  const variance = GAME_CONFIG.generationWindow.itemLevelVariance

  it('generates an item with id, name, and rarity', () => {
    const item = generateMockItem(5, variance)
    expect(item.id).toBeTruthy()
    expect(item.name).toBeTruthy()
    expect(['common', 'uncommon', 'rare', 'legendary']).toContain(item.rarity)
  })

  it('generates item level within creatureLevel ± variance', () => {
    const creatureLevel = 10
    const items = Array.from({ length: 100 }, () => generateMockItem(creatureLevel, variance))
    for (const item of items) {
      expect(item.level).toBeGreaterThanOrEqual(creatureLevel - variance)
      expect(item.level).toBeLessThanOrEqual(creatureLevel + variance)
    }
  })

  it('generates item with valid archeType', () => {
    const item = generateMockItem(5, variance)
    expect(['weapon', 'helmet', 'chest', 'legs', 'boots', 'gloves', 'belt', 'ring', 'amulet', 'shield', 'consumable', 'spell']).toContain(item.archeType)
  })

  it('generates stats according to rarity', () => {
    const legItems = Array.from({ length: 50 }, () => generateMockItem(5, variance)).filter(i => i.rarity === 'legendary')
    const commonItems = Array.from({ length: 50 }, () => generateMockItem(5, variance)).filter(i => i.rarity === 'common')
    if (legItems.length > 0 && commonItems.length > 0) {
      const legStatSum = legItems.reduce((sum, i) => sum + Object.values(i.stats).reduce((a, b) => a + b, 0), 0) / legItems.length
      const comStatSum = commonItems.reduce((sum, i) => sum + Object.values(i.stats).reduce((a, b) => a + b, 0), 0) / commonItems.length
      expect(legStatSum).toBeGreaterThanOrEqual(comStatSum)
    }
  })
})

describe('getRarityColor', () => {
  it('returns correct colors for each rarity', () => {
    expect(getRarityColor('common')).toBe('#888')
    expect(getRarityColor('uncommon')).toBe('#4488ff')
    expect(getRarityColor('rare')).toBe('#44cc66')
    expect(getRarityColor('legendary')).toBe('#aa44ff')
  })
})
