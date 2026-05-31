import { useState, useEffect, useCallback } from 'react'
import './ActionMenu.css'

interface ActionMenuProps {
  onAttack: () => void
  onCastSpell: () => void
  onUseItem: () => void
  onFlee: () => void
  disabled?: boolean
}

const ACTIONS = [
  { id: 'attack', icon: 'fa-crosshairs', label: 'Attack', row: 0, col: 0 },
  { id: 'cast', icon: 'fa-wand-sparkles', label: 'Cast Spell', row: 0, col: 1 },
  { id: 'item', icon: 'fa-flask', label: 'Use Item', row: 1, col: 0 },
  { id: 'flee', icon: 'fa-person-running', label: 'Flee', row: 1, col: 1 },
] as const

function ActionMenu({ onAttack, onCastSpell, onUseItem, onFlee, disabled = false }: ActionMenuProps) {
  const [focusIdx, setFocusIdx] = useState(0)

  const handleAction = useCallback((id: string) => {
    if (disabled) return
    switch (id) {
      case 'attack': onAttack(); break
      case 'cast': onCastSpell(); break
      case 'item': onUseItem(); break
      case 'flee': onFlee(); break
    }
  }, [disabled, onAttack, onCastSpell, onUseItem, onFlee])

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if (disabled) return
      switch (e.key) {
        case 'ArrowUp': setFocusIdx(i => (i >= 2 ? i - 2 : i)); e.preventDefault(); break
        case 'ArrowDown': setFocusIdx(i => (i < 2 ? i + 2 : i)); e.preventDefault(); break
        case 'ArrowLeft': setFocusIdx(i => (i % 2 === 1 ? i - 1 : i)); e.preventDefault(); break
        case 'ArrowRight': setFocusIdx(i => (i % 2 === 0 && i < 3 ? i + 1 : i)); e.preventDefault(); break
        case 'Enter': handleAction(ACTIONS[focusIdx].id); e.preventDefault(); break
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [disabled, focusIdx, handleAction])

  return (
    <div id="action-menu">
      {ACTIONS.map((action, idx) => (
        <button
          key={action.id}
          className={idx === focusIdx ? 'focused' : ''}
          onClick={() => handleAction(action.id)}
          disabled={disabled}
        >
          <span className="ai"><i className={`fa-solid ${action.icon}`} /></span>
          <span className="al">{action.label}</span>
        </button>
      ))}
    </div>
  )
}

export default ActionMenu
