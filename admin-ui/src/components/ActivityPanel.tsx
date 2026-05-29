import { useEffect, useRef } from 'react'
import { useNavigate, useLocation } from 'react-router-dom'
import { History, X } from 'lucide-react'
import { useQueryClient } from '@tanstack/react-query'
import { useUi } from '@/stores/ui'
import { useAuditLog } from '@/api/generated/hooks'
import { cn, relativeTime } from '@/lib/utils'
import type { AuditAction, AuditLogEntry } from '@/api/generated/types'

function actionBadgeVariant(action: AuditAction): string {
  switch (action) {
    case 'created':
      return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-100'
    case 'updated':
      return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-100'
    case 'deleted':
    case 'forceDeleted':
      return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-100'
    case 'adjusted':
      return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-100'
  }
}

function resourceDetailPath(resourceType: string, resourceId: string): string | null {
  switch (resourceType) {
    case 'blueprint':
      return `/blueprints/${resourceId}`
    case 'affix':
      return `/affixes/${resourceId}`
    case 'global_meta_attribute':
      return `/global-meta-attributes/${resourceId}`
    default:
      return null
  }
}

export function ActivityPanel() {
  const { panelDocked, closePanel, selectedClientId } = useUi()
  const panelRef = useRef<HTMLDivElement>(null)
  const navigate = useNavigate()
  const location = useLocation()
  const queryClient = useQueryClient()

  const { data } = useAuditLog({
    client_id: selectedClientId ?? undefined,
    limit: 50,
  })

  useEffect(() => {
    if (panelDocked) return
    function handleClickOutside(event: MouseEvent) {
      if (panelRef.current && !panelRef.current.contains(event.target as Node)) {
        closePanel()
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [closePanel, panelDocked])

  useEffect(() => {
    queryClient.invalidateQueries({ queryKey: ['auditLog'] })
  }, [location.pathname, queryClient])

  const entries = data?.data ?? []

  const handleEntryClick = (entry: AuditLogEntry) => {
    const path = resourceDetailPath(entry.resource_type, entry.resource_id)
    if (path) navigate(path)
  }

  return (
    <div
      ref={panelRef}
      className={cn(
        panelDocked
          ? 'w-80 border-l bg-background shrink-0'
          : 'fixed top-0 right-0 bottom-0 z-30 w-80 border-l bg-background shadow-lg animate-in slide-in-from-right',
        'flex flex-col',
      )}
    >
      <div className="flex items-center justify-between px-4 h-12 border-b shrink-0">
        <div className="flex items-center gap-2 text-sm font-medium">
          <History className="h-4 w-4" />
          Activity
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={closePanel}
            className="inline-flex items-center justify-center h-8 w-8 rounded-md hover:bg-accent hover:text-accent-foreground"
            aria-label="Close activity panel"
          >
            <X className="h-4 w-4" />
          </button>
        </div>
      </div>
      <div className="flex-1 overflow-y-auto">
        {entries.length === 0 ? (
          <div className="p-4 text-sm text-muted-foreground">No recent activity.</div>
        ) : (
          <ul className="divide-y">
            {entries.map((entry) => (
              <li key={entry.id}>
                <button
                  onClick={() => handleEntryClick(entry)}
                  className="w-full text-left px-4 py-3 hover:bg-accent transition-colors cursor-pointer"
                >
                  <div className="flex items-center justify-between gap-2">
                    <span className="text-xs text-muted-foreground">
                      {relativeTime(entry.timestamp)}
                    </span>
                    <span
                      className={cn(
                        'inline-flex items-center rounded px-1.5 py-0.5 text-xs font-medium',
                        actionBadgeVariant(entry.action),
                      )}
                    >
                      {entry.action}
                    </span>
                  </div>
                  <div className="mt-1 text-sm text-muted-foreground">
                    {entry.actor_key_name}
                  </div>
                  <div className="mt-0.5 text-sm">
                    <span className="font-medium">{entry.resource_type}</span>{' '}
                    <span>{entry.resource_id}</span>
                  </div>
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>
      <div className="border-t px-4 py-2 shrink-0">
        <button
          onClick={() => navigate('/audit-log')}
          className="w-full text-left text-sm text-muted-foreground hover:text-foreground transition-colors"
        >
          View all
        </button>
      </div>
    </div>
  )
}
