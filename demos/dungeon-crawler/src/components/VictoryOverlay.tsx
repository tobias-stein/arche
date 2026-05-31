import { useState, useEffect, useCallback } from 'react'
import { getGameState } from '../GameState'
import './VictoryOverlay.css'

interface VictoryOverlayProps {
  onPlayAgain?: () => void
  onQuit?: () => void
}

function VictoryOverlay({ onPlayAgain, onQuit }: VictoryOverlayProps) {
  const [visible, setVisible] = useState(false)
  const [level, setLevel] = useState(1)
  const [rooms, setRooms] = useState(0)
  const [enemies, setEnemies] = useState(0)
  const [items, setItems] = useState(0)

  useEffect(() => {
    const gs = getGameState()

    function onGameVictory() {
      setLevel(gs.player.level)
      setRooms(gs.visitedRooms.size)
      setEnemies(gs.enemiesSlain)
      setItems(gs.itemsCollected)
      setVisible(true)
    }

    function onGameRestarted() {
      setVisible(false)
    }

    function onGameQuit() {
      setVisible(false)
    }

    gs.on('game:victory', onGameVictory)
    gs.on('game:restarted', onGameRestarted)
    gs.on('game:quit', onGameQuit)

    return () => {
      gs.off('game:victory', onGameVictory)
      gs.off('game:restarted', onGameRestarted)
      gs.off('game:quit', onGameQuit)
    }
  }, [])

  const handlePlayAgain = useCallback(() => {
    onPlayAgain?.()
  }, [onPlayAgain])

  const handleQuit = useCallback(() => {
    onQuit?.()
  }, [onQuit])

  useEffect(() => {
    if (!visible) return

    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === 'Enter') {
        e.preventDefault()
        handlePlayAgain()
      }
      if (e.key === 'q' || e.key === 'Q') {
        e.preventDefault()
        handleQuit()
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [visible, handlePlayAgain, handleQuit])

  if (!visible) return null

  return (
    <div className="victory-overlay" data-testid="victory-overlay">
      <div className="victory-bg" />
      <div className="victory-content">
        <h1 className="victory-title">YOU WIN!</h1>

        <div className="victory-stats">
          <div className="victory-stat">
            <span className="stat-label">Final Level</span>
            <span className="stat-value">{level}</span>
          </div>
          <div className="victory-stat">
            <span className="stat-label">Rooms Explored</span>
            <span className="stat-value">{rooms}</span>
          </div>
          <div className="victory-stat">
            <span className="stat-label">Enemies Slain</span>
            <span className="stat-value">{enemies}</span>
          </div>
          <div className="victory-stat">
            <span className="stat-label">Items Collected</span>
            <span className="stat-value">{items}</span>
          </div>
        </div>

        <div className="victory-actions">
          <button className="vic-play" onClick={handlePlayAgain}>
            Play Again <span className="key-hint">[Enter]</span>
          </button>
          <button className="vic-quit" onClick={handleQuit}>
            Quit <span className="key-hint">[Q]</span>
          </button>
        </div>
      </div>
    </div>
  )
}

export default VictoryOverlay
