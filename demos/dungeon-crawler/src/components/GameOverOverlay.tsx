import { useState, useEffect, useCallback, useRef } from 'react'
import { getGameState } from '../GameState'
import './GameOverOverlay.css'

function formatTime(ms: number): string {
  const totalSeconds = Math.floor(ms / 1000)
  const minutes = Math.floor(totalSeconds / 60)
  const seconds = totalSeconds % 60
  return `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`
}

interface GameOverOverlayProps {
  onPlayAgain?: () => void
  onQuit?: () => void
}

function GameOverOverlay({ onPlayAgain, onQuit }: GameOverOverlayProps) {
  const [visible, setVisible] = useState(false)
  const [level, setLevel] = useState(1)
  const [rooms, setRooms] = useState(0)
  const [enemies, setEnemies] = useState(0)
  const [timeSurvived, setTimeSurvived] = useState('00:00')
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null)

  useEffect(() => {
    const gs = getGameState()

    function onGameOver() {
      setLevel(gs.player.level)
      setRooms(gs.visitedRooms.size)
      setEnemies(gs.enemiesSlain)
      setVisible(true)

      const elapsed = Date.now() - gs.gameStartTime
      setTimeSurvived(formatTime(elapsed))

      timerRef.current = setInterval(() => {
        const gs2 = getGameState()
        const elapsed2 = Date.now() - gs2.gameStartTime
        setTimeSurvived(formatTime(elapsed2))
      }, 1000)
    }

    function onGameRestarted() {
      setVisible(false)
      if (timerRef.current) {
        clearInterval(timerRef.current)
        timerRef.current = null
      }
    }

    function onGameQuit() {
      setVisible(false)
      if (timerRef.current) {
        clearInterval(timerRef.current)
        timerRef.current = null
      }
    }

    gs.on('game:over', onGameOver)
    gs.on('game:restarted', onGameRestarted)
    gs.on('game:quit', onGameQuit)

    return () => {
      gs.off('game:over', onGameOver)
      gs.off('game:restarted', onGameRestarted)
      gs.off('game:quit', onGameQuit)
      if (timerRef.current) clearInterval(timerRef.current)
    }
  }, [])

  const handlePlayAgain = useCallback(() => {
    const gs = getGameState()
    gs.restartGame()
    if (onPlayAgain) onPlayAgain()
  }, [onPlayAgain])

  const handleQuit = useCallback(() => {
    getGameState().quitToTitle()
    if (onQuit) onQuit()
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
    <div className="gameover-overlay" data-testid="gameover-overlay">
      <div className="gameover-bg" />
      <div className="gameover-content">
        <h1 className="gameover-title">GAME OVER</h1>

        <div className="gameover-stats">
          <div className="gameover-stat">
            <span className="stat-label">Level Reached</span>
            <span className="stat-value">{level}</span>
          </div>
          <div className="gameover-stat">
            <span className="stat-label">Rooms Explored</span>
            <span className="stat-value">{rooms}</span>
          </div>
          <div className="gameover-stat">
            <span className="stat-label">Enemies Slain</span>
            <span className="stat-value">{enemies}</span>
          </div>
          <div className="gameover-stat">
            <span className="stat-label">Time Survived</span>
            <span className="stat-value">{timeSurvived}</span>
          </div>
        </div>

        <div className="gameover-actions">
          <button className="go-play" onClick={handlePlayAgain}>
            Play Again <span className="key-hint">[Enter]</span>
          </button>
          <button className="go-quit" onClick={handleQuit}>
            Quit <span className="key-hint">[Q]</span>
          </button>
        </div>
      </div>
    </div>
  )
}

export default GameOverOverlay
