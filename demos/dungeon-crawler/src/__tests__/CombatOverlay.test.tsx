import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, fireEvent, cleanup, act } from '@testing-library/react'
import CombatOverlay from '../components/CombatOverlay'
import { getGameState, resetGameState } from '../GameState'
import type { CreatureState } from '../types'

function makeCreature(overrides?: Partial<CreatureState>): CreatureState {
  return {
    id: 'c1',
    name: 'Goblin Scout',
    archeType: 'creature',
    subtype: 'monster',
    difficulty: 'normal',
    level: 3,
    hp: { current: 30, max: 40 },
    attack: 8,
    defense: 5,
    xpReward: 20,
    position: { x: 5, y: 3 },
    aggroRange: 1,
    aggro: false,
    stunned: false,
    stunTimer: 0,
    ...overrides,
  }
}

describe('CombatOverlay', () => {
  beforeEach(() => {
    resetGameState()
    vi.useFakeTimers()
  })

  afterEach(() => {
    cleanup()
    vi.useRealTimers()
  })

  function startCombat() {
    const gs = getGameState()
    act(() => {
      gs.startCombat(makeCreature())
    })
    act(() => {
      vi.advanceTimersByTime(500)
    })
  }

  it('does not render when no combat is active', () => {
    const { container } = render(<CombatOverlay />)
    expect(container.innerHTML).toBe('')
  })

  it('renders the action menu during player turn', () => {
    render(<CombatOverlay />)
    startCombat()
    expect(screen.getByText('Flee')).toBeInTheDocument()
    expect(screen.getByText('Attack')).toBeInTheDocument()
  })

  it('shows flee confirmation dialog when Flee is clicked', () => {
    render(<CombatOverlay />)
    startCombat()
    fireEvent.click(screen.getByText('Flee'))
    expect(screen.getByText('Flee from combat?')).toBeInTheDocument()
    expect(screen.getByText('Cowardice has its rewards.')).toBeInTheDocument()
    expect(screen.getByText('Yes, Flee')).toBeInTheDocument()
    expect(screen.getByText('Stay & Fight')).toBeInTheDocument()
  })

  it('executes flee when Yes, Flee is clicked', () => {
    render(<CombatOverlay />)
    startCombat()
    const gs = getGameState()
    const fleeEvents: string[] = []
    gs.on('combat:fled', (id) => fleeEvents.push(id as string))
    fireEvent.click(screen.getByText('Flee'))
    fireEvent.click(screen.getByText('Yes, Flee'))
    expect(fleeEvents.length).toBe(1)
    expect(gs.combatActive).toBe(false)
  })

  it('returns to action menu when Stay & Fight is clicked', () => {
    render(<CombatOverlay />)
    startCombat()
    fireEvent.click(screen.getByText('Flee'))
    expect(screen.getByText('Flee from combat?')).toBeInTheDocument()
    fireEvent.click(screen.getByText('Stay & Fight'))
    expect(screen.queryByText('Flee from combat?')).not.toBeInTheDocument()
    expect(screen.getByText('Flee')).toBeInTheDocument()
  })

  it('closes flee dialog with Escape key', () => {
    render(<CombatOverlay />)
    startCombat()
    fireEvent.click(screen.getByText('Flee'))
    expect(screen.getByText('Flee from combat?')).toBeInTheDocument()
    fireEvent.keyDown(window, { key: 'Escape' })
    expect(screen.queryByText('Flee from combat?')).not.toBeInTheDocument()
  })

  it('confirms flee with Enter key on Yes, Flee', () => {
    render(<CombatOverlay />)
    startCombat()
    const gs = getGameState()
    const fleeEvents: string[] = []
    gs.on('combat:fled', (id) => fleeEvents.push(id as string))
    fireEvent.click(screen.getByText('Flee'))
    expect(screen.getByText('Flee from combat?')).toBeInTheDocument()
    fireEvent.keyDown(window, { key: 'Enter' })
    expect(fleeEvents.length).toBe(1)
    expect(gs.combatActive).toBe(false)
  })

  it('does not show flee dialog after combat ends', () => {
    render(<CombatOverlay />)
    startCombat()
    const gs = getGameState()
    fireEvent.click(screen.getByText('Flee'))
    expect(screen.getByText('Flee from combat?')).toBeInTheDocument()
    act(() => {
      gs.endCombat()
    })
    expect(screen.queryByText('Flee from combat?')).not.toBeInTheDocument()
  })
})
