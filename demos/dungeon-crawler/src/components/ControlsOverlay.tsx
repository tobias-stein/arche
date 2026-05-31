import { useState, useEffect, useCallback } from 'react'
import { getGameState } from '../GameState'
import './ControlsOverlay.css'

const BINDINGS_LEFT = [
  { keys: ['W', 'A', 'S', 'D'], label: 'Move' },
  { keys: ['Arrows'], label: 'Move' },
  { keys: ['I'], label: 'Equipment' },
  { keys: ['M'], label: 'Map' },
]

const BINDINGS_RIGHT = [
  { keys: ['Enter'], label: 'Confirm / Interact' },
  { keys: ['Esc'], label: 'Back / Cancel' },
  { keys: ['L'], label: 'Toggle Log' },
  { keys: ['H', '?'], label: 'Help / Controls' },
  { keys: ['Q'], label: 'Quit to Title' },
]

function ControlsOverlay() {
  const [visible, setVisible] = useState(false)

  useEffect(() => {
    const gs = getGameState()

    function onControlsVisible(v: boolean) {
      setVisible(v)
    }

    gs.on('controls:visible', onControlsVisible)

    return () => {
      gs.off('controls:visible', onControlsVisible)
    }
  }, [])

  const close = useCallback(() => {
    getGameState().setControlsVisible(false)
  }, [])

  if (!visible) return null

  return (
    <div className="controls-overlay" data-testid="controls-overlay" onClick={close}>
      <button className="controls-close" onClick={close} aria-label="Close controls">
        <i className="fa-solid fa-xmark" />
      </button>
      <h2 className="controls-title">CONTROLS</h2>
      <div className="controls-grid">
        <div className="controls-col">
          {BINDINGS_LEFT.map((binding, i) => (
            <div key={i} className="controls-binding">
              {binding.keys.map(k => <kbd key={k}>{k}</kbd>)}
              <span className="binding-label">{binding.label}</span>
            </div>
          ))}
        </div>
        <div className="controls-col">
          {BINDINGS_RIGHT.map((binding, i) => (
            <div key={i} className="controls-binding">
              {binding.keys.map(k => <kbd key={k}>{k}</kbd>)}
              <span className="binding-label">{binding.label}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}

export default ControlsOverlay
