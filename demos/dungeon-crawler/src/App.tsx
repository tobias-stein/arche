import { useState, useCallback } from 'react'
import { GameComponent } from './game/GameComponent'
import ActivityLog from './components/ActivityLog'
import BottomToolbar from './components/BottomToolbar'
import { useBreakpoint } from './hooks/useBreakpoint'
import './styles/responsive.css'

function App() {
  const [logExpanded, setLogExpanded] = useState(false)
  const bp = useBreakpoint()

  const toggleLog = useCallback(() => {
    setLogExpanded(prev => !prev)
  }, [])

  const isMobile = bp === 'mobile'

  return (
    <>
      <GameComponent />
      <ActivityLog
        expanded={isMobile ? logExpanded : undefined}
        onToggle={isMobile ? toggleLog : undefined}
      />
      <BottomToolbar onToggleLog={toggleLog} />
    </>
  )
}

export default App
