import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import { render, screen, fireEvent, cleanup, waitFor, act } from '@testing-library/react'
import ActivityLog from '../components/ActivityLog'
import { getGameState, resetGameState } from '../GameState'
import type { LogEvent } from '../types'

beforeEach(() => {
  resetGameState()
})

afterEach(() => {
  cleanup()
})

describe('ActivityLog', () => {
  it('renders collapsed by default', () => {
    render(<ActivityLog />)
    expect(screen.getByText('Activity Log')).toBeInTheDocument()
    const log = screen.getByTestId('activity-log')
    expect(log.classList.contains('expanded')).toBe(false)
  })

  it('shows latest entry preview when collapsed', () => {
    const gs = getGameState()
    render(<ActivityLog />)
    act(() => {
      gs.addLogEntry({
        type: 'player_damage_dealt',
        message: 'You hit Goblin for 12 damage.',
        icon: 'fa-crosshairs',
      })
    })
    expect(screen.getByText(/You hit Goblin for 12 damage/)).toBeInTheDocument()
  })

  it('expands on header click', () => {
    render(<ActivityLog />)
    const header = screen.getByText('Activity Log')
    fireEvent.click(header)
    const log = screen.getByTestId('activity-log')
    expect(log.classList.contains('expanded')).toBe(true)
  })

  it('collapses on second header click', () => {
    render(<ActivityLog />)
    const header = screen.getByText('Activity Log')
    fireEvent.click(header)
    fireEvent.click(header)
    const log = screen.getByTestId('activity-log')
    expect(log.classList.contains('expanded')).toBe(false)
  })

  it('toggles on [L] key press', () => {
    render(<ActivityLog />)
    act(() => { fireEvent.keyDown(window, { key: 'l' }) })
    const log = screen.getByTestId('activity-log')
    expect(log.classList.contains('expanded')).toBe(true)
    act(() => { fireEvent.keyDown(window, { key: 'l' }) })
    expect(log.classList.contains('expanded')).toBe(false)
  })

  it('handles uppercase L key', () => {
    render(<ActivityLog />)
    act(() => { fireEvent.keyDown(window, { key: 'L' }) })
    const log = screen.getByTestId('activity-log')
    expect(log.classList.contains('expanded')).toBe(true)
  })

  it('displays multiple log entries', () => {
    const gs = getGameState()
    render(<ActivityLog />)
    act(() => {
      gs.addLogEntry({ type: 'player_damage_dealt', message: 'Hit for 10.', icon: 'fa-crosshairs' })
      gs.addLogEntry({ type: 'enemy_slain', message: 'Goblin slain!', icon: 'fa-skull' })
    })
    expect(screen.getByText(/Hit for 10/)).toBeInTheDocument()
    expect(screen.getByText(/Goblin slain/)).toBeInTheDocument()
  })

  it('shows entries with correct event data', () => {
    const events: LogEvent[] = [
      { type: 'player_damage_dealt', message: 'Dealt 12 damage.', icon: 'fa-crosshairs' },
      { type: 'player_damage_taken', message: 'Took 5 damage.', icon: 'fa-shield-hacked' },
      { type: 'enemy_slain', message: 'Goblin slain!', icon: 'fa-skull' },
      { type: 'spell_cast', message: 'Fire Bolt cast!', icon: 'fa-wand-sparkles' },
      { type: 'potion_consumed', message: 'Health Potion used.', icon: 'fa-flask' },
      { type: 'item_picked_up', message: 'Iron Sword picked up.', icon: 'fa-box-open' },
      { type: 'item_dropped', message: 'Wood Shield dropped.', icon: 'fa-arrow-up-from-bracket' },
      { type: 'room_entered', message: 'Entered Dark Hallway.', icon: 'fa-arrow-right-to-bracket' },
      { type: 'encounter_started', message: 'Ambush!', icon: 'fa-triangle-exclamation' },
      { type: 'level_up', message: 'Level up!', icon: 'fa-star' },
      { type: 'flee_attempt', message: 'You fled!', icon: 'fa-person-running' },
    ]

    const gs = getGameState()
    const { container } = render(<ActivityLog />)
    act(() => {
      for (const e of events) {
        gs.addLogEntry(e)
      }
    })

    fireEvent.click(screen.getByText('Activity Log'))

    const entries = container.querySelectorAll('.le')
    expect(entries.length).toBe(events.length)
    entries.forEach((el, idx) => {
      const text = el.textContent || ''
      // entries are displayed newest-first, so match from end
      const eventIdx = events.length - 1 - idx
      expect(text).toContain(events[eventIdx].message)
    })
  })

  it('expands to show all entries', () => {
    const gs = getGameState()
    const { container } = render(<ActivityLog />)
    act(() => {
      for (let i = 1; i <= 10; i++) {
        gs.addLogEntry({ type: 'player_damage_dealt', message: `Entry ${i}`, icon: 'fa-crosshairs' })
      }
    })

    fireEvent.click(screen.getByText('Activity Log'))

    const entries = container.querySelectorAll('.le')
    expect(entries.length).toBe(10)
    entries.forEach((el, idx) => {
      // entries are newest-first: Entry 10 at idx 0, Entry 1 at idx 9
      const expected = 10 - idx
      expect(el.textContent).toContain(`Entry ${expected}`)
    })
  })

  it('limits entries in collapsed preview to 5', () => {
    const gs = getGameState()
    render(<ActivityLog />)
    act(() => {
      for (let i = 0; i < 10; i++) {
        gs.addLogEntry({ type: 'player_damage_dealt', message: `Msg ${i}`, icon: 'fa-crosshairs' })
      }
    })

    const logBody = screen.getByTestId('log-body')
    const entryElements = logBody.querySelectorAll('.le')
    expect(entryElements.length).toBeLessThanOrEqual(5)
  })

  it('rolls off entries beyond MAX_LOG', async () => {
    const gs = getGameState()
    const { container } = render(<ActivityLog />)

    act(() => {
      for (let i = 0; i < 250; i++) {
        gs.addLogEntry({ type: 'player_damage_dealt', message: `Entry ${i}`, icon: 'fa-crosshairs' })
      }
    })

    fireEvent.click(screen.getByText('Activity Log'))

    await waitFor(() => {
      const entries = container.querySelectorAll('.le')
      expect(entries.length).toBeLessThanOrEqual(200)

      const texts = Array.from(entries).map(el => el.textContent || '')
      expect(texts.some(t => t.includes('Entry 0'))).toBe(false)
      expect(texts.some(t => t.includes('Entry 249'))).toBe(true)
    })
  })
})
