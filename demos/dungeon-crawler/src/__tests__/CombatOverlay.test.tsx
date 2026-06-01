import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, fireEvent, cleanup, act } from '@testing-library/react'
import CombatOverlay from '../components/CombatOverlay'
import { getGameState, resetGameState } from '../GameState'
import type { CreatureState, ItemState } from '../types'
import SpellPanel from '../components/SpellPanel'
import UseItemPanel from '../components/UseItemPanel'
import FleePanel from '../components/FleePanel'

function makeCreature(overrides?: Partial<CreatureState>): CreatureState {
  return {
    id: 'c1',
    name: 'Goblin Scout',
    archeType: 'creature',
    subtype: 'monster',
    difficulty: 'normal',
    level: 3,
    hp: { current: 30, max: 40 },
    attack: 8,
    defense: 5,
    xpReward: 20,
    position: { x: 5, y: 3 },
    aggroRange: 1,
    aggro: false,
    stunned: false,
    stunTimer: 0,
    ...overrides,
  }
}

function makeSpell(overrides?: Partial<ItemState>): ItemState {
  return {
    id: 'spell_1',
    name: 'Fire Bolt',
    archeType: 'spell',
    rarity: 'legendary',
    level: 1,
    stats: { damage: 12, mana_cost: 5 },
    affixes: [],
    ...overrides,
  }
}

function makeConsumable(overrides?: Partial<ItemState>): ItemState {
  return {
    id: 'item_1',
    name: 'Health Potion',
    archeType: 'consumable',
    subtype: 'potion',
    rarity: 'common',
    level: 1,
    stats: { heal: 25 },
    affixes: [],
    ...overrides,
  }
}

function makeEquipment(overrides?: Partial<ItemState>): ItemState {
  return {
    id: 'eq_1',
    name: 'Iron Sword',
    archeType: 'weapon',
    rarity: 'common',
    level: 1,
    equipSlot: 'weapon',
    stats: { attack: 5 },
    affixes: [],
    ...overrides,
  }
}

describe('CombatOverlay', () => {
  beforeEach(() => {
    resetGameState()
    vi.useFakeTimers()
  })

  afterEach(() => {
    cleanup()
    vi.useRealTimers()
  })

  function startCombat() {
    const gs = getGameState()
    act(() => {
      gs.startCombat(makeCreature())
    })
    act(() => {
      vi.advanceTimersByTime(500)
    })
  }

  it('does not render when no combat is active', () => {
    const { container } = render(<CombatOverlay />)
    expect(container.innerHTML).toBe('')
  })

  it('renders the action menu during player turn', () => {
    render(<CombatOverlay />)
    startCombat()
    expect(screen.getByText('Flee')).toBeInTheDocument()
    expect(screen.getByText('Attack')).toBeInTheDocument()
  })

  it('shows flee confirmation dialog when Flee is clicked', () => {
    render(<CombatOverlay />)
    startCombat()
    fireEvent.click(screen.getByText('Flee'))
    expect(screen.getByText('Flee from combat?')).toBeInTheDocument()
    expect(screen.getByText('Cowardice has its rewards.')).toBeInTheDocument()
    expect(screen.getByText('Yes, Flee')).toBeInTheDocument()
    expect(screen.getByText('Stay & Fight')).toBeInTheDocument()
  })

  it('executes flee when Yes, Flee is clicked', () => {
    render(<CombatOverlay />)
    startCombat()
    const gs = getGameState()
    fireEvent.click(screen.getByText('Flee'))
    fireEvent.click(screen.getByText('Yes, Flee'))
    expect(gs.combatActive).toBe(false)
  })

  it('returns to action menu when Stay & Fight is clicked', () => {
    render(<CombatOverlay />)
    startCombat()
    fireEvent.click(screen.getByText('Flee'))
    expect(screen.getByText('Flee from combat?')).toBeInTheDocument()
    fireEvent.click(screen.getByText('Stay & Fight'))
    expect(screen.queryByText('Flee from combat?')).not.toBeInTheDocument()
    expect(screen.getByText('Flee')).toBeInTheDocument()
  })

  it('closes flee dialog with Escape key', () => {
    render(<CombatOverlay />)
    startCombat()
    fireEvent.click(screen.getByText('Flee'))
    expect(screen.getByText('Flee from combat?')).toBeInTheDocument()
    fireEvent.keyDown(window, { key: 'Escape' })
    expect(screen.queryByText('Flee from combat?')).not.toBeInTheDocument()
  })

  it('confirms flee with Enter key on Yes, Flee', () => {
    render(<CombatOverlay />)
    startCombat()
    const gs = getGameState()
    fireEvent.click(screen.getByText('Flee'))
    expect(screen.getByText('Flee from combat?')).toBeInTheDocument()
    fireEvent.keyDown(window, { key: 'Enter' })
    expect(gs.combatActive).toBe(false)
  })

  it('does not show flee dialog after combat ends', () => {
    render(<CombatOverlay />)
    startCombat()
    const gs = getGameState()
    fireEvent.click(screen.getByText('Flee'))
    expect(screen.getByText('Flee from combat?')).toBeInTheDocument()
    act(() => {
      gs.endCombat()
    })
    expect(screen.queryByText('Flee from combat?')).not.toBeInTheDocument()
  })
})

