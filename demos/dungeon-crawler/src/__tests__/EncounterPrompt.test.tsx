import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, fireEvent, cleanup, act } from '@testing-library/react'
import EncounterPrompt from '../components/EncounterPrompt'
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

describe('EncounterPrompt', () => {
  beforeEach(() => {
    resetGameState()
  })

  afterEach(() => {
    cleanup()
  })

  it('does not render when no encounter is active', () => {
    const { container } = render(<EncounterPrompt />)
    expect(container.innerHTML).toBe('')
  })

  it('renders creature name on encounter:prompt event', () => {
    render(<EncounterPrompt />)
    const gs = getGameState()
    act(() => {
      gs.showEncounterPrompt(makeCreature({ name: 'Rat King' }))
    })
    const names = screen.getAllByText('Rat King')
    expect(names.length).toBeGreaterThanOrEqual(1)
  })

  it('renders level and difficulty', () => {
    render(<EncounterPrompt />)
    const gs = getGameState()
    act(() => {
      gs.showEncounterPrompt(makeCreature({ level: 5, difficulty: 'elite' }))
    })
    const levelTexts = screen.getAllByText(/LV 5/)
    expect(levelTexts.length).toBeGreaterThanOrEqual(1)
    const eliteTexts = screen.getAllByText(/ELITE/)
    expect(eliteTexts.length).toBeGreaterThanOrEqual(1)
  })

  it('renders HP, ATK, DEF stats', () => {
    render(<EncounterPrompt />)
    const gs = getGameState()
    act(() => {
      gs.showEncounterPrompt(makeCreature({ hp: { current: 30, max: 40 }, attack: 8, defense: 5 }))
    })
    expect(screen.getByText('30/40')).toBeInTheDocument()
    expect(screen.getByText(/ATK 8/)).toBeInTheDocument()
    expect(screen.getByText(/DEF 5/)).toBeInTheDocument()
  })

  it('renders Engage and Back away buttons', () => {
    render(<EncounterPrompt />)
    const gs = getGameState()
    act(() => {
      gs.showEncounterPrompt(makeCreature())
    })
    expect(screen.getByText('Engage')).toBeInTheDocument()
    expect(screen.getByText('Back away')).toBeInTheDocument()
  })

  it('calls proceedToCombat when Engage is clicked', () => {
    render(<EncounterPrompt />)
    const gs = getGameState()
    const combatEvents: unknown[] = []
    gs.on('combat:started', (c) => combatEvents.push(c))

    act(() => {
      gs.showEncounterPrompt(makeCreature())
    })
    fireEvent.click(screen.getByText('Engage'))

    expect(combatEvents.length).toBe(1)
    expect(gs.combatActive).toBe(true)
  })

  it('calls backAwayFromEncounter when Back away is clicked', () => {
    render(<EncounterPrompt />)
    const gs = getGameState()
    const backEvents: string[] = []
    gs.on('encounter:backed-away', (id) => backEvents.push(id as string))

    act(() => {
      gs.showEncounterPrompt(makeCreature())
    })
    fireEvent.click(screen.getByText('Back away'))

    expect(backEvents.length).toBe(1)
    expect(gs.encounterActive).toBe(false)
  })

  it('does not render after encounter is dismissed', () => {
    const { container } = render(<EncounterPrompt />)
    const gs = getGameState()
    act(() => {
      gs.showEncounterPrompt(makeCreature())
    })
    expect(screen.getByText('Engage')).toBeInTheDocument()

    act(() => {
      gs.dismissEncounter()
    })
    expect(container.innerHTML).toBe('')
  })

  it('does not render after combat starts', () => {
    const { container } = render(<EncounterPrompt />)
    const gs = getGameState()
    act(() => {
      gs.showEncounterPrompt(makeCreature())
    })
    expect(screen.getByText('Engage')).toBeInTheDocument()

    act(() => {
      gs.proceedToCombat()
    })
    expect(container.innerHTML).toBe('')
  })

  it('engages with Enter key', () => {
    render(<EncounterPrompt />)
    const gs = getGameState()

    act(() => {
      gs.showEncounterPrompt(makeCreature())
    })
    fireEvent.keyDown(window, { key: 'Enter' })

    expect(gs.combatActive).toBe(true)
  })

  it('backs away with Escape key', () => {
    render(<EncounterPrompt />)
    const gs = getGameState()
    const backEvents: string[] = []
    gs.on('encounter:backed-away', (id) => backEvents.push(id as string))

    act(() => {
      gs.showEncounterPrompt(makeCreature())
    })
    fireEvent.keyDown(window, { key: 'Escape' })

    expect(backEvents.length).toBe(1)
  })
})
