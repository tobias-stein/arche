import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, fireEvent, cleanup, act } from '@testing-library/react'
import GameOverOverlay from '../components/GameOverOverlay'
import { getGameState, resetGameState } from '../GameState'

describe('GameOverOverlay', () => {
  beforeEach(() => {
    resetGameState()
  })

  afterEach(() => {
    cleanup()
  })

  it('does not render when game is active', () => {
    const { container } = render(<GameOverOverlay />)
    expect(container.innerHTML).toBe('')
  })

  it('renders when game:over event fires', () => {
    render(<GameOverOverlay />)
    const gs = getGameState()
    act(() => {
      gs.startGame()
      gs.gameOver = true
      gs.emit('game:over')
    })
    expect(screen.getByText('GAME OVER')).toBeInTheDocument()
  })

  it('shows player level and stats', () => {
    render(<GameOverOverlay />)
    const gs = getGameState()
    act(() => {
      gs.startGame()
      gs.player.level = 5
      gs.gameOver = true
      gs.emit('game:over')
    })
    expect(screen.getByText('Level Reached')).toBeInTheDocument()
    expect(screen.getByText('Rooms Explored')).toBeInTheDocument()
    expect(screen.getByText('Enemies Slain')).toBeInTheDocument()
    expect(screen.getByText('Time Survived')).toBeInTheDocument()
  })

  it('shows Play Again and Quit buttons', () => {
    render(<GameOverOverlay />)
    const gs = getGameState()
    act(() => {
      gs.startGame()
      gs.gameOver = true
      gs.emit('game:over')
    })
    expect(screen.getByText('Play Again')).toBeInTheDocument()
    expect(screen.getByText('Quit')).toBeInTheDocument()
  })

  it('calls restartGame when Play Again is clicked', () => {
    const gs = getGameState()
    const restartFn = vi.spyOn(gs, 'restartGame')
    render(<GameOverOverlay />)
    act(() => {
      gs.startGame()
      gs.gameOver = true
      gs.emit('game:over')
    })
    fireEvent.click(screen.getByText('Play Again'))
    expect(restartFn).toHaveBeenCalledTimes(1)
  })

  it('calls quitToTitle when Quit is clicked', () => {
    const gs = getGameState()
    const quitFn = vi.spyOn(gs, 'quitToTitle')
    render(<GameOverOverlay />)
    act(() => {
      gs.startGame()
      gs.gameOver = true
      gs.emit('game:over')
    })
    fireEvent.click(screen.getByText('Quit'))
    expect(quitFn).toHaveBeenCalledTimes(1)
  })

  it('disappears on game:restarted event', () => {
    const { container } = render(<GameOverOverlay />)
    const gs = getGameState()
    act(() => {
      gs.startGame()
      gs.gameOver = true
      gs.emit('game:over')
    })
    expect(screen.getByText('GAME OVER')).toBeInTheDocument()
    act(() => {
      gs.emit('game:restarted')
    })
    expect(container.innerHTML).toBe('')
  })

  it('disappears on game:quit event', () => {
    const { container } = render(<GameOverOverlay />)
    const gs = getGameState()
    act(() => {
      gs.startGame()
      gs.gameOver = true
      gs.emit('game:over')
    })
    expect(screen.getByText('GAME OVER')).toBeInTheDocument()
    act(() => {
      gs.emit('game:quit')
    })
    expect(container.innerHTML).toBe('')
  })

  it('restarts with Enter key', () => {
    const gs = getGameState()
    const restartFn = vi.spyOn(gs, 'restartGame')
    render(<GameOverOverlay />)
    act(() => {
      gs.startGame()
      gs.gameOver = true
      gs.emit('game:over')
    })
    fireEvent.keyDown(window, { key: 'Enter' })
    expect(restartFn).toHaveBeenCalledTimes(1)
  })

  it('quits with Q key', () => {
    const gs = getGameState()
    const quitFn = vi.spyOn(gs, 'quitToTitle')
    render(<GameOverOverlay />)
    act(() => {
      gs.startGame()
      gs.gameOver = true
      gs.emit('game:over')
    })
    fireEvent.keyDown(window, { key: 'q' })
    expect(quitFn).toHaveBeenCalledTimes(1)
  })
})
