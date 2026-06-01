import { useState, useEffect, useCallback, useRef } from 'react'
import { getGameState } from '../GameState'
import { getRarityColor, getItemIcon } from '../game/loot'
import type { ItemState, EquipSlot } from '../types'
import { GAME_CONFIG } from '../config'
import './InventoryDialog.css'

const EQUIP_SLOTS: EquipSlot[] = ['weapon', 'helmet', 'chest', 'legs', 'boots', 'gloves', 'belt', 'ring', 'amulet', 'shield']
const EQUIP_SLOT_LABELS: Record<EquipSlot, string> = {
  weapon: 'WEAPON',
  helmet: 'HEAD',
  chest: 'CHEST',
  legs: 'LEGS',
  boots: 'FEET',
  gloves: 'HANDS',
  belt: 'BELT',
  ring: 'RING',
  amulet: 'NECK',
  shield: 'OFFHAND',
}

interface DragState {
  source: HTMLElement
  sourceType: 'equipment' | 'inventory' | 'spell'
  sourceEquipSlot?: EquipSlot
  sourceIdx?: number
  itemType: string
  equipSlot?: EquipSlot
  itemId: string
  html: string
  itemName: string
  itemIcon: string
  rarity: string
}

let dragState: DragState | null = null
let dragFloat: HTMLElement | null = null
let dragOffX = 0
let dragOffY = 0
let pendingAbandon: DragState | null = null

interface Props {
  onClose?: () => void
  lootActive?: boolean
}

