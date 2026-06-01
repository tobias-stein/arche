import { useState, useEffect, useCallback, useRef } from 'react'
import { getGameState } from '../GameState'
import { calculatePlayerDamage, calculateCreatureDamage } from '../game/combat'
import type { CreatureState, ItemState } from '../types'
import EnemyCard from './EnemyCard'
import ActionMenu from './ActionMenu'
import SpellPanel from './SpellPanel'
import UseItemPanel from './UseItemPanel'
import FleePanel from './FleePanel'
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
  const [panel, setPanel] = useState<'spell' | 'item' | 'flee' | null>(null)
  const [spellbook, setSpellbook] = useState<(ItemState | null)[]>([])
  const [inventory, setInventory] = useState<(ItemState | null)[]>([])
  const [playerMp, setPlayerMp] = useState(0)
  const enemyTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  const syncState = useCallback(() => {
    const gs = getGameState()
    setSpellbook([...gs.spellbook])
    setInventory([...gs.inventory])
    setPlayerMp(gs.player.mp.current)
  }, [])

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
      setPanel(null)
      syncState()

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
        syncState()
      }
    }

    function onVictory(_creatureId: string, xp: number) {
      setVictory(true)
      setTurn(null)
      setShowMenu(false)
      setPanel(null)
      setXpGained(xp)
      setFlash(true)
      setTimeout(() => setFlash(false), 400)
    }

    function onDefeat() {
      setDefeat(true)
      setTurn(null)
      setShowMenu(false)
      setPanel(null)

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
      setPanel(null)
    }

    function onFled() {
      setVisible(false)
      setCreature(null)
      setPanel(null)
    }

    function onStatsChanged() {
      syncState()
    }

    gs.on('combat:started', onCombatStarted)
    gs.on('combat:creature-damaged', onCreatureDamaged)
    gs.on('combat:turn-changed', onTurnChanged)
    gs.on('combat:victory', onVictory)
    gs.on('combat:defeat', onDefeat)
    gs.on('combat:ended', onCombatEnded)
    gs.on('combat:fled', onFled)
    gs.on('player:stats-changed', onStatsChanged)
    gs.on('inventory:changed', onStatsChanged)

    return () => {
      gs.off('combat:started', onCombatStarted)
      gs.off('combat:creature-damaged', onCreatureDamaged)
      gs.off('combat:turn-changed', onTurnChanged)
      gs.off('combat:victory', onVictory)
      gs.off('combat:defeat', onDefeat)
      gs.off('combat:ended', onCombatEnded)
      gs.off('combat:fled', onFled)
      gs.off('player:stats-changed', onStatsChanged)
      gs.off('inventory:changed', onStatsChanged)
      if (enemyTimerRef.current) clearTimeout(enemyTimerRef.current)
    }
  }, [syncState])

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
    syncState()
    setPanel('spell')
  }, [syncState])

  const handleUseItem = useCallback(() => {
    syncState()
    setPanel('item')
  }, [syncState])

  const handleFlee = useCallback(() => {
    setPanel('flee')
  }, [])

  const handleClosePanel = useCallback(() => {
    setPanel(null)
  }, [])

  const endPlayerTurn = useCallback(() => {
    setShowMenu(false)
    setPanel(null)
    enemyTimerRef.current = setTimeout(() => {
      const gs = getGameState()
      if (!gs.combatCreature) return
      const enemyDamage = calculateCreatureDamage(gs.player, gs.combatCreature)
      gs.processEnemyTurn(enemyDamage)
    }, 1000)
  }, [])

  const handleCastSpellSelect = useCallback(async (spell: ItemState) => {
    const gs = getGameState()
    if (!gs.combatCreature || gs.combatTurn !== 'player') return

    const manaCost = spell.stats.mana_cost ?? 0
    if (gs.player.mp.current < manaCost) return

    gs.setPlayerMp(gs.player.mp.current - manaCost)

    const damage = spell.stats.damage ?? 0
    const heal = spell.stats.heal ?? 0

    if (damage > 0) {
      setPanel(null)
      setShowMenu(false)
      gs.addLogEntry({
        type: 'spell_cast',
        message: `Cast ${spell.name} for ${damage} damage!`,
        icon: 'fa-solid fa-wand-sparkles',
      })
      await gs.processPlayerAttack(damage)
      if (gs.combatCreature.hp.current > 0) {
        enemyTimerRef.current = setTimeout(() => {
          const gs2 = getGameState()
          if (!gs2.combatCreature) return
          const enemyDamage = calculateCreatureDamage(gs2.player, gs2.combatCreature)
          gs2.processEnemyTurn(enemyDamage)
        }, 1000)
      }
    } else if (heal > 0) {
      gs.setPlayerHp(gs.player.hp.current + heal)
      gs.addLogEntry({
        type: 'spell_cast',
        message: `Cast ${spell.name}, healed for ${heal}!`,
        icon: 'fa-solid fa-wand-sparkles',
      })
      endPlayerTurn()
    }
  }, [endPlayerTurn])

  const handleUseItemSelect = useCallback((item: ItemState, idx: number) => {
    const gs = getGameState()
    if (!gs.combatCreature || gs.combatTurn !== 'player') return

    const healVal = item.stats.heal ?? 0
    const manaVal = item.stats.mana ?? 0

    if (healVal > 0) {
      gs.setPlayerHp(gs.player.hp.current + healVal)
    }
    if (manaVal > 0) {
      gs.setPlayerMp(gs.player.mp.current + manaVal)
    }

    gs.dropItemFromInventory(idx)
    gs.addLogEntry({
      type: 'potion_consumed',
      message: `Used ${item.name}${healVal > 0 ? `, restored ${healVal} HP` : ''}${manaVal > 0 ? `, restored ${manaVal} MP` : ''}`,
      icon: 'fa-solid fa-flask',
    })

    endPlayerTurn()
  }, [endPlayerTurn])

  const handleFleeConfirm = useCallback(() => {
    const gs = getGameState()
    setPanel(null)
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

      {panel === 'spell' && (
        <SpellPanel
          spells={spellbook}
          playerMp={playerMp}
          onSelect={handleCastSpellSelect}
          onClose={handleClosePanel}
        />
      )}

      {panel === 'item' && (
        <UseItemPanel
          inventory={inventory}
          onSelect={handleUseItemSelect}
          onClose={handleClosePanel}
        />
      )}

      {panel === 'flee' && (
        <FleePanel
          onFlee={handleFleeConfirm}
          onClose={handleClosePanel}
        />
      )}

      {showMenu && turn === 'player' && !victory && !defeat && !panel && (
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
