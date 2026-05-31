import { useState, useEffect, useCallback, useRef } from 'react'
import { getGameState } from '../GameState'
import './TitleScreen.css'

interface TitleScreenProps {
  onStart: () => void
}

function TitleScreen({ onStart }: TitleScreenProps) {
  const [visible, setVisible] = useState(true)
  const [fading, setFading] = useState(false)
  const fadingRef = useRef(false)

  useEffect(() => {
    const gs = getGameState()

    function onGameQuit() {
      setVisible(true)
      setFading(false)
      fadingRef.current = false
    }

    gs.on('game:quit', onGameQuit)

    return () => {
      gs.off('game:quit', onGameQuit)
    }
  }, [])

  const handleStart = useCallback(() => {
    if (fadingRef.current) return
    fadingRef.current = true
    setFading(true)
    setTimeout(() => {
      setVisible(false)
      onStart()
    }, 350)
  }, [onStart])

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === 'Enter') handleStart()
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [handleStart])

  if (!visible) return null

  return (
    <div className={`title-screen${fading ? ' title-screen--fading' : ''}`} onClick={handleStart}>
      <div className="title-screen__bg" />
      <h1 className="title-screen__title">Pixel Quest</h1>
      <p className="title-screen__subtitle">A Retro Roguelike Adventure</p>
      <div className="title-screen__prompt">
        <i className="fa-solid fa-play" />  PRESS ENTER TO START
      </div>
    </div>
  )
}

export default TitleScreen
