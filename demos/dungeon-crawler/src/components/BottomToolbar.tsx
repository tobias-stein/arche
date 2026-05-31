import { useCallback } from 'react'
import { getGameState } from '../GameState'
import './BottomToolbar.css'

interface BottomToolbarProps {
  onToggleLog: () => void
}

function BottomToolbar({ onToggleLog }: BottomToolbarProps) {
  const handleInventory = useCallback(() => {
    getGameState().emitInventoryRequested()
  }, [])

  const handleMap = useCallback(() => {
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'M' }))
  }, [])

  const handleLog = useCallback(() => {
    onToggleLog()
  }, [onToggleLog])

  const handleHelp = useCallback(() => {
    window.dispatchEvent(new KeyboardEvent('keydown', { key: '?' }))
  }, [])

  return (
    <div id="bottom-toolbar" data-testid="bottom-toolbar">
      <button className="tb-btn" onClick={handleInventory} aria-label="Inventory">
        <i className="fa-solid fa-suitcase" />
      </button>
      <button className="tb-btn" onClick={handleMap} aria-label="Map">
        <i className="fa-solid fa-map" />
      </button>
      <button className="tb-btn" onClick={handleLog} aria-label="Log">
        <i className="fa-solid fa-scroll" />
      </button>
      <button className="tb-btn" onClick={handleHelp} aria-label="Help">
        <i className="fa-solid fa-question" />
      </button>
    </div>
  )
}

export default BottomToolbar
