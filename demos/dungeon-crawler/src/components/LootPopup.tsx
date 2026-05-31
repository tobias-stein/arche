import { useState, useEffect, useCallback, useRef } from 'react'
import { getGameState } from '../GameState'
import { getRarityColor, getItemIcon } from '../game/loot'
import type { ItemState, CreatureState } from '../types'
import './LootPopup.css'

function LootPopup() {
  const [items, setItems] = useState<ItemState[]>([])
  const [creature, setCreature] = useState<CreatureState | null>(null)
  const [xpGained, setXpGained] = useState(0)
  const [visible, setVisible] = useState(false)
  const [selectedIdx, setSelectedIdx] = useState(0)
  const [levelUp, setLevelUp] = useState(false)
  const containerRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const gs = getGameState()

    function onLootShow(lootItems: ItemState[]) {
      setItems([...lootItems])
      setCreature(gs.combatCreature)
      setSelectedIdx(0)
      setVisible(true)
    }

    function onLootChanged(updatedItems: ItemState[]) {
      setItems([...updatedItems])
      if (updatedItems.length === 0 && gs.combatActive) {
        gs.endCombat()
      }
    }

    function onVictory(_creatureId: string, xp: number) {
      setXpGained(xp)
    }

    function onLevelUp() {
      setLevelUp(true)
    }

    function onCombatEnded() {
      setVisible(false)
      setItems([])
      setLevelUp(false)
    }

    gs.on('loot:show', onLootShow)
    gs.on('loot:items-changed', onLootChanged)
    gs.on('combat:victory', onVictory)
    gs.on('xp:level-up', onLevelUp)
    gs.on('combat:ended', onCombatEnded)

    return () => {
      gs.off('loot:show', onLootShow)
      gs.off('loot:items-changed', onLootChanged)
      gs.off('combat:victory', onVictory)
      gs.off('xp:level-up', onLevelUp)
      gs.off('combat:ended', onCombatEnded)
    }
  }, [])

  const closeLoot = useCallback(() => {
    const gs = getGameState()
    gs.endCombat()
    setVisible(false)
    setItems([])
  }, [])

  const handleTakeItem = useCallback((itemId: string) => {
    const gs = getGameState()
    gs.takeLootItem(itemId)
  }, [])

  const handleTakeAll = useCallback(() => {
    const gs = getGameState()
    gs.takeAllLoot()
  }, [])

  const handleLeave = useCallback(() => {
    const gs = getGameState()
    gs.dismissLoot()
    gs.endCombat()
    setVisible(false)
    setItems([])
  }, [])

  const handleOpenInventory = useCallback(() => {
    const gs = getGameState()
    gs.emitInventoryRequested()
  }, [])

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if (!visible) return

      switch (e.key) {
        case 'ArrowDown':
        case 'ArrowRight':
          e.preventDefault()
          setSelectedIdx(prev => Math.min(prev + 1, items.length - 1))
          break
        case 'ArrowUp':
        case 'ArrowLeft':
          e.preventDefault()
          setSelectedIdx(prev => Math.max(prev - 1, 0))
          break
        case 'Enter':
          e.preventDefault()
          if (items[selectedIdx]) {
            handleTakeItem(items[selectedIdx].id)
          }
          break
        case ' ':
          e.preventDefault()
          handleTakeAll()
          break
        case 'Escape':
          e.preventDefault()
          handleLeave()
          break
        case 'i':
        case 'I':
          e.preventDefault()
          handleOpenInventory()
          break
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [visible, items, selectedIdx, handleTakeItem, handleTakeAll, handleLeave, handleOpenInventory])

  useEffect(() => {
    if (visible && containerRef.current) {
      containerRef.current.focus()
    }
  }, [visible])

  if (!visible) return null

  const creatureName = creature?.name ?? 'Enemy'

  return (
    <div className="loot-overlay" ref={containerRef} tabIndex={-1}>
      <div className="victory-bg" onClick={handleLeave} />

      <div className="loot-dialog">
        {levelUp && (
          <div className="ld-level-badge">
            <i className="fa-solid fa-star" /> Level Up!
          </div>
        )}

        <div className="ld-header">
          <h3><i className="fa-solid fa-crosshairs" /> Victory!</h3>
          <span className="ld-xp">+{xpGained} XP</span>
        </div>

        <div className="ld-subtitle">
          Defeated <strong>{creatureName}</strong>
        </div>

        <div className="ld-label">LOOT</div>

        <div className="ld-items">
          {items.map((item, idx) => {
            const rarityColor = getRarityColor(item.rarity)
            const icon = getItemIcon(item)
            return (
              <div
                key={item.id}
                className={`ld-item ${idx === selectedIdx ? 'selected' : ''}`}
                data-rarity={item.rarity}
                onClick={() => handleTakeItem(item.id)}
              >
                <span className="li-icon" style={{ color: rarityColor }}>
                  <i className={icon} />
                </span>
                <span className="li-name">{item.name}</span>
                <span className="li-rarity-corner" style={{ borderColor: `transparent transparent ${rarityColor} transparent` }} />
              </div>
            )
          })}
        </div>

        <div className="ld-actions">
          <button className="ld-btn" onClick={handleTakeAll}>
            <i className="fa-solid fa-box-open" /> Take All
          </button>
          <button className="ld-btn ld-btn-secondary" onClick={handleLeave}>
            <i className="fa-solid fa-door-open" /> Leave
          </button>
          <button className="ld-btn ld-btn-secondary" onClick={handleOpenInventory}>
            <i className="fa-solid fa-suitcase" /> Inventory
          </button>
        </div>

        <div className="ld-hints">
          <span><kbd>Enter</kbd> Take</span>
          <span><kbd>Space</kbd> All</span>
          <span><kbd>Esc</kbd> Leave</span>
          <span><kbd>I</kbd> Inv</span>
        </div>
      </div>
    </div>
  )
}

export default LootPopup
