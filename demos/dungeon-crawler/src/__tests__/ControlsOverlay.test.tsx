import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, fireEvent, cleanup, act } from '@testing-library/react'
import ControlsOverlay from '../components/ControlsOverlay'
import { getGameState, resetGameState } from '../GameState'

describe('ControlsOverlay', () => {
  beforeEach(() => {
    resetGameState()
  })

  afterEach(() => {
    cleanup()
  })

  it('does not render when controls are hidden', () => {
    const { container } = render(<ControlsOverlay />)
    expect(container.innerHTML).toBe('')
  })

  it('renders when controls are toggled on', () => {
    render(<ControlsOverlay />)
    const gs = getGameState()
    act(() => {
      gs.setControlsVisible(true)
    })
    expect(screen.getByText('CONTROLS')).toBeInTheDocument()
  })

  it('renders key binding labels', () => {
    render(<ControlsOverlay />)
    const gs = getGameState()
    act(() => {
      gs.setControlsVisible(true)
    })
    const moveLabels = screen.getAllByText('Move')
    expect(moveLabels.length).toBe(2)
    expect(screen.getByText('Equipment')).toBeInTheDocument()
    expect(screen.getByText('Map')).toBeInTheDocument()
    expect(screen.getByText('Help / Controls')).toBeInTheDocument()
  })

  it('hides when close button is clicked', () => {
    const { container } = render(<ControlsOverlay />)
    const gs = getGameState()
    act(() => {
      gs.setControlsVisible(true)
    })
    expect(screen.getByText('CONTROLS')).toBeInTheDocument()
    fireEvent.click(screen.getByLabelText('Close controls'))
    expect(container.innerHTML).toBe('')
  })

  it('hides when controlsVisible is set to false', () => {
    const { container } = render(<ControlsOverlay />)
    const gs = getGameState()
    act(() => {
      gs.setControlsVisible(true)
    })
    expect(screen.getByText('CONTROLS')).toBeInTheDocument()
    act(() => {
      gs.setControlsVisible(false)
    })
    expect(container.innerHTML).toBe('')
  })

  it('closes when overlay background is clicked', () => {
    const { container } = render(<ControlsOverlay />)
    const gs = getGameState()
    act(() => {
      gs.setControlsVisible(true)
    })
    expect(screen.getByText('CONTROLS')).toBeInTheDocument()
    fireEvent.click(screen.getByTestId('controls-overlay'))
    expect(container.innerHTML).toBe('')
  })

  it('renders WASD and Arrow key bindings', () => {
    render(<ControlsOverlay />)
    const gs = getGameState()
    act(() => {
      gs.setControlsVisible(true)
    })
    expect(screen.getByText('W')).toBeInTheDocument()
    expect(screen.getByText('A')).toBeInTheDocument()
    expect(screen.getByText('S')).toBeInTheDocument()
    expect(screen.getByText('D')).toBeInTheDocument()
    expect(screen.getByText('Arrows')).toBeInTheDocument()
  })
})
