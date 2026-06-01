import { useState, useCallback, useEffect } from 'react'
import { GameComponent } from './game/GameComponent'
import { getGameState } from './GameState'
import { DungeonScene } from './game/DungeonScene'
import TitleScreen from './components/TitleScreen'
import ActivityLog from './components/ActivityLog'
import MiniMap from './components/MiniMap'
import BottomToolbar from './components/BottomToolbar'
import CombatOverlay from './components/CombatOverlay'
import EncounterPrompt from './components/EncounterPrompt'
import GameOverOverlay from './components/GameOverOverlay'
import VictoryOverlay from './components/VictoryOverlay'
import ControlsOverlay from './components/ControlsOverlay'
import LootPopup from './components/LootPopup'
import InventoryDialog from './components/InventoryDialog'
import PlayerCard from './components/PlayerCard'
import { useBreakpoint } from './hooks/useBreakpoint'
import './styles/responsive.css'

function App() {
  const [logExpanded, setLogExpanded] = useState(false)
  const [showInventory, setShowInventory] = useState(false)
  const [lootVisible, setLootVisible] = useState(false)
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
      gs.showEncounterPrompt(creature)
    }

    function onEnemySlain() {
      gs.incrementEnemiesSlain()
    }

    function onInventoryRequested() {
      setShowInventory(true)
    }

    function onLootShow() {
      setLootVisible(true)
    }

    function onLootOrChestClosed() {
      setLootVisible(false)
    }

    gs.on('encounter:started', onEncounterStarted)
    gs.on('combat:victory', onEnemySlain)
    gs.on('inventory:requested', onInventoryRequested)
    gs.on('loot:show', onLootShow)
    gs.on('chest:loot-show', onLootShow)
    gs.on('combat:ended', onLootOrChestClosed)
    gs.on('chest:loot-dismissed', onLootOrChestClosed)

    return () => {
      gs.off('encounter:started', onEncounterStarted)
      gs.off('combat:victory', onEnemySlain)
      gs.off('inventory:requested', onInventoryRequested)
      gs.off('loot:show', onLootShow)
      gs.off('chest:loot-show', onLootShow)
      gs.off('combat:ended', onLootOrChestClosed)
      gs.off('chest:loot-dismissed', onLootOrChestClosed)
    }
  }, [])

  const handleStart = useCallback(() => {
    getGameState().startGame()
  }, [])

  const handleCloseInventory = useCallback(() => {
    setShowInventory(false)
  }, [])

  const handleRestartDungeon = useCallback(() => {
    const scene = DungeonScene.currentInstance
    if (scene) {
      scene.generateDungeon()
    }
  }, [])

  const handlePlayAgain = useCallback(() => {
    const gs = getGameState()
    gs.restartGame()
    handleRestartDungeon()
    gs.startGame()
  }, [handleRestartDungeon])

  const handleQuit = useCallback(() => {
    getGameState().quitToTitle()
  }, [])

  useEffect(() => {
    function isHelpKey(e: KeyboardEvent): boolean {
      return e.key === 'h' || e.key === 'H' || e.key === '?' || e.key === '/'
    }

    function handleKeyDown(e: KeyboardEvent) {
      const gs = getGameState()
      if (gs.encounterActive || gs.combatActive || gs.gameOver || gs.victory) return
      if (lootVisible) return

      if (e.key === 'i' || e.key === 'I') {
        e.preventDefault()
        setShowInventory(prev => !prev)
        return
      }

      if (e.key === 'Escape') {
        if (showInventory) {
          setShowInventory(false)
          return
        }
      }

      if (isHelpKey(e)) {
        e.preventDefault()
        gs.controlsVisible ? gs.setControlsVisible(false) : gs.toggleControls()
        return
      }

      if (gs.controlsVisible) {
        gs.setControlsVisible(false)
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [showInventory, lootVisible])

  return (
    <>
      <TitleScreen onStart={handleStart} />
      <PlayerCard />
      <GameComponent />
      <ActivityLog
        expanded={isMobile ? logExpanded : undefined}
        onToggle={isMobile ? toggleLog : undefined}
      />
      <MiniMap />
      <BottomToolbar onToggleLog={toggleLog} />
      <EncounterPrompt />
      <CombatOverlay />
      <GameOverOverlay
        onPlayAgain={handlePlayAgain}
        onQuit={handleQuit}
      />
      <VictoryOverlay
        onPlayAgain={handlePlayAgain}
        onQuit={handleQuit}
      />
      <ControlsOverlay />
      <LootPopup inventoryOpen={showInventory && lootVisible} />
      {showInventory && <InventoryDialog onClose={handleCloseInventory} lootActive={lootVisible} />}
    </>
  )
}

export default App
