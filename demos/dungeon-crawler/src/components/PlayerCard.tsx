import { useState, useEffect } from 'react'
import { getGameState } from '../GameState'
import type { PlayerState } from '../types'
import './PlayerCard.css'

function PlayerCard() {
  const [player, setPlayer] = useState<PlayerState>(() => getGameState().player)
  const [visible, setVisible] = useState(false)

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

  if (!visible) return null

  const hpPercent = player.hp.max > 0 ? (player.hp.current / player.hp.max) * 100 : 0
  const mpPercent = player.mp.max > 0 ? (player.mp.current / player.mp.max) * 100 : 0
  const xpPercent = player.xp.next > 0 ? (player.xp.current / player.xp.next) * 100 : 0

  function handleClick() {
    getGameState().emitInventoryRequested()
  }

  return (
    <div id="player-card" onClick={handleClick}>
      <div className="ph">
        <div className="pa"><i className="fa-solid fa-user" /></div>
        <div>
          <div className="pn">{player.name}</div>
          <div className="pl">LV {player.level}</div>
        </div>
      </div>
      <div className="pb">
        <div className="br">
          <span className="lbl"><i className="fa-solid fa-heart" /></span>
          <div className="bb"><div className="bf hp" style={{ width: `${hpPercent}%` }} /></div>
          <span className="bt">{player.hp.current}/{player.hp.max}</span>
        </div>
        <div className="br">
          <span className="lbl"><i className="fa-solid fa-star" /></span>
          <div className="bb"><div className="bf mp" style={{ width: `${mpPercent}%` }} /></div>
          <span className="bt">{player.mp.current}/{player.mp.max}</span>
        </div>
      </div>
      <div className="ps">
        <span><i className="fa-solid fa-crosshairs" /> ATK {player.attack}</span>
        <span><i className="fa-solid fa-shield-halved" /> DEF {player.defense}</span>
      </div>
      <div className="xp-row">
        <div className="xp-bar">
          <div className="xp-fill" style={{ width: `${xpPercent}%` }} />
        </div>
        <span className="xp-text">XP {player.xp.current}/{player.xp.next}</span>
      </div>
    </div>
  )
}

export default PlayerCard
