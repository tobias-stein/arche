import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, fireEvent, cleanup, act } from '@testing-library/react'
import VictoryOverlay from '../components/VictoryOverlay'
import { getGameState, resetGameState } from '../GameState'

describe('VictoryOverlay', () => {
  beforeEach(() => {
    resetGameState()
  })

  afterEach(() => {
    cleanup()
  })

  it('does not render when game is active', () => {
    const { container } = render(<VictoryOverlay />)
    expect(container.innerHTML).toBe('')
  })

  it('renders YOU WIN! text on game:victory event', () => {
    render(<VictoryOverlay />)
    const gs = getGameState()
    act(() => {
      gs.startGame()
      gs.emit('game:victory')
    })
    expect(screen.getByText('YOU WIN!')).toBeInTheDocument()
  })

  it('shows stat labels', () => {
    render(<VictoryOverlay />)
    const gs = getGameState()
    act(() => {
      gs.startGame()
      gs.emit('game:victory')
    })
    expect(screen.getByText('Final Level')).toBeInTheDocument()
    expect(screen.getByText('Rooms Explored')).toBeInTheDocument()
    expect(screen.getByText('Enemies Slain')).toBeInTheDocument()
    expect(screen.getByText('Items Collected')).toBeInTheDocument()
  })

  it('shows Play Again and Quit buttons', () => {
    render(<VictoryOverlay />)
    const gs = getGameState()
    act(() => {
      gs.startGame()
      gs.emit('game:victory')
    })
    expect(screen.getByText('Play Again')).toBeInTheDocument()
    expect(screen.getByText('Quit')).toBeInTheDocument()
  })

  it('calls restartGame when Play Again is clicked', () => {
    const gs = getGameState()
    const restartFn = vi.spyOn(gs, 'restartGame')
    render(<VictoryOverlay />)
    act(() => {
      gs.startGame()
      gs.emit('game:victory')
    })
    fireEvent.click(screen.getByText('Play Again'))
    expect(restartFn).toHaveBeenCalledTimes(1)
  })

  it('calls quitToTitle when Quit is clicked', () => {
    const gs = getGameState()
    const quitFn = vi.spyOn(gs, 'quitToTitle')
    render(<VictoryOverlay />)
    act(() => {
      gs.startGame()
      gs.emit('game:victory')
    })
    fireEvent.click(screen.getByText('Quit'))
    expect(quitFn).toHaveBeenCalledTimes(1)
  })

  it('disappears on game:restarted event', () => {
    const { container } = render(<VictoryOverlay />)
    const gs = getGameState()
    act(() => {
      gs.startGame()
      gs.emit('game:victory')
    })
    expect(screen.getByText('YOU WIN!')).toBeInTheDocument()
    act(() => {
      gs.emit('game:restarted')
    })
    expect(container.innerHTML).toBe('')
  })

  it('disappears on game:quit event', () => {
    const { container } = render(<VictoryOverlay />)
    const gs = getGameState()
    act(() => {
      gs.startGame()
      gs.emit('game:victory')
    })
    expect(screen.getByText('YOU WIN!')).toBeInTheDocument()
    act(() => {
      gs.emit('game:quit')
    })
    expect(container.innerHTML).toBe('')
  })

  it('restarts with Enter key', () => {
    const gs = getGameState()
    const restartFn = vi.spyOn(gs, 'restartGame')
    render(<VictoryOverlay />)
    act(() => {
      gs.startGame()
      gs.emit('game:victory')
    })
    fireEvent.keyDown(window, { key: 'Enter' })
    expect(restartFn).toHaveBeenCalledTimes(1)
  })

  it('quits with Q key', () => {
    const gs = getGameState()
    const quitFn = vi.spyOn(gs, 'quitToTitle')
    render(<VictoryOverlay />)
    act(() => {
      gs.startGame()
      gs.emit('game:victory')
    })
    fireEvent.keyDown(window, { key: 'q' })
    expect(quitFn).toHaveBeenCalledTimes(1)
  })
})
