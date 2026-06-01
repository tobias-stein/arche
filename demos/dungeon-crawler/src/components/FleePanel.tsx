import { useEffect } from 'react'

interface FleePanelProps {
  onFlee: () => void
  onClose: () => void
}

function FleePanel({ onFlee, onClose }: FleePanelProps) {
  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      switch (e.key) {
        case 'Escape':
          onClose()
          e.preventDefault()
          break
        case 'Enter':
          onFlee()
          e.preventDefault()
          break
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [onFlee, onClose])

  return (
    <div className="combat-panel-overlay" onClick={onClose}>
      <div className="combat-panel flee-panel" onClick={e => e.stopPropagation()}>
        <div className="cp-header">
          <span>Flee from combat?</span>
          <button className="cp-close" onClick={onClose} aria-label="Close">×</button>
        </div>
        <p className="flee-sub">Cowardice has its rewards.</p>
        <div className="flee-buttons">
          <button className="flee-yes" onClick={onFlee}>Yes, Flee</button>
          <button className="flee-no" onClick={onClose}>Stay &amp; Fight</button>
        </div>
      </div>
    </div>
  )
}

export default FleePanel
