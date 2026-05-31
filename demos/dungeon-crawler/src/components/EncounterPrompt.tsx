import { useState, useEffect, useCallback } from 'react'
import { getGameState } from '../GameState'
import type { CreatureState } from '../types'
import './EncounterPrompt.css'

const DIFFICULTY_COLORS: Record<string, string> = {
  normal: '#88b898',
  champion: '#ffa500',
  elite: '#ff4444',
  boss: '#aa44ff',
}

function EncounterPrompt() {
  const [creature, setCreature] = useState<CreatureState | null>(null)
  const [visible, setVisible] = useState(false)

  useEffect(() => {
    const gs = getGameState()

    function onEncounterPrompt(c: CreatureState) {
      setCreature(c)
      setVisible(true)
    }

    function onEncounterDismissed() {
      setVisible(false)
      setCreature(null)
    }

    function onCombatStarted() {
      setVisible(false)
      setCreature(null)
    }

    gs.on('encounter:prompt', onEncounterPrompt)
    gs.on('encounter:dismissed', onEncounterDismissed)
    gs.on('combat:started', onCombatStarted)

    return () => {
      gs.off('encounter:prompt', onEncounterPrompt)
      gs.off('encounter:dismissed', onEncounterDismissed)
      gs.off('combat:started', onCombatStarted)
    }
  }, [])

  const handleEngage = useCallback(() => {
    getGameState().proceedToCombat()
  }, [])

  const handleBackAway = useCallback(() => {
    getGameState().backAwayFromEncounter()
  }, [])

  useEffect(() => {
    if (!visible) return

    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === 'Enter') {
        e.preventDefault()
        handleEngage()
      }
      if (e.key === 'Escape') {
        e.preventDefault()
        handleBackAway()
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [visible, handleEngage, handleBackAway])

  if (!visible || !creature) return null

  const hpPercent = creature.hp.max > 0 ? (creature.hp.current / creature.hp.max) * 100 : 0
  const badgeColor = DIFFICULTY_COLORS[creature.difficulty] ?? '#88b898'

  return (
    <div className="encounter-overlay" data-testid="encounter-prompt">
      <div className="encounter-card">
        <h3 className="encounter-name">{creature.name}</h3>
        <div className="encounter-difficulty" style={{ color: badgeColor }}>
          LV {creature.level} · {creature.difficulty.toUpperCase()}
        </div>

        <div className="encounter-card-mini">
          <div className="ph">
            <div className="pa"><i className="fa-solid fa-dragon" /></div>
            <div>
              <div className="pn">{creature.name}</div>
              <div className="pl" style={{ color: badgeColor }}>LV {creature.level} · {creature.difficulty.toUpperCase()}</div>
            </div>
          </div>
          <div className="pb">
            <div className="br">
              <span className="lbl"><i className="fa-solid fa-heart" /></span>
              <div className="bb"><div className="bf hp" style={{ width: `${hpPercent}%` }} /></div>
              <span className="bt">{creature.hp.current}/{creature.hp.max}</span>
            </div>
          </div>
          <div className="ps">
            <span><i className="fa-solid fa-crosshairs" /> ATK {creature.attack}</span>
            <span><i className="fa-solid fa-shield-halved" /> DEF {creature.defense}</span>
          </div>
        </div>

        <div className="encounter-actions">
          <button className="enc-eng" onClick={handleEngage}>
            Engage <span className="key-hint">[Enter]</span>
          </button>
          <button className="enc-back" onClick={handleBackAway}>
            Back away <span className="key-hint">[Esc]</span>
          </button>
        </div>
      </div>
    </div>
  )
}

export default EncounterPrompt
