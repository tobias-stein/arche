import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, fireEvent, cleanup, act } from '@testing-library/react'
import { getGameState, resetGameState } from '../GameState'
import InventoryDialog from '../components/InventoryDialog'
import { GAME_CONFIG } from '../config'
import type { ItemState, EquipSlot } from '../types'
import { resetItemIdCounter } from '../game/loot'

function makeItem(overrides: Partial<ItemState> & { name: string; equipSlot?: EquipSlot }): ItemState {
  return {
    id: overrides.id ?? `item_${Math.random()}`,
    name: overrides.name,
    archeType: overrides.archeType ?? (overrides.equipSlot || 'weapon'),
    rarity: overrides.rarity ?? 'common',
    level: overrides.level ?? 1,
    stats: overrides.stats ?? {},
    affixes: [],
    equipSlot: overrides.equipSlot,
    subtype: overrides.subtype,
  }
}

beforeEach(() => {
  resetItemIdCounter()
  resetGameState()
})

afterEach(() => {
  cleanup()
})

describe('InventoryDialog', () => {
  it('renders when mounted', () => {
    render(<InventoryDialog />)
    expect(screen.getByText(/INVENTORY & EQUIPMENT/i)).toBeInTheDocument()
  })

  it('shows Equipment, Inventory, and Spells sections', () => {
    render(<InventoryDialog />)
    expect(screen.getByText('Equipment')).toBeInTheDocument()
    expect(screen.getByText('Inventory')).toBeInTheDocument()
    expect(screen.getByText('Spells')).toBeInTheDocument()
  })

  it('shows close button', () => {
    render(<InventoryDialog />)
    const closeBtn = document.querySelector('.ec')
    expect(closeBtn).toBeInTheDocument()
  })

  it('calls onClose when close button is clicked', () => {
    const onClose = vi.fn()
    render(<InventoryDialog onClose={onClose} />)
    const closeBtn = document.querySelector('.ec')!
    fireEvent.click(closeBtn)
    expect(onClose).toHaveBeenCalledTimes(1)
  })

  it('calls onClose on Escape key', () => {
    const onClose = vi.fn()
    render(<InventoryDialog onClose={onClose} />)
    act(() => {
      fireEvent.keyDown(window, { key: 'Escape' })
    })
    expect(onClose).toHaveBeenCalledTimes(1)
  })

  it('renders 10 equipment slots', () => {
    const { container } = render(<InventoryDialog />)
    const eqGrid = container.querySelector('.eq-grid')
    expect(eqGrid).toBeInTheDocument()
    const slots = eqGrid!.querySelectorAll('.el')
    expect(slots.length).toBe(10)
  })

  it('renders 18 inventory slots', () => {
    const { container } = render(<InventoryDialog />)
    const invGrid = container.querySelector('.inv-grid')
    expect(invGrid).toBeInTheDocument()
    const slots = invGrid!.querySelectorAll('.el')
    expect(slots.length).toBe(GAME_CONFIG.capacity.inventorySlots)
  })

  it('renders 6 spell slots', () => {
    const { container } = render(<InventoryDialog />)
    const spellGrid = container.querySelector('.spell-grid')
    expect(spellGrid).toBeInTheDocument()
    const slots = spellGrid!.querySelectorAll('.el')
    expect(slots.length).toBe(GAME_CONFIG.capacity.spellbookSlots)
  })

  it('displays inventory items from game state', () => {
    const gs = getGameState()
    const item = makeItem({ name: 'Iron Sword', equipSlot: 'weapon' })
    gs.inventory[0] = item

    render(<InventoryDialog />)
    expect(screen.getByText('Iron Sword')).toBeInTheDocument()
  })

  it('displays equipment items from game state', () => {
    const gs = getGameState()
    const item = makeItem({ name: 'Steel Helm', equipSlot: 'helmet' })
    gs.equipment.helmet = item

    render(<InventoryDialog />)
    expect(screen.getByText('Steel Helm')).toBeInTheDocument()
  })

  it('displays equipment slot labels', () => {
    const { container } = render(<InventoryDialog />)
    const eqSlots = container.querySelectorAll('.eq-grid .el')
    expect(eqSlots.length).toBe(10)

    const labels = ['WEAPON', 'HEAD', 'CHEST', 'LEGS', 'FEET', 'HANDS', 'BELT', 'RING', 'NECK', 'OFFHAND']
    eqSlots.forEach((slot, idx) => {
      expect(slot.getAttribute('data-equip-slot')).toBe(labels[idx])
    })
  })

  it('shows empty slots without has class', () => {
    const { container } = render(<InventoryDialog />)
    const invSlots = container.querySelectorAll('.inv-grid .el')
    invSlots.forEach(slot => {
      expect(slot.classList.contains('has')).toBe(false)
    })
  })

  it('shows filled slots with has class', () => {
    const gs = getGameState()
    const item = makeItem({ name: 'Health Potion' })
    gs.inventory[0] = item

    const { container } = render(<InventoryDialog />)
    const firstSlot = container.querySelector('.inv-grid .el:first-child')
    expect(firstSlot!.classList.contains('has')).toBe(true)
  })

  it('renders rarity data attribute on items', () => {
    const gs = getGameState()
    const item = makeItem({ name: 'Rare Sword', rarity: 'rare', equipSlot: 'weapon' })
    gs.equipment.weapon = item

    const { container } = render(<InventoryDialog />)
    const slot = container.querySelector('.eq-grid .el.has')
    expect(slot!.getAttribute('data-rarity')).toBe('rare')
  })

  it('shows drop zone', () => {
    const { container } = render(<InventoryDialog />)
    const dropZone = container.querySelector('.et')
    expect(dropZone).toBeInTheDocument()
    expect(dropZone!.textContent).toContain('DROP')
  })

  it('equips item from inventory when clicking Enter on it', () => {
    const gs = getGameState()
    const item = makeItem({ name: 'Iron Sword', equipSlot: 'weapon' })
    gs.inventory[0] = item

    render(<InventoryDialog />)
    const firstInvSlot = document.querySelector('.inv-grid .el:first-child')!
    fireEvent.keyDown(firstInvSlot, { key: 'Enter' })

    expect(gs.equipment.weapon?.name).toBe('Iron Sword')
    expect(gs.inventory[0]).toBeNull()
  })

  it('unequips item when clicking Enter on equipped slot', () => {
    const gs = getGameState()
    const item = makeItem({ name: 'Iron Sword', equipSlot: 'weapon' })
    gs.equipment.weapon = item
    gs.inventory[0] = null

    render(<InventoryDialog />)
    const equipSlot = document.querySelector('.eq-grid .el:first-child')!
    fireEvent.keyDown(equipSlot, { key: 'Enter' })

    expect(gs.equipment.weapon).toBeUndefined()
    expect(gs.inventory[0]!.name).toBe('Iron Sword')
  })

  it('consumes potion from inventory when clicking Enter', () => {
    const gs = getGameState()
    const potion = makeItem({
      name: 'Health Potion',
      archeType: 'consumable',
      stats: { heal: 50 },
    })
    gs.player.hp.current = 50
    gs.inventory[0] = potion

    render(<InventoryDialog />)
    const firstInvSlot = document.querySelector('.inv-grid .el:first-child')!
    fireEvent.keyDown(firstInvSlot, { key: 'Enter' })

    expect(gs.player.hp.current).toBe(100)
    expect(gs.inventory[0]).toBeNull()
  })

  it('spellbook shows spells from game state', () => {
    const gs = getGameState()
    const spell = makeItem({ name: 'Fire Bolt', archeType: 'spell' })
    gs.spellbook[0] = spell

    render(<InventoryDialog />)
    expect(screen.getByText('Fire Bolt')).toBeInTheDocument()
  })

  it('opens abandon dialog when X is pressed on an item', () => {
    const gs = getGameState()
    const item = makeItem({ name: 'Old Sword', equipSlot: 'weapon' })
    gs.inventory[0] = item

    render(<InventoryDialog />)
    const firstInvSlot = document.querySelector('.inv-grid .el:first-child')!
    fireEvent.keyDown(firstInvSlot, { key: 'x' })

    expect(screen.getByText(/Abandon/i)).toBeInTheDocument()
    expect(screen.getByText('Yes, Delete')).toBeInTheDocument()
  })

  it('cancels abandon and keeps item', () => {
    const gs = getGameState()
    const item = makeItem({ name: 'Old Sword', equipSlot: 'weapon' })
    gs.inventory[0] = item

    render(<InventoryDialog />)
    const firstInvSlot = document.querySelector('.inv-grid .el:first-child')!
    fireEvent.keyDown(firstInvSlot, { key: 'x' })

    const cancelBtn = screen.getByText('Cancel')
    fireEvent.click(cancelBtn)

    expect(gs.inventory[0]?.name).toBe('Old Sword')
    expect(screen.queryByText('Yes, Delete')).not.toBeInTheDocument()
  })

  it('confirms abandon and removes item', () => {
    const gs = getGameState()
    const item = makeItem({ name: 'Old Sword', equipSlot: 'weapon' })
    gs.inventory[0] = item

    render(<InventoryDialog />)
    const firstInvSlot = document.querySelector('.inv-grid .el:first-child')!
    fireEvent.keyDown(firstInvSlot, { key: 'x' })

    const deleteBtn = screen.getByText('Yes, Delete')
    fireEvent.click(deleteBtn)

    expect(gs.inventory[0]).toBeNull()
  })

  it('U key unequips item to inventory', () => {
    const gs = getGameState()
    const item = makeItem({ name: 'Iron Sword', equipSlot: 'weapon' })
    gs.equipment.weapon = item
    gs.inventory[0] = null

    render(<InventoryDialog />)
    const equipSlot = document.querySelector('.eq-grid .el:first-child')!
    fireEvent.keyDown(equipSlot, { key: 'u' })

    expect(gs.equipment.weapon).toBeUndefined()
    expect(gs.inventory[0]!.name).toBe('Iron Sword')
  })

  it('Delete key on spell opens abandon dialog', () => {
    const gs = getGameState()
    const spell = makeItem({ name: 'Fire Bolt', archeType: 'spell' })
    gs.spellbook[0] = spell

    render(<InventoryDialog />)
    const firstSpellSlot = document.querySelector('.spell-grid .el:first-child')!
    fireEvent.keyDown(firstSpellSlot, { key: 'Delete' })

    expect(screen.getByText(/Abandon/i)).toBeInTheDocument()
  })

  it('moveToEquipment moves item from inventory to equipment', () => {
    const gs = getGameState()
    const item = makeItem({ name: 'Iron Sword', equipSlot: 'weapon' })
    gs.inventory[0] = item

    render(<InventoryDialog />)
    act(() => {
      gs.moveToEquipment(0, 'weapon')
    })

    expect(gs.equipment.weapon?.name).toBe('Iron Sword')
    expect(gs.inventory[0]).toBeNull()
  })

  it('moveToInventory moves item from equipment to inventory', () => {
    const gs = getGameState()
    const item = makeItem({ name: 'Iron Sword', equipSlot: 'weapon' })
    gs.equipment.weapon = item
    gs.inventory[0] = null

    render(<InventoryDialog />)
    act(() => {
      gs.moveToInventory('weapon', 0)
    })

    expect(gs.equipment.weapon).toBeUndefined()
    expect(gs.inventory[0]!.name).toBe('Iron Sword')
  })

  it('swapInventorySlots swaps items between inventory slots', () => {
    const gs = getGameState()
    const sword = makeItem({ name: 'Sword', equipSlot: 'weapon' })
    const shield = makeItem({ name: 'Shield', equipSlot: 'shield' })
    gs.inventory[0] = sword
    gs.inventory[1] = shield

    render(<InventoryDialog />)
    act(() => {
      gs.swapInventorySlots(0, 1)
    })

    expect(gs.inventory[0]?.name).toBe('Shield')
    expect(gs.inventory[1]?.name).toBe('Sword')
  })

  it('addSpellToBook adds spell to empty slot', () => {
    const gs = getGameState()
    const spell = makeItem({ name: 'Fire Bolt', archeType: 'spell' })

    render(<InventoryDialog />)
    act(() => {
      gs.addSpellToBook(spell)
    })

    expect(gs.spellbook[0]?.name).toBe('Fire Bolt')
  })

  it('addSpellToBook returns replace=true when full', () => {
    const gs = getGameState()
    for (let i = 0; i < GAME_CONFIG.capacity.spellbookSlots; i++) {
      gs.spellbook[i] = makeItem({ name: `Spell ${i}`, archeType: 'spell' })
    }
    const newSpell = makeItem({ name: 'New Spell', archeType: 'spell' })

    const result = gs.addSpellToBook(newSpell)
    expect(result.success).toBe(false)
    expect(result.replace).toBe(true)
  })

  it('replaceSpell replaces a spell slot', () => {
    const gs = getGameState()
    for (let i = 0; i < GAME_CONFIG.capacity.spellbookSlots; i++) {
      gs.spellbook[i] = makeItem({ name: `Spell ${i}`, archeType: 'spell' })
    }
    const newSpell = makeItem({ name: 'New Spell', archeType: 'spell' })

    render(<InventoryDialog />)
    act(() => {
      gs.replaceSpell(0, newSpell)
    })

    expect(gs.spellbook[0]?.name).toBe('New Spell')
  })

  it('swaps spells between spell slots', () => {
    const gs = getGameState()
    gs.spellbook[0] = makeItem({ name: 'Fire', archeType: 'spell' })
    gs.spellbook[1] = makeItem({ name: 'Ice', archeType: 'spell' })

    render(<InventoryDialog />)
    act(() => {
      gs.swapSpellSlots(0, 1)
    })

    expect(gs.spellbook[0]?.name).toBe('Ice')
    expect(gs.spellbook[1]?.name).toBe('Fire')
  })

  it('dropItemFromInventory removes item', () => {
    const gs = getGameState()
    gs.inventory[0] = makeItem({ name: 'Item', equipSlot: 'weapon' })

    render(<InventoryDialog />)
    act(() => {
      gs.dropItemFromInventory(0)
    })

    expect(gs.inventory[0]).toBeNull()
  })

  it('dropItemFromEquipment removes item', () => {
    const gs = getGameState()
    gs.equipment.weapon = makeItem({ name: 'Sword', equipSlot: 'weapon' })

    render(<InventoryDialog />)
    act(() => {
      gs.dropItemFromEquipment('weapon')
    })

    expect(gs.equipment.weapon).toBeUndefined()
  })

  it('dropItemFromSpellbook removes spell', () => {
    const gs = getGameState()
    gs.spellbook[0] = makeItem({ name: 'Fire Bolt', archeType: 'spell' })

    render(<InventoryDialog />)
    act(() => {
      gs.dropItemFromSpellbook(0)
    })

    expect(gs.spellbook[0]).toBeNull()
  })
})