describe('SpellPanel', () => {
  const onSelect = vi.fn()
  const onClose = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    cleanup()
  })

  it('renders list of known spells', () => {
    const spells = [makeSpell({ name: 'Fire Bolt' }), makeSpell({ name: 'Ice Shard', id: 'spell_2' })]
    render(<SpellPanel spells={spells} playerMp={20} onSelect={onSelect} onClose={onClose} />)
    expect(screen.getByText('Fire Bolt')).toBeInTheDocument()
    expect(screen.getByText('Ice Shard')).toBeInTheDocument()
    expect(screen.getByText('Select Spell')).toBeInTheDocument()
  })

  it('shows mana cost for each spell', () => {
    const spells = [makeSpell({ stats: { damage: 12, mana_cost: 5 } })]
    render(<SpellPanel spells={spells} playerMp={20} onSelect={onSelect} onClose={onClose} />)
    expect(screen.getByText('5 MP')).toBeInTheDocument()
  })

  it('greys out spells with insufficient mana', () => {
    const spells = [makeSpell({ stats: { damage: 12, mana_cost: 10 } })]
    render(<SpellPanel spells={spells} playerMp={5} onSelect={onSelect} onClose={onClose} />)
    const item = screen.getByText('10 MP').closest('.cp-item')
    expect(item?.classList.contains('disabled')).toBe(true)
  })

  it('does not call onSelect for disabled spell on click', () => {
    const spells = [makeSpell({ stats: { damage: 12, mana_cost: 10 } })]
    render(<SpellPanel spells={spells} playerMp={5} onSelect={onSelect} onClose={onClose} />)
    fireEvent.click(screen.getByText('10 MP'))
    expect(onSelect).not.toHaveBeenCalled()
  })

  it('calls onSelect when clicking a spell', () => {
    const spells = [makeSpell({ name: 'Fire Bolt', stats: { damage: 12, mana_cost: 5 } })]
    render(<SpellPanel spells={spells} playerMp={20} onSelect={onSelect} onClose={onClose} />)
    fireEvent.click(screen.getByText('Fire Bolt'))
    expect(onSelect).toHaveBeenCalledWith(spells[0])
  })

  it('shows empty state when no spells known', () => {
    render(<SpellPanel spells={[null, null]} playerMp={20} onSelect={onSelect} onClose={onClose} />)
    expect(screen.getByText('No spells known.')).toBeInTheDocument()
  })

  it('calls onClose on Escape key', () => {
    const spells = [makeSpell()]
    render(<SpellPanel spells={spells} playerMp={20} onSelect={onSelect} onClose={onClose} />)
    fireEvent.keyDown(window, { key: 'Escape' })
    expect(onClose).toHaveBeenCalledTimes(1)
  })

  it('selects spell on Enter key', () => {
    const spells = [makeSpell()]
    render(<SpellPanel spells={spells} playerMp={20} onSelect={onSelect} onClose={onClose} />)
    fireEvent.keyDown(window, { key: 'Enter' })
    expect(onSelect).toHaveBeenCalledWith(spells[0])
  })

  it('navigates with arrow keys', () => {
    const spells = [makeSpell({ name: 'Spell A', id: 'a' }), makeSpell({ name: 'Spell B', id: 'b' })]
    render(<SpellPanel spells={spells} playerMp={20} onSelect={onSelect} onClose={onClose} />)
    fireEvent.keyDown(window, { key: 'ArrowDown' })
    fireEvent.keyDown(window, { key: 'Enter' })
    expect(onSelect).toHaveBeenCalledWith(spells[1])

    fireEvent.keyDown(window, { key: 'ArrowUp' })
    fireEvent.keyDown(window, { key: 'Enter' })
    expect(onSelect).toHaveBeenCalledWith(spells[0])
  })

  it('does not select disabled spell on Enter', () => {
    const spells = [makeSpell({ stats: { damage: 12, mana_cost: 10 } })]
    render(<SpellPanel spells={spells} playerMp={5} onSelect={onSelect} onClose={onClose} />)
    fireEvent.keyDown(window, { key: 'Enter' })
    expect(onSelect).not.toHaveBeenCalled()
  })

  it('calls onClose when clicking backdrop', () => {
    const spells = [makeSpell()]
    const { container } = render(<SpellPanel spells={spells} playerMp={20} onSelect={onSelect} onClose={onClose} />)
    fireEvent.click(container.querySelector('.combat-panel-overlay')!)
    expect(onClose).toHaveBeenCalledTimes(1)
  })

  it('does not call onClose when clicking panel interior', () => {
    const spells = [makeSpell()]
    render(<SpellPanel spells={spells} playerMp={20} onSelect={onSelect} onClose={onClose} />)
    fireEvent.click(screen.getByText('Select Spell'))
    expect(onClose).not.toHaveBeenCalled()
  })

  it('calls onClose when clicking close button', () => {
    const spells = [makeSpell()]
    render(<SpellPanel spells={spells} playerMp={20} onSelect={onSelect} onClose={onClose} />)
    fireEvent.click(screen.getByLabelText('Close'))
    expect(onClose).toHaveBeenCalledTimes(1)
  })
})

