import { useState, useEffect, useCallback, useRef } from 'react'
import { getGameState } from '../GameState'
import { calculatePlayerDamage, calculateCreatureDamage } from '../game/combat'
import type { CreatureState } from '../types'
import EnemyCard from './EnemyCard'
import ActionMenu from './ActionMenu'
import './CombatOverlay.css'

function CombatOverlay() {
  const [creature, setCreature] = useState<CreatureState | null>(null)
  const [turn, setTurn] = useState<'player' | 'enemy' | null>(null)
  const [victory, setVictory] = useState(false)
  const [defeat, setDefeat] = useState(false)
  const [visible, setVisible] = useState(false)
  const [flash, setFlash] = useState(false)
  const [animating, setAnimating] = useState(false)
  const [showMenu, setShowMenu] = useState(false)
  const [xpGained, setXpGained] = useState(0)
  const enemyTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  useEffect(() => {
    const gs = getGameState()

    function onCombatStarted(c: CreatureState) {
      setCreature(c)
      setTurn('player')
      setVictory(false)
      setDefeat(false)
      setXpGained(0)
      setVisible(true)
      setAnimating(true)
      setShowMenu(false)

      setTimeout(() => {
        setAnimating(false)
        setShowMenu(true)
      }, 500)
    }

    function onCreatureDamaged(_damage: number, c: CreatureState) {
      setCreature({ ...c })
    }

    function onTurnChanged(t: unknown) {
      setTurn(t as 'player' | 'enemy')
      if (t === 'player') {
        setShowMenu(true)
      }
    }

    function onVictory(_creatureId: string, xp: number) {
      setVictory(true)
      setTurn(null)
      setShowMenu(false)
      setXpGained(xp)
      setFlash(true)
      setTimeout(() => setFlash(false), 400)
    }

    function onDefeat() {
      setDefeat(true)
      setTurn(null)
      setShowMenu(false)

      enemyTimerRef.current = setTimeout(() => {
        gs.endCombat()
      }, 2000)
    }

    function onCombatEnded() {
      setVisible(false)
      setCreature(null)
      setTurn(null)
      setVictory(false)
      setDefeat(false)
      setShowMenu(false)
    }

    function onFled() {
      setVisible(false)
      setCreature(null)
    }

    gs.on('combat:started', onCombatStarted)
    gs.on('combat:creature-damaged', onCreatureDamaged)
    gs.on('combat:turn-changed', onTurnChanged)
    gs.on('combat:victory', onVictory)
    gs.on('combat:defeat', onDefeat)
    gs.on('combat:ended', onCombatEnded)
    gs.on('combat:fled', onFled)

    return () => {
      gs.off('combat:started', onCombatStarted)
      gs.off('combat:creature-damaged', onCreatureDamaged)
      gs.off('combat:turn-changed', onTurnChanged)
      gs.off('combat:victory', onVictory)
      gs.off('combat:defeat', onDefeat)
      gs.off('combat:ended', onCombatEnded)
      gs.off('combat:fled', onFled)
      if (enemyTimerRef.current) clearTimeout(enemyTimerRef.current)
    }
  }, [])

  const handleAttack = useCallback(async () => {
    const gs = getGameState()
    if (!gs.combatCreature || gs.combatTurn !== 'player') return

    setShowMenu(false)
    const damage = calculatePlayerDamage(gs.player, gs.combatCreature)
    await gs.processPlayerAttack(damage)

    if (gs.combatCreature.hp.current > 0) {
      enemyTimerRef.current = setTimeout(() => {
        const gs2 = getGameState()
        if (!gs2.combatCreature) return
        const enemyDamage = calculateCreatureDamage(gs2.player, gs2.combatCreature)
        gs2.processEnemyTurn(enemyDamage)
      }, 1000)
    }
  }, [])

  const handleCastSpell = useCallback(() => {
    alert('Cast Spell — will be wired in slice 9')
  }, [])

  const handleUseItem = useCallback(() => {
    alert('Use Item — will be wired in slice 9')
  }, [])

  const handleFlee = useCallback(() => {
    const gs = getGameState()
    gs.fleeCombat()
  }, [])

  if (!visible || !creature) return null

  return (
    <div className={`combat-overlay ${visible ? 'visible' : ''}`}>
      <div className="combat-bg">
        <div className="combat-floor" />
        <div className="combat-vs"><i className="fa-solid fa-crosshairs" /> VS <i className="fa-solid fa-crosshairs" /></div>
      </div>

      <div className={`combat-flash ${flash ? 'active' : ''}`} />

      {animating && (
        <div className="combat-encounter-text">
          <span>A wild <strong>{creature.name}</strong> appears!</span>
        </div>
      )}

      <EnemyCard creature={creature} />

      {showMenu && turn === 'player' && !victory && !defeat && (
        <ActionMenu
          onAttack={handleAttack}
          onCastSpell={handleCastSpell}
          onUseItem={handleUseItem}
          onFlee={handleFlee}
        />
      )}

      {turn === 'enemy' && !victory && !defeat && (
        <div className="combat-enemy-turn">
          <span>Enemy turn...</span>
        </div>
      )}

      {victory && (
        <div className="combat-victory">
          <div className="cv-content">
            <h3><i className="fa-solid fa-crosshairs" /> Victory!</h3>
            <p className="cv-xp">+{xpGained} XP</p>
          </div>
        </div>
      )}

      {defeat && (
        <div className="combat-defeat">
          <div className="cd-content">
            <h3>Defeated</h3>
            <p>Game Over</p>
          </div>
        </div>
      )}
    </div>
  )
}

export default CombatOverlay
