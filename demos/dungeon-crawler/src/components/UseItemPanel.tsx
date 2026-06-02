import { useState, useEffect } from 'react'
import type { ItemState } from '../types'
import { ceil } from '../utils/format'

interface UseItemPanelProps {
  inventory: (ItemState | null)[]
  onSelect: (item: ItemState, idx: number) => void
  onClose: () => void
}

function UseItemPanel({ inventory, onSelect, onClose }: UseItemPanelProps) {
  const consumables: { item: ItemState; idx: number }[] = []
  inventory.forEach((item, idx) => {
    if (item && item.archeType === 'consumable') {
      consumables.push({ item, idx })
    }
  })
  const [focusIdx, setFocusIdx] = useState(0)

  useEffect(() => {
    setFocusIdx(0)
  }, [inventory])

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      switch (e.key) {
        case 'ArrowUp':
          setFocusIdx(i => (i > 0 ? i - 1 : consumables.length - 1))
          e.preventDefault()
          break
        case 'ArrowDown':
          setFocusIdx(i => (i < consumables.length - 1 ? i + 1 : 0))
          e.preventDefault()
          break
        case 'Enter':
          if (consumables.length > 0) {
            const { item, idx } = consumables[focusIdx]
            onSelect(item, idx)
          }
          e.preventDefault()
          break
        case 'Escape':
          onClose()
          e.preventDefault()
          break
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [consumables, focusIdx, onSelect, onClose])

  return (
    <div className="combat-panel-overlay" onClick={onClose}>
      <div className="combat-panel item-panel" onClick={e => e.stopPropagation()}>
        <div className="cp-header">
          <span>Use Item</span>
          <button className="cp-close" onClick={onClose} aria-label="Close">×</button>
        </div>
        <div className="cp-list">
          {consumables.length === 0 ? (
            <div className="cp-empty">No usable items.</div>
          ) : (
            consumables.map(({ item, idx }, i) => {
              const healVal = item.stats.heal ?? 0
              const manaVal = item.stats.mana ?? 0
              const effectText = healVal > 0 ? `+${ceil(healVal)} HP` : manaVal > 0 ? `+${ceil(manaVal)} MP` : ''
              return (
                <div
                  key={`${item.id}-${idx}`}
                  className={`cp-item ${i === focusIdx ? 'focused' : ''}`}
                  onClick={() => onSelect(item, idx)}
                >
                  <span className="cpi-icon"><i className="fa-solid fa-flask" /></span>
                  <span className="cpi-name">{item.name}</span>
                  <span className="cpi-effect">{effectText}</span>
                </div>
              )
            })
          )}
        </div>
      </div>
    </div>
  )
}

export default UseItemPanel