describe('UseItemPanel', () => {
  const onSelect = vi.fn()
  const onClose = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    cleanup()
  })

  it('renders consumable items from inventory', () => {
    const inventory = [makeConsumable({ name: 'Health Potion' }), makeConsumable({ name: 'Mana Potion', id: 'item_2', stats: { mana: 15 } })]
    render(<UseItemPanel inventory={inventory} onSelect={onSelect} onClose={onClose} />)
    expect(screen.getByText('Health Potion')).toBeInTheDocument()
    expect(screen.getByText('Mana Potion')).toBeInTheDocument()
    expect(screen.getByText('Use Item')).toBeInTheDocument()
  })

  it('filters out non-consumable items', () => {
    const inventory = [makeConsumable(), makeEquipment()]
    render(<UseItemPanel inventory={inventory} onSelect={onSelect} onClose={onClose} />)
    expect(screen.getByText('Health Potion')).toBeInTheDocument()
    expect(screen.queryByText('Iron Sword')).not.toBeInTheDocument()
  })

  it('shows effect value for healing items', () => {
    const inventory = [makeConsumable({ stats: { heal: 25 } })]
    render(<UseItemPanel inventory={inventory} onSelect={onSelect} onClose={onClose} />)
    expect(screen.getByText('+25 HP')).toBeInTheDocument()
  })

  it('shows effect value for mana items', () => {
    const inventory = [makeConsumable({ stats: { mana: 15 } })]
    render(<UseItemPanel inventory={inventory} onSelect={onSelect} onClose={onClose} />)
    expect(screen.getByText('+15 MP')).toBeInTheDocument()
  })

  it('shows empty state when no consumables', () => {
    const inventory = [null, null, makeEquipment()]
    render(<UseItemPanel inventory={inventory} onSelect={onSelect} onClose={onClose} />)
    expect(screen.getByText('No usable items.')).toBeInTheDocument()
  })

  it('shows empty state when inventory is empty', () => {
    render(<UseItemPanel inventory={[]} onSelect={onSelect} onClose={onClose} />)
    expect(screen.getByText('No usable items.')).toBeInTheDocument()
  })

  it('calls onSelect with item and index on click', () => {
    const inventory = [makeConsumable({ name: 'Potion' }), null]
    render(<UseItemPanel inventory={inventory} onSelect={onSelect} onClose={onClose} />)
    fireEvent.click(screen.getByText('Potion'))
    expect(onSelect).toHaveBeenCalledWith(inventory[0], 0)
  })

  it('calls onClose on Escape key', () => {
    const inventory = [makeConsumable()]
    render(<UseItemPanel inventory={inventory} onSelect={onSelect} onClose={onClose} />)
    fireEvent.keyDown(window, { key: 'Escape' })
    expect(onClose).toHaveBeenCalledTimes(1)
  })

  it('selects item on Enter key', () => {
    const inventory = [makeConsumable()]
    render(<UseItemPanel inventory={inventory} onSelect={onSelect} onClose={onClose} />)
    fireEvent.keyDown(window, { key: 'Enter' })
    expect(onSelect).toHaveBeenCalled()
  })

  it('navigates with arrow keys', () => {
    const item2 = makeConsumable({ name: 'Second Potion', id: 'item_2' })
    const inventory = [makeConsumable({ name: 'First Potion' }), item2]
    render(<UseItemPanel inventory={inventory} onSelect={onSelect} onClose={onClose} />)
    fireEvent.keyDown(window, { key: 'ArrowDown' })
    fireEvent.keyDown(window, { key: 'Enter' })
    expect(onSelect).toHaveBeenCalledWith(item2, 1)
  })

  it('calls onClose when clicking backdrop', () => {
    const inventory = [makeConsumable()]
    const { container } = render(<UseItemPanel inventory={inventory} onSelect={onSelect} onClose={onClose} />)
    fireEvent.click(container.querySelector('.combat-panel-overlay')!)
    expect(onClose).toHaveBeenCalledTimes(1)
  })

  it('does not call onClose when clicking inside panel', () => {
    const inventory = [makeConsumable()]
    render(<UseItemPanel inventory={inventory} onSelect={onSelect} onClose={onClose} />)
    fireEvent.click(screen.getByText('Use Item'))
    expect(onClose).not.toHaveBeenCalled()
  })
})