function InventoryDialog({ onClose, lootActive }: Props) {
  const [inventory, setInventory] = useState<(ItemState | null)[]>(
    () => [...getGameState().inventory]
  )
  const [equipment, setEquipment] = useState<Partial<Record<EquipSlot, ItemState>>>(
    () => ({ ...getGameState().equipment })
  )
  const [spellbook, setSpellbook] = useState<(ItemState | null)[]>(
    () => [...getGameState().spellbook]
  )
  const [dragActive, setDragActive] = useState(false)
  const [tooltip, setTooltip] = useState<{
    item: ItemState
    x: number
    y: number
    pinned: boolean
  } | null>(null)
  const [abandonItem, setAbandonItem] = useState<DragState | null>(null)
  const [showSpellReplace, setShowSpellReplace] = useState(false)
  const [pendingSpell, setPendingSpell] = useState<ItemState | null>(null)
  const [replaceConfirmIdx, setReplaceConfirmIdx] = useState<number | null>(null)

  const dialogRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const gs = getGameState()

    function onInventoryChanged() {
      syncState()
    }

    gs.on('inventory:changed', onInventoryChanged)

    return () => {
      gs.off('inventory:changed', onInventoryChanged)
      cleanupDrag()
    }
  }, [])

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === 'Escape') {
        e.preventDefault()
        closeDialog()
        return
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [])

  useEffect(() => {
    function onMouseMove(e: MouseEvent) {
      if (dragFloat && dragState) {
        dragFloat.style.left = (e.clientX - dragOffX) + 'px'
        dragFloat.style.top = (e.clientY - dragOffY) + 'px'
      }
    }

    function onMouseUp() {
      if (dragFloat && dragState && !pendingAbandon) {
        dropAtPosition()
      }
    }

    if (dragActive) {
      window.addEventListener('mousemove', onMouseMove)
      window.addEventListener('mouseup', onMouseUp)
      return () => {
        window.removeEventListener('mousemove', onMouseMove)
        window.removeEventListener('mouseup', onMouseUp)
      }
    }
  }, [dragActive])

  const syncState = useCallback(() => {
    const gs = getGameState()
    setInventory([...gs.inventory])
    setEquipment({ ...gs.equipment })
    setSpellbook([...gs.spellbook])
  }, [])

  const closeDialog = useCallback(() => {
    setTooltip(null)
    cleanupDrag()
    setAbandonItem(null)
    setShowSpellReplace(false)
    setPendingSpell(null)
    setReplaceConfirmIdx(null)
    onClose?.()
  }, [onClose])

  const clearDragUI = useCallback(() => {
    document.querySelectorAll('.dragging, .drag-valid, .drag-invalid, .drag-over-trash')
      .forEach(el => el.classList.remove('dragging', 'drag-valid', 'drag-invalid', 'drag-over-trash'))
    const et = document.querySelector('.et')
    if (et) et.classList.remove('et-active')
    if (dragFloat) {
      dragFloat.remove()
      dragFloat = null
    }
    setDragActive(false)
  }, [])

  const cleanupDrag = useCallback(() => {
    clearDragUI()
    dragState = null
    pendingAbandon = null
  }, [clearDragUI])

  const getTargetAtPosition = useCallback((x: number, y: number): {
    type: 'equipment' | 'inventory' | 'spell' | 'trash' | null
    equipSlot?: EquipSlot
    idx?: number
    element: HTMLElement | null
  } => {
    const trash = document.querySelector('.et.et-active')
    if (trash) {
      const rect = trash.getBoundingClientRect()
      if (x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom) {
        return { type: 'trash', element: trash as HTMLElement }
      }
    }

    const el = document.elementsFromPoint(x, y).find(e =>
      e instanceof HTMLElement && e.closest('.el')
    ) as HTMLElement | null
    if (!el) return { type: null, element: null }

    const slot = el.closest('.el') as HTMLElement | null
    if (!slot) return { type: null, element: null }

    const slotType = slot.dataset.slotType
    if (slotType === 'equipment') {
      const es = slot.dataset.equipSlot as EquipSlot
      return { type: 'equipment', equipSlot: es, element: slot }
    }
    if (slotType === 'inventory') {
      return { type: 'inventory', idx: Number(slot.dataset.slotIdx), element: slot }
    }
    if (slotType === 'spell') {
      return { type: 'spell', idx: Number(slot.dataset.slotIdx), element: slot }
    }
    return { type: null, element: null }
  }, [])

  const isValidDrop = useCallback((source: DragState, target: {
    type: 'equipment' | 'inventory' | 'spell' | 'trash' | null
    equipSlot?: EquipSlot
    idx?: number
  }): boolean => {
    if (!target.type || target.type === 'trash') return target.type === 'trash'
    if (source.itemType === 'spell') return target.type === 'spell'
    if (source.itemType === 'consumable') return target.type === 'inventory'
    if (source.itemType === 'equipment' && source.equipSlot) {
      if (target.type === 'inventory') return true
      if (target.type === 'equipment') return source.equipSlot === target.equipSlot
    }
    return false
  }, [])

  function performSwap(source: DragState, target: {
    type: 'equipment' | 'inventory' | 'spell' | null
    equipSlot?: EquipSlot
    idx?: number
  }) {
    const gs = getGameState()

    if (source.sourceType === 'inventory' && target.type === 'inventory' && target.idx !== undefined) {
      gs.swapInventorySlots(source.sourceIdx!, target.idx)
    } else if (source.sourceType === 'inventory' && target.type === 'equipment' && target.equipSlot) {
      gs.moveToEquipment(source.sourceIdx!, target.equipSlot)
    } else if (source.sourceType === 'equipment' && target.type === 'inventory' && target.idx !== undefined) {
      gs.moveToInventory(source.sourceEquipSlot!, target.idx)
    } else if (source.sourceType === 'equipment' && target.type === 'equipment' && target.equipSlot) {
      const item = gs.equipment[source.sourceEquipSlot!]
      const targetItem = gs.equipment[target.equipSlot]
      if (item && targetItem) {
        const tempSlot = source.sourceEquipSlot!
        delete gs.equipment[tempSlot]
        gs.equipment[target.equipSlot] = item
        gs.equipment[tempSlot] = targetItem
        gs.emit('inventory:changed', { inventory: gs.inventory, equipment: gs.equipment, spellbook: gs.spellbook })
      }
    } else if (source.sourceType === 'spell' && target.type === 'spell' && target.idx !== undefined) {
      gs.swapSpellSlots(source.sourceIdx!, target.idx)
    }

    syncState()
  }

  function dropAtPosition() {
    if (!dragState) return

    const mouseEvent = window.event as MouseEvent | undefined
    const x = mouseEvent?.clientX ?? 0
    const y = mouseEvent?.clientY ?? 0

    const target = getTargetAtPosition(x, y)

    if (target.type === 'trash') {
      showAbandonConfirm(dragState)
      return
    }

    if (target.type && isValidDrop(dragState, target as { type: 'equipment' | 'inventory' | 'spell'; equipSlot?: EquipSlot; idx?: number })) {
      performSwap(dragState, target as { type: 'equipment' | 'inventory' | 'spell'; equipSlot?: EquipSlot; idx?: number })
    }

    cleanupDrag()
  }

  function showAbandonConfirm(source: DragState) {
    pendingAbandon = source
    setAbandonItem(source)
    clearDragUI()
  }

  function confirmAbandon() {
    const gs = getGameState()
    if (abandonItem) {
      if (abandonItem.sourceType === 'inventory' && abandonItem.sourceIdx !== undefined) {
        gs.dropItemFromInventory(abandonItem.sourceIdx)
      } else if (abandonItem.sourceType === 'equipment' && abandonItem.sourceEquipSlot) {
        gs.dropItemFromEquipment(abandonItem.sourceEquipSlot)
      } else if (abandonItem.sourceType === 'spell' && abandonItem.sourceIdx !== undefined) {
        gs.dropItemFromSpellbook(abandonItem.sourceIdx)
      }
      syncState()
    }
    setAbandonItem(null)
    pendingAbandon = null
    cleanupDrag()
  }

  function cancelAbandon() {
    setAbandonItem(null)
    pendingAbandon = null
    cleanupDrag()
  }

  function handleDragStart(
    e: React.MouseEvent,
    sourceType: 'equipment' | 'inventory' | 'spell',
    equipSlot?: EquipSlot,
    idx?: number,
  ) {
    e.preventDefault()
    const gs = getGameState()
    let item: ItemState | undefined
    let html = ''
    let itemName = ''
    let itemIcon = ''
    let rarity = ''
    let itemType = ''
    let itemEquipSlot: EquipSlot | undefined

    if (sourceType === 'equipment' && equipSlot) {
      item = gs.equipment[equipSlot]
      if (!item) return
      itemType = 'equipment'
      itemEquipSlot = item.equipSlot
      itemName = item.name
      rarity = item.rarity
    } else if (sourceType === 'inventory' && idx !== undefined) {
      const invItem = gs.inventory[idx]
      if (!invItem) return
      item = invItem
      itemType = item.equipSlot ? 'equipment' : 'consumable'
      itemEquipSlot = item.equipSlot
      itemName = item.name
      rarity = item.rarity
    } else if (sourceType === 'spell' && idx !== undefined) {
      const s = gs.spellbook[idx]
      if (s) item = s
      if (!item) return
      itemType = 'spell'
      itemName = item.name
      rarity = item.rarity
    }

    if (!item) return

    html = `<span class="ei"><i class="${getItemIcon(item)}"></i></span><span class="en">${item.name}</span>`

    dragState = {
      source: e.currentTarget as HTMLElement,
      sourceType,
      sourceEquipSlot: equipSlot,
      sourceIdx: idx,
      itemType,
      equipSlot: itemEquipSlot,
      itemId: item.id,
      html,
      itemName,
      itemIcon,
      rarity,
    }

    const sourceEl = e.currentTarget as HTMLElement
    sourceEl.classList.add('dragging')

    const fl = document.createElement('div')
    fl.className = 'drag-float'
    if (itemEquipSlot) fl.dataset.equipSlot = itemEquipSlot
    fl.innerHTML = html
    const sr = sourceEl.getBoundingClientRect()
    dragOffX = e.clientX - sr.left
    dragOffY = e.clientY - sr.top
    fl.style.left = (e.clientX - dragOffX) + 'px'
    fl.style.top = (e.clientY - dragOffY) + 'px'
    fl.style.width = sr.width + 'px'
    fl.style.height = sr.height + 'px'
    document.body.appendChild(fl)
    dragFloat = fl

    const dropZone = document.querySelector('.et')
    if (dropZone) dropZone.classList.add('et-active')

    setDragActive(true)
  }

  function getTargetType(slotType?: string): 'equipment' | 'inventory' | 'spell' | null {
    if (slotType === 'equipment') return 'equipment'
    if (slotType === 'inventory') return 'inventory'
    if (slotType === 'spell') return 'spell'
    return null
  }

  function handleDragOver(e: React.MouseEvent, equipSlot?: EquipSlot, idx?: number, slotType?: string) {
    if (!dragState) return

    document.querySelectorAll('.drag-valid, .drag-invalid')
      .forEach(el => el.classList.remove('drag-valid', 'drag-invalid'))

    const target = e.currentTarget as HTMLElement
    const targetType = getTargetType(slotType)

    if (targetType && isValidDrop(dragState, { type: targetType, equipSlot, idx })) {
      e.preventDefault()
      target.classList.add('drag-valid')
    } else if (targetType) {
      target.classList.add('drag-invalid')
    }
  }

  function handleDrop(e: React.MouseEvent, equipSlot?: EquipSlot, idx?: number, slotType?: string) {
    e.preventDefault()
    if (!dragState) return

    const targetType = getTargetType(slotType)

    if (targetType && isValidDrop(dragState, { type: targetType, equipSlot, idx })) {
      performSwap(dragState, { type: targetType, equipSlot, idx })
    }

    cleanupDrag()
  }

  function handleMouseEnter(e: React.MouseEvent, item: ItemState) {
    if (tooltip?.pinned) return
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
    setTooltip({
      item,
      x: rect.right + 8,
      y: rect.top,
      pinned: false,
    })
  }

  function handleMouseLeave() {
    if (tooltip?.pinned) return
    setTooltip(null)
  }

  function handleTooltipClick(e: React.MouseEvent, item: ItemState) {
    e.stopPropagation()
    if (tooltip?.pinned) {
      setTooltip(null)
    } else {
      const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
      setTooltip({
        item,
        x: rect.right + 8,
        y: rect.top,
        pinned: true,
      })
    }
  }

  function handleEnterOnItem(sourceType: 'equipment' | 'inventory' | 'spell', equipSlot?: EquipSlot, idx?: number) {
    const gs = getGameState()
    if (sourceType === 'inventory' && idx !== undefined) {
      const item = gs.inventory[idx]
      if (!item) return
      if (item.equipSlot) {
        gs.equipItem(idx)
      } else if (item.archeType === 'consumable') {
        consumeItem(item)
        gs.dropItemFromInventory(idx)
      }
    } else if (sourceType === 'equipment' && equipSlot) {
      gs.unequipItem(equipSlot)
    } else if (sourceType === 'spell' && idx !== undefined) {
      const spell = gs.spellbook[idx]
      if (spell) {
        handleAddSpell(spell)
      }
    }
    syncState()
  }

  function handleXOnItem(sourceType: 'equipment' | 'inventory' | 'spell', equipSlot?: EquipSlot, idx?: number) {
    const gs = getGameState()
    let item: ItemState | undefined
    if (sourceType === 'inventory' && idx !== undefined) {
      item = gs.inventory[idx] ?? undefined
    } else if (sourceType === 'equipment' && equipSlot) {
      item = gs.equipment[equipSlot]
    } else if (sourceType === 'spell' && idx !== undefined) {
      const s = gs.spellbook[idx]
      if (s) item = s
    }
    if (!item) return
    const drag: DragState = {
      source: document.createElement('div'),
      sourceType,
      sourceEquipSlot: equipSlot,
      sourceIdx: idx,
      itemType: sourceType === 'spell' ? 'spell' : item.equipSlot ? 'equipment' : 'consumable',
      equipSlot: item.equipSlot,
      itemId: item.id,
      html: '',
      itemName: item.name,
      itemIcon: getItemIcon(item),
      rarity: item.rarity,
    }
    showAbandonConfirm(drag)
  }

  function handleUOnEquipment(equipSlot: EquipSlot) {
    const gs = getGameState()
    gs.unequipItem(equipSlot)
    syncState()
  }

  function handleDeleteOnSpell(idx: number) {
    const gs = getGameState()
    const spell = gs.spellbook[idx]
    if (!spell) return
    const drag: DragState = {
      source: document.createElement('div'),
      sourceType: 'spell',
      sourceIdx: idx,
      itemType: 'spell',
      itemId: spell.id,
      html: '',
      itemName: spell.name,
      itemIcon: getItemIcon(spell),
      rarity: spell.rarity,
    }
    showAbandonConfirm(drag)
  }

  function consumeItem(item: ItemState) {
    const gs = getGameState()
    const heal = item.stats.heal ?? 0
    if (heal > 0) {
      gs.setPlayerHp(gs.player.hp.current + heal)
    }
    const mana = item.stats.mana ?? 0
    if (mana > 0) {
      gs.setPlayerMp(gs.player.mp.current + mana)
    }
    gs.addLogEntry({
      type: 'potion_consumed',
      message: `Consumed ${item.name}`,
      icon: 'fa-solid fa-flask',
    })
  }

  function handleAddSpell(spell: ItemState) {
    const gs = getGameState()
    const result = gs.addSpellToBook(spell)
    if (!result.success && result.replace) {
      setPendingSpell(spell)
      setShowSpellReplace(true)
    }
    syncState()
  }

  function handleReplaceSpell(slotIdx: number) {
    setReplaceConfirmIdx(slotIdx)
  }

  function confirmReplace() {
    if (replaceConfirmIdx !== null && pendingSpell) {
      const gs = getGameState()
      gs.replaceSpell(replaceConfirmIdx, pendingSpell)
      syncState()
    }
    setShowSpellReplace(false)
    setPendingSpell(null)
    setReplaceConfirmIdx(null)
  }

  function cancelReplace() {
    setShowSpellReplace(false)
    setPendingSpell(null)
    setReplaceConfirmIdx(null)
  }

  function getCompareItem(item: ItemState): ItemState | null {
    if (!item.equipSlot) return null
    return getGameState().equipment[item.equipSlot] ?? null
  }

  function renderTooltip() {
    if (!tooltip) return null
    const { item, x, y } = tooltip
    const rarityColor = getRarityColor(item.rarity)
    const compareItem = getCompareItem(item)

    return (
      <div
        className="inv-tooltip"
        style={{
          left: Math.min(x, window.innerWidth - 230),
          top: Math.min(y, window.innerHeight - 200),
        }}
        onClick={() => setTooltip(null)}
      >
        <div className="tt-name" style={{ color: rarityColor }}>
          {item.name}
        </div>
        <div className="tt-rarity" style={{ background: rarityColor }}>
          {item.rarity}
        </div>
        {item.subtype && <div className="tt-desc">A {item.subtype} item</div>}
        {item.equipSlot && (
          <div className="tt-slot">{EQUIP_SLOT_LABELS[item.equipSlot]}</div>
        )}
        <div className="tt-stats">
          {Object.entries(item.stats).map(([key, val]) => (
            <span key={key}>{key}: {val}</span>
          ))}
          {item.level > 0 && <span>Level: {item.level}</span>}
        </div>
        {compareItem && (
          <div className="tt-compare">
            <div className="tt-clbl">Equipped:</div>
            <div style={{ color: getRarityColor(compareItem.rarity) }}>
              {compareItem.name}
            </div>
            {Object.entries(compareItem.stats).map(([key, val]) => (
              <span key={key} className="tt-cstat">
                {key}: {val} {item.stats[key] !== undefined ? (val > item.stats[key] ? '(+)' : val < item.stats[key] ? '(-)' : '') : ''}
              </span>
            ))}
          </div>
        )}
        {!compareItem && item.equipSlot && (
          <div className="tt-compare">
            <div className="tt-clbl">Equipped:</div>
            <span className="tt-cstat empty">(empty)</span>
          </div>
        )}
      </div>
    )
  }

  function renderAbandonDialog() {
    if (!abandonItem) return null
    return (
      <div id="abandon-dialog" className="visible">
        <div className="ad">
          <p>Abandon <strong>{abandonItem.itemName}</strong>?</p>
          <p className="ad-sub">This item will be destroyed.</p>
          <div className="ad-item">
            <span className="ai"><i className={abandonItem.itemIcon || 'fa-solid fa-circle-question'} /></span>
            <span className="al">{abandonItem.itemName}</span>
          </div>
          <div className="ab">
            <button className="ad-yes" onClick={confirmAbandon}>Yes, Delete</button>
            <button className="ad-no" onClick={cancelAbandon}>Cancel</button>
          </div>
        </div>
      </div>
    )
  }

  function renderSpellReplaceDialog() {
    if (!showSpellReplace || !pendingSpell) return null
    const gs = getGameState()
    const currentSpells = gs.spellbook

    if (replaceConfirmIdx !== null) {
      const oldSpell = currentSpells[replaceConfirmIdx]
      return (
        <div id="spell-replace-dialog" className="visible">
          <div className="sr">
            <h3>Replace Spell?</h3>
            <p>
              Replace <strong>{oldSpell?.name ?? 'empty'}</strong> with{' '}
              <strong>{pendingSpell.name}</strong>?
            </p>
            <div className="sr-actions">
              <button className="sr-yes" onClick={confirmReplace}>Yes, Replace</button>
              <button onClick={cancelReplace}>Cancel</button>
            </div>
          </div>
        </div>
      )
    }

    return (
      <div id="spell-replace-dialog" className="visible">
        <div className="sr">
          <h3>Spellbook Full</h3>
          <p>Select a spell to replace with <strong>{pendingSpell.name}</strong></p>
          <div className="sr-grid">
            {currentSpells.map((spell, idx) => (
              <div
                key={idx}
                className={`el ${spell ? 'has' : ''}`}
                onClick={() => spell && handleReplaceSpell(idx)}
              >
                {spell ? (
                  <>
                    <span className="ei"><i className={getItemIcon(spell)} /></span>
                    <span className="en">{spell.name}</span>
                  </>
                ) : (
                  <span className="en">Empty</span>
                )}
              </div>
            ))}
            <div className="el highlight">
              <span className="ei"><i className={getItemIcon(pendingSpell)} /></span>
              <span className="en">{pendingSpell.name}</span>
            </div>
          </div>
          <div className="sr-actions">
            <button onClick={cancelReplace}>Cancel</button>
          </div>
        </div>
      </div>
    )
  }

  function renderEquipmentSlot(equipSlot: EquipSlot) {
    const item = equipment[equipSlot]
    const label = EQUIP_SLOT_LABELS[equipSlot]

    return (
      <div
        key={`eq-${equipSlot}`}
        className={`el ${item ? 'has' : ''}`}
        data-equip-slot={label}
        data-slot-type="equipment"
        data-rarity={item?.rarity ?? ''}
        onMouseDown={(e) => item && handleDragStart(e, 'equipment', equipSlot)}
        onMouseOver={(e) => item && handleMouseEnter(e, item)}
        onMouseOut={handleMouseLeave}
        onClick={(e) => item && handleTooltipClick(e, item)}
        onMouseMove={(e) => item && handleDragOver(e, equipSlot, undefined, 'equipment')}
        onMouseUp={(e) => handleDrop(e, equipSlot, undefined, 'equipment')}
        onKeyDown={(e) => {
          if (e.key === 'Enter' && item) handleEnterOnItem('equipment', equipSlot)
          if (e.key === 'x' || e.key === 'X') handleXOnItem('equipment', equipSlot)
          if (e.key === 'u' || e.key === 'U') handleUOnEquipment(equipSlot)
        }}
        tabIndex={0}
      >
        {item ? (
          <>
            <span className="ei"><i className={getItemIcon(item)} style={{ color: getRarityColor(item.rarity) }} /></span>
            <span className="en">{item.name}</span>
          </>
        ) : null}
      </div>
    )
  }

  function renderInventorySlots() {
    const slots: React.ReactNode[] = []
    for (let i = 0; i < GAME_CONFIG.capacity.inventorySlots; i++) {
      const item = inventory[i]
      const equipSlotLabel = item?.equipSlot && EQUIP_SLOT_LABELS[item.equipSlot]
      slots.push(
        <div
          key={`inv-${i}`}
          className={`el ${item ? 'has' : ''}`}
          data-slot-type="inventory"
          data-slot-idx={i}
          data-rarity={item?.rarity ?? ''}
          data-equip-slot={equipSlotLabel || undefined}
          onMouseDown={(e) => item && handleDragStart(e, 'inventory', undefined, i)}
          onMouseOver={(e) => item && handleMouseEnter(e, item)}
          onMouseOut={handleMouseLeave}
          onClick={(e) => item && handleTooltipClick(e, item)}
          onMouseMove={(e) => item && handleDragOver(e, undefined, i, 'inventory')}
          onMouseUp={(e) => handleDrop(e, undefined, i, 'inventory')}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && item) handleEnterOnItem('inventory', undefined, i)
            if (e.key === 'x' || e.key === 'X') handleXOnItem('inventory', undefined, i)
          }}
          tabIndex={0}
        >
          {item ? (
            <>
              <span className="ei"><i className={getItemIcon(item)} style={{ color: getRarityColor(item.rarity) }} /></span>
              <span className="en">{item.name}</span>
            </>
          ) : null}
        </div>,
      )
    }
    return slots
  }

  function renderSpellSlots() {
    const slots: React.ReactNode[] = []

    for (let i = 0; i < GAME_CONFIG.capacity.spellbookSlots; i++) {
      const item = spellbook[i]
      slots.push(
        <div
          key={`spell-${i}`}
          className={`el ${item ? 'has' : ''}`}
          data-slot-type="spell"
          data-slot-idx={i}
          data-rarity={item?.rarity ?? ''}
          onMouseDown={(e) => item && handleDragStart(e, 'spell', undefined, i)}
          onMouseOver={(e) => item && handleMouseEnter(e, item)}
          onMouseOut={handleMouseLeave}
          onClick={(e) => item && handleTooltipClick(e, item)}
          onMouseMove={(e) => item && handleDragOver(e, undefined, i, 'spell')}
          onMouseUp={(e) => handleDrop(e, undefined, i, 'spell')}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && item) handleEnterOnItem('spell', undefined, i)
            if (e.key === 'x' || e.key === 'X') handleXOnItem('spell', undefined, i)
            if (e.key === 'Delete' || e.key === 'Backspace') handleDeleteOnSpell(i)
          }}
          tabIndex={0}
        >
          {item ? (
            <>
              <span className="ei"><i className={getItemIcon(item)} style={{ color: getRarityColor(item.rarity) }} /></span>
              <span className="en">{item.name}</span>
            </>
          ) : null}
        </div>,
      )
    }
    return slots
  }

  return (
    <>
      <div id="equipment-dialog" className={`visible${lootActive ? ' loot-mode' : ''}`} ref={dialogRef}>
        <div className="ed">
          <div className="eh">
            <h2>INVENTORY &amp; EQUIPMENT</h2>
            <div className="eb">
              <button className="ec" onClick={closeDialog}>
                <i className="fa-solid fa-xmark" />
              </button>
            </div>
          </div>

          <div className="es-title">Equipment</div>
          <div className="eg eq-grid">
            {EQUIP_SLOTS.map(slot => renderEquipmentSlot(slot))}
          </div>

          <div className="es-title">Inventory</div>
          <div className="eg inv-grid">
            {renderInventorySlots()}
          </div>

          <div className="es-title">Spells</div>
          <div className="eg spell-grid">
            {renderSpellSlots()}
          </div>

          <div className="et">
            <span className="eti"><i className="fa-solid fa-hand" /></span>
            DROP
            <span className="eti"><i className="fa-solid fa-trash-can" /></span>
          </div>
        </div>
      </div>

      {renderTooltip()}
      {renderAbandonDialog()}
      {renderSpellReplaceDialog()}
    </>
  )
}

export default InventoryDialog
