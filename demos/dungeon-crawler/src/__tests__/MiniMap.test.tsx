import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import { render, screen, fireEvent, cleanup, act } from '@testing-library/react'
import MiniMap from '../components/MiniMap'
import { getGameState, resetGameState } from '../GameState'
import type { Dungeon } from '../game/dungeon-generator'

beforeEach(() => {
  resetGameState()
})

afterEach(() => {
  cleanup()
})

describe('MiniMap', () => {
  it('renders small minimap canvas', () => {
    render(<MiniMap />)
    const canvas = screen.getByTestId('mini-canvas-small')
    expect(canvas).toBeInTheDocument()
    expect(canvas.tagName).toBe('CANVAS')
  })

  it('small minimap is visible by default', () => {
    render(<MiniMap />)
    const container = screen.getByTestId('minimap-small')
    expect(container.classList.contains('hidden')).toBe(false)
  })

  it('opens overlay when small minimap is clicked', () => {
    render(<MiniMap />)
    const container = screen.getByTestId('minimap-small')
    fireEvent.click(container)
    const overlay = screen.getByTestId('minimap-overlay')
    expect(overlay).toBeInTheDocument()
    expect(container.classList.contains('hidden')).toBe(true)
  })

  it('hides small minimap when overlay opens', () => {
    render(<MiniMap />)
    fireEvent.click(screen.getByTestId('minimap-small'))
    expect(screen.getByTestId('minimap-small').classList.contains('hidden')).toBe(true)
  })

  it('closes overlay when close button is clicked', () => {
    render(<MiniMap />)
    fireEvent.click(screen.getByTestId('minimap-small'))
    expect(screen.getByTestId('minimap-overlay')).toBeInTheDocument()
    fireEvent.click(screen.getByLabelText('Close minimap'))
    expect(screen.queryByTestId('minimap-overlay')).toBeNull()
  })

  it('toggles overlay with M key', () => {
    render(<MiniMap />)
    act(() => { fireEvent.keyDown(window, { key: 'm' }) })
    expect(screen.getByTestId('minimap-overlay')).toBeInTheDocument()
    act(() => { fireEvent.keyDown(window, { key: 'm' }) })
    expect(screen.queryByTestId('minimap-overlay')).toBeNull()
  })

  it('toggles overlay with uppercase M key', () => {
    render(<MiniMap />)
    act(() => { fireEvent.keyDown(window, { key: 'M' }) })
    expect(screen.getByTestId('minimap-overlay')).toBeInTheDocument()
  })

  it('shows large canvas in overlay', () => {
    render(<MiniMap />)
    fireEvent.click(screen.getByTestId('minimap-small'))
    const largeCanvas = screen.getByTestId('mini-canvas-large')
    expect(largeCanvas).toBeInTheDocument()
    expect(largeCanvas.tagName).toBe('CANVAS')
  })

  it('shows zoom controls in overlay', () => {
    render(<MiniMap />)
    fireEvent.click(screen.getByTestId('minimap-small'))
    const slider = screen.getByLabelText('Zoom')
    expect(slider).toBeInTheDocument()
    expect(slider).toHaveAttribute('type', 'range')
  })

  it('shows zoom level text', () => {
    render(<MiniMap />)
    fireEvent.click(screen.getByTestId('minimap-small'))
    const zoomLevel = document.querySelector('.level')
    expect(zoomLevel).toBeInTheDocument()
  })

  it('renders without dungeon data (no crash)', () => {
    render(<MiniMap />)
    expect(screen.getByTestId('minimap-small')).toBeInTheDocument()
  })

  it('does not crash when dungeon:ready fires', () => {
    const gs = getGameState()
    render(<MiniMap />)
    act(() => {
      gs.emit('dungeon:ready', { rooms: [{ x: 1, y: 1, id: 0 }], grid: [[0]], bossRoom: -1, entranceRoom: 0, treeEdges: [], extraEdges: [] } as unknown as Dungeon)
    })
    expect(screen.getByTestId('minimap-small')).toBeInTheDocument()
  })

  it('does not crash when room:changed fires', () => {
    const gs = getGameState()
    render(<MiniMap />)
    act(() => {
      gs.emit('dungeon:ready', { rooms: [{ x: 1, y: 1, id: 0 }], grid: [[0]], bossRoom: -1, entranceRoom: 0, treeEdges: [], extraEdges: [] } as unknown as Dungeon)
    })
    act(() => {
      gs.emit('room:changed', 0)
    })
    expect(screen.getByTestId('minimap-small')).toBeInTheDocument()
  })

  it('close overlay returns small minimap to visible', () => {
    render(<MiniMap />)
    fireEvent.click(screen.getByTestId('minimap-small'))
    expect(screen.queryByTestId('minimap-small')).not.toBeNull()
    fireEvent.click(screen.getByLabelText('Close minimap'))
    const small = screen.getByTestId('minimap-small')
    expect(small.classList.contains('hidden')).toBe(false)
  })
})