describe('FleePanel', () => {
  const onFlee = vi.fn()
  const onClose = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    cleanup()
  })

  it('renders flee confirmation', () => {
    render(<FleePanel onFlee={onFlee} onClose={onClose} />)
    expect(screen.getByText('Flee from combat?')).toBeInTheDocument()
    expect(screen.getByText('Cowardice has its rewards.')).toBeInTheDocument()
    expect(screen.getByText('Yes, Flee')).toBeInTheDocument()
    expect(screen.getByText('Stay & Fight')).toBeInTheDocument()
  })

  it('calls onFlee when clicking Yes', () => {
    render(<FleePanel onFlee={onFlee} onClose={onClose} />)
    fireEvent.click(screen.getByText('Yes, Flee'))
    expect(onFlee).toHaveBeenCalledTimes(1)
  })

  it('calls onClose when clicking Stay', () => {
    render(<FleePanel onFlee={onFlee} onClose={onClose} />)
    fireEvent.click(screen.getByText('Stay & Fight'))
    expect(onClose).toHaveBeenCalledTimes(1)
  })

  it('calls onClose on Escape key', () => {
    render(<FleePanel onFlee={onFlee} onClose={onClose} />)
    fireEvent.keyDown(window, { key: 'Escape' })
    expect(onClose).toHaveBeenCalledTimes(1)
  })

  it('calls onFlee on Enter key', () => {
    render(<FleePanel onFlee={onFlee} onClose={onClose} />)
    fireEvent.keyDown(window, { key: 'Enter' })
    expect(onFlee).toHaveBeenCalledTimes(1)
  })

  it('calls onClose when clicking backdrop', () => {
    const { container } = render(<FleePanel onFlee={onFlee} onClose={onClose} />)
    fireEvent.click(container.querySelector('.combat-panel-overlay')!)
    expect(onClose).toHaveBeenCalledTimes(1)
  })
})
