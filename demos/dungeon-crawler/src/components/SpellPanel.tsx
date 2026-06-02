import { useState, useEffect } from 'react'
import type { ItemState } from '../types'
import { ceil } from '../utils/format'

interface SpellPanelProps {
  spells: (ItemState | null)[]
  playerMp: number
  onSelect: (spell: ItemState) => void
  onClose: () => void
}

function SpellPanel({ spells, playerMp, onSelect, onClose }: SpellPanelProps) {
  const knownSpells = spells.filter((s): s is ItemState => s !== null)
  const [focusIdx, setFocusIdx] = useState(0)

  useEffect(() => {
    setFocusIdx(0)
  }, [spells])

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      switch (e.key) {
        case 'ArrowUp':
          setFocusIdx(i => (i > 0 ? i - 1 : knownSpells.length - 1))
          e.preventDefault()
          break
        case 'ArrowDown':
          setFocusIdx(i => (i < knownSpells.length - 1 ? i + 1 : 0))
          e.preventDefault()
          break
        case 'Enter':
          if (knownSpells.length > 0) {
            const spell = knownSpells[focusIdx]
            const manaCost = spell.stats.mana_cost ?? 0
            if (playerMp >= manaCost) {
              onSelect(spell)
            }
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
  }, [knownSpells, focusIdx, playerMp, onSelect, onClose])

  return (
    <div className="combat-panel-overlay" onClick={onClose}>
      <div className="combat-panel spell-panel" onClick={e => e.stopPropagation()}>
        <div className="cp-header">
          <span>Select Spell</span>
          <button className="cp-close" onClick={onClose} aria-label="Close">×</button>
        </div>
        <div className="cp-list">
          {knownSpells.length === 0 ? (
            <div className="cp-empty">No spells known.</div>
          ) : (
            knownSpells.map((spell, idx) => {
              const manaCost = spell.stats.mana_cost ?? 0
              const canCast = playerMp >= manaCost
              return (
                <div
                  key={spell.id}
                  className={`cp-item ${idx === focusIdx ? 'focused' : ''} ${canCast ? '' : 'disabled'}`}
                  onClick={() => canCast && onSelect(spell)}
                >
                  <span className="cpi-icon"><i className="fa-solid fa-wand-sparkles" /></span>
                  <span className="cpi-name">{spell.name}</span>
                  <span className="cpi-cost">{ceil(manaCost)} MP</span>
                </div>
              )
            })
          )}
        </div>
      </div>
    </div>
  )
}

export default SpellPanel
