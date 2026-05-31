import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import EnemyCard from '../components/EnemyCard'
import type { CreatureState } from '../types'

function makeCreature(overrides?: Partial<CreatureState>): CreatureState {
  return {
    id: 'c1',
    name: 'Goblin',
    archeType: 'creature',
    subtype: 'monster',
    difficulty: 'normal',
    level: 3,
    hp: { current: 30, max: 40 },
    attack: 8,
    defense: 5,
    xpReward: 20,
    position: { x: 0, y: 0 },
    aggroRange: 1,
    aggro: false,
    stunned: false,
    stunTimer: 0,
    ...overrides,
  }
}

describe('EnemyCard', () => {
  it('renders creature name', () => {
    const creature = makeCreature({ name: 'Rat King' })
    render(<EnemyCard creature={creature} />)
    expect(screen.getByText('Rat King')).toBeInTheDocument()
  })

  it('renders level and difficulty', () => {
    const creature = makeCreature({ level: 5, difficulty: 'elite' })
    render(<EnemyCard creature={creature} />)
    expect(screen.getByText(/LV 5/)).toBeInTheDocument()
    expect(screen.getByText(/ELITE/)).toBeInTheDocument()
  })

  it('renders HP values', () => {
    const creature = makeCreature({ hp: { current: 25, max: 40 } })
    render(<EnemyCard creature={creature} />)
    expect(screen.getByText('25/40')).toBeInTheDocument()
  })

  it('renders ATK and DEF', () => {
    const creature = makeCreature({ attack: 12, defense: 7 })
    render(<EnemyCard creature={creature} />)
    expect(screen.getByText('ATK 12')).toBeInTheDocument()
    expect(screen.getByText('DEF 7')).toBeInTheDocument()
  })
})
