import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, fireEvent, cleanup } from '@testing-library/react'
import ActionMenu from '../components/ActionMenu'

describe('ActionMenu', () => {
  const onAttack = vi.fn()
  const onCastSpell = vi.fn()
  const onUseItem = vi.fn()
  const onFlee = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    cleanup()
  })

  it('renders all four action buttons', () => {
    render(<ActionMenu onAttack={onAttack} onCastSpell={onCastSpell} onUseItem={onUseItem} onFlee={onFlee} />)
    expect(screen.getByText('Attack')).toBeInTheDocument()
    expect(screen.getByText('Cast Spell')).toBeInTheDocument()
    expect(screen.getByText('Use Item')).toBeInTheDocument()
    expect(screen.getByText('Flee')).toBeInTheDocument()
  })

  it('calls onAttack when Attack button is clicked', () => {
    render(<ActionMenu onAttack={onAttack} onCastSpell={onCastSpell} onUseItem={onUseItem} onFlee={onFlee} />)
    fireEvent.click(screen.getByText('Attack'))
    expect(onAttack).toHaveBeenCalledTimes(1)
  })

  it('calls onFlee when Flee button is clicked', () => {
    render(<ActionMenu onAttack={onAttack} onCastSpell={onCastSpell} onUseItem={onUseItem} onFlee={onFlee} />)
    fireEvent.click(screen.getByText('Flee'))
    expect(onFlee).toHaveBeenCalledTimes(1)
  })

  it('does not call handlers when disabled', () => {
    render(<ActionMenu onAttack={onAttack} onCastSpell={onCastSpell} onUseItem={onUseItem} onFlee={onFlee} disabled />)
    fireEvent.click(screen.getByText('Attack'))
    expect(onAttack).not.toHaveBeenCalled()
  })

  it('navigates down then right then enter on Flee', () => {
    render(<ActionMenu onAttack={onAttack} onCastSpell={onCastSpell} onUseItem={onUseItem} onFlee={onFlee} />)

    fireEvent.keyDown(window, { key: 'ArrowDown' })
    fireEvent.keyDown(window, { key: 'ArrowRight' })
    fireEvent.keyDown(window, { key: 'Enter' })
    expect(onFlee).toHaveBeenCalledTimes(1)
  })

  it('navigates right then left', () => {
    render(<ActionMenu onAttack={onAttack} onCastSpell={onCastSpell} onUseItem={onUseItem} onFlee={onFlee} />)

    fireEvent.keyDown(window, { key: 'ArrowRight' })
    fireEvent.keyDown(window, { key: 'Enter' })
    expect(onCastSpell).toHaveBeenCalledTimes(1)

    fireEvent.keyDown(window, { key: 'ArrowLeft' })
    fireEvent.keyDown(window, { key: 'Enter' })
    expect(onAttack).toHaveBeenCalledTimes(1)
  })

  it('navigates down then up back to first row', () => {
    render(<ActionMenu onAttack={onAttack} onCastSpell={onCastSpell} onUseItem={onUseItem} onFlee={onFlee} />)

    fireEvent.keyDown(window, { key: 'ArrowDown' })
    fireEvent.keyDown(window, { key: 'Enter' })
    expect(onUseItem).toHaveBeenCalledTimes(1)

    fireEvent.keyDown(window, { key: 'ArrowUp' })
    fireEvent.keyDown(window, { key: 'Enter' })
    expect(onAttack).toHaveBeenCalledTimes(1)
  })
})
