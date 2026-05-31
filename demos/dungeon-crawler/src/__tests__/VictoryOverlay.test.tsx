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

  it('calls onPlayAgain when Play Again is clicked', () => {
    const onPlayAgain = vi.fn()
    render(<VictoryOverlay onPlayAgain={onPlayAgain} />)
    const gs = getGameState()
    act(() => {
      gs.startGame()
      gs.emit('game:victory')
    })
    fireEvent.click(screen.getByText('Play Again'))
    expect(onPlayAgain).toHaveBeenCalledTimes(1)
  })

  it('calls onQuit when Quit is clicked', () => {
    const onQuit = vi.fn()
    render(<VictoryOverlay onQuit={onQuit} />)
    const gs = getGameState()
    act(() => {
      gs.startGame()
      gs.emit('game:victory')
    })
    fireEvent.click(screen.getByText('Quit'))
    expect(onQuit).toHaveBeenCalledTimes(1)
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
    const onPlayAgain = vi.fn()
    render(<VictoryOverlay onPlayAgain={onPlayAgain} />)
    const gs = getGameState()
    act(() => {
      gs.startGame()
      gs.emit('game:victory')
    })
    fireEvent.keyDown(window, { key: 'Enter' })
    expect(onPlayAgain).toHaveBeenCalledTimes(1)
  })

  it('quits with Q key', () => {
    const onQuit = vi.fn()
    render(<VictoryOverlay onQuit={onQuit} />)
    const gs = getGameState()
    act(() => {
      gs.startGame()
      gs.emit('game:victory')
    })
    fireEvent.keyDown(window, { key: 'q' })
    expect(onQuit).toHaveBeenCalledTimes(1)
  })
})
