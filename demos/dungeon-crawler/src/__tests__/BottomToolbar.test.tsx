import { describe, it, expect, vi, afterEach, beforeEach } from 'vitest'
import { render, screen, fireEvent, cleanup, act } from '@testing-library/react'
import BottomToolbar from '../components/BottomToolbar'
import { getGameState, resetGameState } from '../GameState'

beforeEach(() => {
  resetGameState()
})

afterEach(() => {
  cleanup()
})

describe('BottomToolbar', () => {
  it('renders four buttons', () => {
    render(<BottomToolbar onToggleLog={() => {}} />)
    expect(screen.getByLabelText('Inventory')).toBeInTheDocument()
    expect(screen.getByLabelText('Map')).toBeInTheDocument()
    expect(screen.getByLabelText('Log')).toBeInTheDocument()
    expect(screen.getByLabelText('Help')).toBeInTheDocument()
  })

  it('emits inventory:requested on Inventory tap', () => {
    const gs = getGameState()
    const events: string[] = []
    gs.on('inventory:requested', () => events.push('inventory:requested'))
    render(<BottomToolbar onToggleLog={() => {}} />)

    fireEvent.click(screen.getByLabelText('Inventory'))
    expect(events).toEqual(['inventory:requested'])
  })

  it('calls onToggleLog when Log button is tapped', () => {
    const onToggleLog = vi.fn()
    render(<BottomToolbar onToggleLog={onToggleLog} />)

    fireEvent.click(screen.getByLabelText('Log'))
    expect(onToggleLog).toHaveBeenCalledTimes(1)
  })

  it('dispatches M keydown for Map button', () => {
    const events: string[] = []
    window.addEventListener('keydown', (e) => events.push(e.key))
    render(<BottomToolbar onToggleLog={() => {}} />)

    fireEvent.click(screen.getByLabelText('Map'))
    expect(events).toContain('M')
  })

  it('dispatches ? keydown for Help button', () => {
    const events: string[] = []
    window.addEventListener('keydown', (e) => events.push(e.key))
    render(<BottomToolbar onToggleLog={() => {}} />)

    fireEvent.click(screen.getByLabelText('Help'))
    expect(events).toContain('?')
  })
})
