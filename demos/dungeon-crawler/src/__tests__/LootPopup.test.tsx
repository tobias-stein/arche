import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, fireEvent, cleanup, act, waitFor } from '@testing-library/react'
import { getGameState, resetGameState } from '../GameState'
import LootPopup from '../components/LootPopup'
import type { CreatureState } from '../types'
import { resetItemIdCounter } from '../game/loot'

function makeCreature(overrides?: Partial<CreatureState>): CreatureState {
  return {
    id: 'creature_1',
    name: 'Goblin',
    archeType: 'creature',
    subtype: 'monster',
    difficulty: 'elite',
    level: 5,
    hp: { current: 10, max: 80 },
    attack: 10,
    defense: 6,
    xpReward: 35,
    position: { x: 5, y: 3 },
    aggroRange: 2,
    aggro: false,
    stunned: false,
    stunTimer: 0,
    ...overrides,
  }
}

beforeEach(() => {
  resetItemIdCounter()
  resetGameState()
})

afterEach(() => {
  cleanup()
})

describe('LootPopup', () => {
  it('renders victory header with XP when loot is shown', async () => {
    const gs = getGameState()
    const creature = makeCreature()

    render(<LootPopup />)

    act(() => {
      gs.startCombat(creature)
      gs.resolveVictory()
    })

    await waitFor(() => {
      expect(screen.getByText(/Victory/i)).toBeInTheDocument()
    })
    expect(screen.getByText(/\+35 XP/)).toBeInTheDocument()
  })

  it('renders loot items in a grid', async () => {
    const gs = getGameState()
    const creature = makeCreature({ difficulty: 'elite', level: 5, hp: { current: 10, max: 80 } })

    render(<LootPopup />)

    act(() => {
      gs.startCombat(creature)
      gs.resolveVictory()
    })

    await waitFor(() => {
      expect(screen.getByText(/LOOT/)).toBeInTheDocument()
    })

    expect(gs.lootItems.length).toBeGreaterThanOrEqual(3)
  })

  it('shows level-up badge when player levels up', async () => {
    const gs = getGameState()
    gs.player.xp.current = 8
    gs.player.xp.next = 10
    // Use champion difficulty to guarantee at least 1 loot item
    const creature = makeCreature({ difficulty: 'champion', level: 1, hp: { current: 5, max: 50 }, xpReward: 10 })

    render(<LootPopup />)

    act(() => {
      gs.startCombat(creature)
      gs.processPlayerAttack(30)
    })

    await waitFor(() => {
      expect(screen.getByText(/Level Up/i)).toBeInTheDocument()
    })
  })

  it('calls takeAllLoot when Take All is clicked', async () => {
    const gs = getGameState()
    const creature = makeCreature({ difficulty: 'elite', level: 5, hp: { current: 10, max: 80 } })

    render(<LootPopup />)

    act(() => {
      gs.startCombat(creature)
      gs.resolveVictory()
    })

    await waitFor(() => {
      expect(screen.getByText(/Take All/)).toBeInTheDocument()
    })

    const prevCount = gs.lootItems.length
    fireEvent.click(screen.getByText(/Take All/))

    await waitFor(() => {
      expect(gs.lootItems.length).toBe(0)
    })
  })

  it('dismisses loot and ends combat on Leave', async () => {
    const gs = getGameState()
    const creature = makeCreature({ difficulty: 'elite', level: 5, hp: { current: 10, max: 80 } })

    render(<LootPopup />)

    act(() => {
      gs.startCombat(creature)
      gs.resolveVictory()
    })

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /Leave/i })).toBeInTheDocument()
    })

    fireEvent.click(screen.getByRole('button', { name: /Leave/i }))

    expect(gs.combatActive).toBe(false)
    expect(gs.lootItems.length).toBe(0)
  })

  it('handles keyboard navigation (Enter to take item)', async () => {
    const gs = getGameState()
    const creature = makeCreature({ difficulty: 'elite', level: 5, hp: { current: 10, max: 80 } })

    render(<LootPopup />)

    act(() => {
      gs.startCombat(creature)
      gs.resolveVictory()
    })

    await waitFor(() => {
      expect(screen.getByText(/LOOT/)).toBeInTheDocument()
    })

    const prevCount = gs.lootItems.length
    act(() => {
      fireEvent.keyDown(window, { key: 'Enter' })
    })

    await waitFor(() => {
      expect(gs.lootItems.length).toBe(prevCount - 1)
    })
  })

  it('handles Space to take all items', async () => {
    const gs = getGameState()
    const creature = makeCreature({ difficulty: 'elite', level: 5, hp: { current: 10, max: 80 } })

    render(<LootPopup />)

    act(() => {
      gs.startCombat(creature)
      gs.resolveVictory()
    })

    await waitFor(() => {
      expect(screen.getByText(/LOOT/)).toBeInTheDocument()
    })

    act(() => {
      fireEvent.keyDown(window, { key: ' ' })
    })

    await waitFor(() => {
      expect(gs.lootItems.length).toBe(0)
    })
  })

  it('handles Esc to leave all items', async () => {
    const gs = getGameState()
    const creature = makeCreature({ difficulty: 'elite', level: 5, hp: { current: 10, max: 80 } })

    render(<LootPopup />)

    act(() => {
      gs.startCombat(creature)
      gs.resolveVictory()
    })

    await waitFor(() => {
      expect(screen.getByText(/LOOT/)).toBeInTheDocument()
    })

    act(() => {
      fireEvent.keyDown(window, { key: 'Escape' })
    })

    await waitFor(() => {
      expect(gs.combatActive).toBe(false)
    })
  })

  it('handles I key to open inventory', async () => {
    const gs = getGameState()
    const creature = makeCreature({ difficulty: 'elite', level: 5, hp: { current: 10, max: 80 } })

    render(<LootPopup />)

    act(() => {
      gs.startCombat(creature)
      gs.resolveVictory()
    })

    await waitFor(() => {
      expect(screen.getByText(/LOOT/)).toBeInTheDocument()
    })

    const inventoryEvents: unknown[] = []
    gs.on('inventory:requested', () => inventoryEvents.push('inv'))

    act(() => {
      fireEvent.keyDown(window, { key: 'i' })
    })

    expect(inventoryEvents).toEqual(['inv'])
  })

  it('renders data-rarity attribute on items', async () => {
    const gs = getGameState()
    const creature = makeCreature({ difficulty: 'elite', level: 5, hp: { current: 10, max: 80 } })

    const { container } = render(<LootPopup />)

    act(() => {
      gs.startCombat(creature)
      gs.resolveVictory()
    })

    await waitFor(() => {
      expect(screen.getByText(/LOOT/)).toBeInTheDocument()
    })

    const items = container.querySelectorAll('.ld-item')
    expect(items.length).toBeGreaterThanOrEqual(3)
    for (const item of Array.from(items)) {
      const rarity = item.getAttribute('data-rarity')
      expect(['common', 'uncommon', 'rare', 'legendary']).toContain(rarity)
    }
  })
})
