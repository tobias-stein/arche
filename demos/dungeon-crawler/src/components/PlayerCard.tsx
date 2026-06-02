import { useState, useEffect, useCallback } from 'react'
import { getGameState } from '../GameState'
import type { PlayerState } from '../types'
import { ceil } from '../utils/format'
import { useBreakpoint } from '../hooks/useBreakpoint'
import './PlayerCard.css'

function PlayerCard() {
  const [player, setPlayer] = useState<PlayerState>(() => getGameState().player)
  const [visible, setVisible] = useState(false)
  const [mobileExpanded, setMobileExpanded] = useState(false)
  const bp = useBreakpoint()

  useEffect(() => {
    const gs = getGameState()

    function onStatsChanged(p: PlayerState) {
      setPlayer({ ...p })
    }

    function onGameStarted() {
      setVisible(true)
    }

    if (gs.gameStarted) setVisible(true)

    gs.on('player:stats-changed', onStatsChanged)
    gs.on('game:started', onGameStarted)
    return () => {
      gs.off('player:stats-changed', onStatsChanged)
      gs.off('game:started', onGameStarted)
    }
  }, [])

  const handleClick = useCallback(() => {
    const gs = getGameState()
    if (gs.encounterActive || gs.combatActive || gs.gameOver || gs.victory) return
    if (bp === 'mobile') {
      setMobileExpanded(prev => !prev)
    } else {
      gs.emitInventoryRequested()
    }
  }, [bp])

  if (!visible) return null

  const hpPercent = player.hp.max > 0 ? (player.hp.current / player.hp.max) * 100 : 0
  const mpPercent = player.mp.max > 0 ? (player.mp.current / player.mp.max) * 100 : 0
  const xpPercent = player.xp.next > 0 ? (player.xp.current / player.xp.next) * 100 : 0

  const isMobile = bp === 'mobile'
  const isTablet = bp === 'tablet'

  return (
    <div
      id="player-card"
      className={`${isMobile ? 'mobile' : ''} ${isTablet ? 'tablet' : ''} ${isMobile && mobileExpanded ? 'expanded' : ''}`}
      onClick={handleClick}
    >
      <div className="ph">
        <div className="pa"><i className="fa-solid fa-user" /></div>
        <div>
          <div className="pn">{player.name}</div>
          <div className="pl">LV {ceil(player.level)}</div>
        </div>
      </div>
      {(!isMobile || mobileExpanded) && (
        <>
          <div className="pb">
            <div className="br">
              <span className="lbl"><i className="fa-solid fa-heart" /></span>
              <div className="bb"><div className="bf hp" style={{ width: `${hpPercent}%` }} /></div>
              <span className="bt">{ceil(player.hp.current)}/{ceil(player.hp.max)}</span>
            </div>
            <div className="br">
              <span className="lbl"><i className="fa-solid fa-star" /></span>
              <div className="bb"><div className="bf mp" style={{ width: `${mpPercent}%` }} /></div>
              <span className="bt">{ceil(player.mp.current)}/{ceil(player.mp.max)}</span>
            </div>
          </div>
          <div className="ps">
            <span><i className="fa-solid fa-crosshairs" /> ATK {ceil(player.attack)}</span>
            <span><i className="fa-solid fa-shield-halved" /> DEF {ceil(player.defense)}</span>
          </div>
          <div className="xp-row">
            <div className="xp-bar">
              <div className="xp-fill" style={{ width: `${xpPercent}%` }} />
            </div>
            <span className="xp-text">XP {ceil(player.xp.current)}/{ceil(player.xp.next)}</span>
          </div>
        </>
      )}
      {isMobile && !mobileExpanded && (
        <div className="mobile-expand-hint">
          <i className="fa-solid fa-chevron-down" />
        </div>
      )}
    </div>
  )
}

export default PlayerCard
