import { useEffect, useRef } from 'react'
import { History, X } from 'lucide-react'
import { useUi } from '@/stores/ui'

export function ActivityPanel() {
  const { closePanel } = useUi()
  const panelRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (panelRef.current && !panelRef.current.contains(event.target as Node)) {
        closePanel()
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [closePanel])

  return (
    <div
      ref={panelRef}
      className="fixed top-16 right-0 bottom-0 z-30 w-80 border-l bg-background shadow-lg animate-in slide-in-from-right"
    >
      <div className="flex items-center justify-between px-4 h-12 border-b">
        <div className="flex items-center gap-2 text-sm font-medium">
          <History className="h-4 w-4" />
          Activity
        </div>
        <button
          onClick={closePanel}
          className="inline-flex items-center justify-center h-8 w-8 rounded-md hover:bg-accent hover:text-accent-foreground"
          aria-label="Close activity panel"
        >
          <X className="h-4 w-4" />
        </button>
      </div>
      <div className="p-4 text-sm text-muted-foreground">
        Activity log will appear here.
      </div>
    </div>
  )
}
