import { useState, useCallback, useEffect } from 'react'
import { GameComponent } from './game/GameComponent'
import { getGameState } from './GameState'
import { DungeonScene } from './game/DungeonScene'
import ActivityLog from './components/ActivityLog'
import MiniMap from './components/MiniMap'
import BottomToolbar from './components/BottomToolbar'
import CombatOverlay from './components/CombatOverlay'
import { useBreakpoint } from './hooks/useBreakpoint'
import './styles/responsive.css'

function App() {
  const [logExpanded, setLogExpanded] = useState(false)
  const bp = useBreakpoint()

  const toggleLog = useCallback(() => {
    setLogExpanded(prev => !prev)
  }, [])

  const isMobile = bp === 'mobile'

  useEffect(() => {
    const gs = getGameState()

    function onEncounterStarted(creatureId: string) {
      const scene = DungeonScene.currentInstance
      if (!scene) return
      const creature = scene.findCreatureById(creatureId)
      if (!creature) return
      gs.startCombat(creature)
    }

    gs.on('encounter:started', onEncounterStarted)
    return () => { gs.off('encounter:started', onEncounterStarted) }
  }, [])

  return (
    <>
      <GameComponent />
      <ActivityLog
        expanded={isMobile ? logExpanded : undefined}
        onToggle={isMobile ? toggleLog : undefined}
      />
      <MiniMap />
      <BottomToolbar onToggleLog={toggleLog} />
      <CombatOverlay />
    </>
  )
}

export default App
