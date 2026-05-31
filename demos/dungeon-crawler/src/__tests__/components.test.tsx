import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { render, screen, fireEvent, cleanup, waitFor, act } from '@testing-library/react'
import TitleScreen from '../components/TitleScreen'
import PlayerCard from '../components/PlayerCard'
import { getGameState, resetGameState } from '../GameState'

beforeEach(() => {
  resetGameState()
})

afterEach(() => {
  cleanup()
})

describe('TitleScreen', () => {
  it('renders game title and prompt', () => {
    render(<TitleScreen onStart={() => {}} />)
    expect(screen.getByText('Pixel Quest')).toBeInTheDocument()
    expect(screen.getByText(/PRESS ENTER TO START/i)).toBeInTheDocument()
  })

  it('renders subtitle', () => {
    render(<TitleScreen onStart={() => {}} />)
    expect(screen.getByText('A Retro Roguelike Adventure')).toBeInTheDocument()
  })

  it('calls onStart when Enter is pressed', async () => {
    vi.useFakeTimers()
    const onStart = vi.fn()
    render(<TitleScreen onStart={onStart} />)
    act(() => { fireEvent.keyDown(window, { key: 'Enter' }) })
    act(() => { vi.advanceTimersByTime(400) })
    expect(onStart).toHaveBeenCalledTimes(1)
    vi.useRealTimers()
  })

  it('calls onStart when clicked', async () => {
    vi.useFakeTimers()
    const onStart = vi.fn()
    render(<TitleScreen onStart={onStart} />)
    act(() => { fireEvent.click(screen.getByText('Pixel Quest')) })
    act(() => { vi.advanceTimersByTime(400) })
    expect(onStart).toHaveBeenCalledTimes(1)
    vi.useRealTimers()
  })
})

describe('PlayerCard', () => {
  it('does not render before game starts', () => {
    const { container } = render(<PlayerCard />)
    expect(container.innerHTML).toBe('')
  })

  it('renders after game starts', async () => {
    const gs = getGameState()
    render(<PlayerCard />)
    act(() => { gs.startGame() })
    await waitFor(() => expect(screen.getByText('Hero')).toBeInTheDocument())
    expect(screen.getByText('LV 1')).toBeInTheDocument()
  })

  it('displays HP from game state', async () => {
    const gs = getGameState()
    render(<PlayerCard />)
    act(() => { gs.startGame() })
    await waitFor(() => expect(screen.getByText(/^100\/100$/)).toBeInTheDocument())
  })

  it('displays MP from game state', async () => {
    const gs = getGameState()
    render(<PlayerCard />)
    act(() => { gs.startGame() })
    await waitFor(() => expect(screen.getByText(/^30\/30$/)).toBeInTheDocument())
  })

  it('displays ATK and DEF from config', async () => {
    const gs = getGameState()
    render(<PlayerCard />)
    act(() => { gs.startGame() })
    await waitFor(() => {
      expect(screen.getByText('ATK 10')).toBeInTheDocument()
      expect(screen.getByText('DEF 5')).toBeInTheDocument()
    })
  })

  it('displays XP', async () => {
    const gs = getGameState()
    render(<PlayerCard />)
    act(() => { gs.startGame() })
    await waitFor(() => expect(screen.getByText(/XP 0\/10/)).toBeInTheDocument())
  })

  it('emits inventory:requested on click', async () => {
    const gs = getGameState()
    const events: string[] = []
    gs.on('inventory:requested', () => events.push('inventory:requested'))
    render(<PlayerCard />)
    act(() => { gs.startGame() })
    await waitFor(() => screen.getByText('Hero'))
    fireEvent.click(screen.getByText('Hero'))
    expect(events).toEqual(['inventory:requested'])
  })

  it('updates HP bar when game state changes', async () => {
    const gs = getGameState()
    render(<PlayerCard />)
    act(() => { gs.startGame() })
    await waitFor(() => expect(screen.getByText(/^100\/100$/)).toBeInTheDocument())
    act(() => { gs.setPlayerHp(50) })
    await waitFor(() => expect(screen.getByText(/^50\/100$/)).toBeInTheDocument())
  })

  it('updates MP bar when game state changes', async () => {
    const gs = getGameState()
    render(<PlayerCard />)
    act(() => { gs.startGame() })
    await waitFor(() => expect(screen.getByText(/^30\/30$/)).toBeInTheDocument())
    act(() => { gs.setPlayerMp(15) })
    await waitFor(() => expect(screen.getByText(/^15\/30$/)).toBeInTheDocument())
  })
})
