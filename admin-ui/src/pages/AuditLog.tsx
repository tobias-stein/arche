import { useState, useCallback } from 'react'
import {
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  ChevronUp,
  RotateCcw,
  Search,
} from 'lucide-react'
import ReactDiffViewer, { DiffMethod } from 'react-diff-viewer-continued'

import { useAuditLog } from '@/api/generated'
import type { AuditLogEntry, AuditLogListQuery } from '@/api/generated'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Skeleton } from '@/components/ui/skeleton'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'

const RESOURCE_TYPES = [
  { value: '', label: 'All' },
  { value: 'blueprint', label: 'Blueprint' },
  { value: 'affix', label: 'Affix' },
  { value: 'global_meta_attribute', label: 'Global Meta Attribute' },
  { value: 'client', label: 'Client' },
  { value: 'api_key', label: 'API Key' },
]

const ACTIONS = [
  { value: '', label: 'All' },
  { value: 'created', label: 'Created' },
  { value: 'updated', label: 'Updated' },
  { value: 'deleted', label: 'Deleted' },
  { value: 'force_deleted', label: 'Force Deleted' },
  { value: 'adjusted', label: 'Adjusted' },
]

const PER_PAGE_OPTIONS = [10, 25, 50]

type FilterState = {
  clientId: string
  resourceType: string
  action: string
  from: string
  to: string
}

const EMPTY_FILTERS: FilterState = {
  clientId: '',
  resourceType: '',
  action: '',
  from: '',
  to: '',
}

function actionBadgeVariant(action: string): 'default' | 'secondary' | 'destructive' | 'outline' {
  switch (action) {
    case 'created':
      return 'default'
    case 'updated':
      return 'secondary'
    case 'deleted':
      return 'destructive'
    case 'forceDeleted':
      return 'destructive'
    case 'adjusted':
      return 'outline'
    default:
      return 'secondary'
  }
}

function formatAction(action: string): string {
  switch (action) {
    case 'forceDeleted':
      return 'Force Deleted'
    default:
      return action.charAt(0).toUpperCase() + action.slice(1)
  }
}

function formatUtcTooltip(iso: string): string {
  const d = new Date(iso)
  return `UTC: ${d.toISOString().replace('T', ' ').replace(/\.\d+Z$/, '')}`
}

function formatLocalTime(iso: string): string {
  return new Date(iso).toLocaleString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}

function JsonDiffView({
  before,
  after,
  onClose,
}: {
  before: Record<string, unknown> | null
  after: Record<string, unknown> | null
  onClose: () => void
}) {
  const oldValue = before ? JSON.stringify(before, null, 2) : ''
  const newValue = after ? JSON.stringify(after, null, 2) : ''

  if (!oldValue && !newValue) {
    return (
      <div className="p-4 text-sm text-muted-foreground">
        No data available
        <button onClick={onClose} className="ml-4 text-primary hover:underline">Close</button>
      </div>
    )
  }

  return (
    <div className="border-t bg-muted/20">
      <div className="flex items-center justify-between px-4 py-2 border-b bg-muted/40">
        <span className="text-xs font-medium text-muted-foreground">JSON Diff</span>
        <Button variant="ghost" size="sm" onClick={onClose}>
          <ChevronUp className="h-4 w-4" />
        </Button>
      </div>
      <div className="p-2 overflow-auto max-h-96">
        <div className="hidden sm:block">
          <ReactDiffViewer
            oldValue={oldValue}
            newValue={newValue}
            splitView
            compareMethod={DiffMethod.WORDS}
            hideLineNumbers={false}
            leftTitle="Before"
            rightTitle="After"
            styles={{
              diffContainer: { fontSize: '12px' },
              line: { wordBreak: 'break-all' },
            }}
          />
        </div>
        <div className="sm:hidden">
          <ReactDiffViewer
            oldValue={oldValue}
            newValue={newValue}
            splitView={false}
            compareMethod={DiffMethod.WORDS}
            hideLineNumbers={false}
            leftTitle="Before"
            rightTitle="After"
            styles={{
              diffContainer: { fontSize: '11px' },
              line: { wordBreak: 'break-all' },
            }}
          />
        </div>
      </div>
    </div>
  )
}

function TableSkeleton() {
  return (
    <div className="space-y-2">
      {Array.from({ length: 8 }).map((_, i) => (
        <div key={i} className="flex items-center gap-4 p-3 border-b">
          <Skeleton className="h-4 w-36" />
          <Skeleton className="h-4 w-24" />
          <Skeleton className="h-4 w-20" />
          <Skeleton className="h-4 w-24" />
          <Skeleton className="h-5 w-20 rounded-md" />
          <Skeleton className="h-4 w-32" />
        </div>
      ))}
    </div>
  )
}

function CardSkeleton() {
  return (
    <div className="space-y-3">
      {Array.from({ length: 5 }).map((_, i) => (
        <Card key={i}>
          <CardContent className="p-4 space-y-2">
            <Skeleton className="h-4 w-3/4" />
            <Skeleton className="h-4 w-1/2" />
            <Skeleton className="h-5 w-16 rounded-md" />
          </CardContent>
        </Card>
      ))}
    </div>
  )
}

export default function AuditLog() {
  const [filters, setFilters] = useState<FilterState>(EMPTY_FILTERS)
  const [appliedFilters, setAppliedFilters] = useState<FilterState>(EMPTY_FILTERS)
  const [perPage, setPerPage] = useState(25)
  const [cursorStack, setCursorStack] = useState<string[]>([])
  const [currentCursor, setCurrentCursor] = useState<string | undefined>(undefined)
  const [expandedRowId, setExpandedRowId] = useState<string | null>(null)

  function buildQuery(): AuditLogListQuery {
    const q: AuditLogListQuery = {
      cursor: currentCursor,
      limit: perPage,
    }
    if (appliedFilters.clientId) q.clientId = appliedFilters.clientId
    if (appliedFilters.resourceType) q.resourceType = appliedFilters.resourceType
    if (appliedFilters.action) q.action = appliedFilters.action
    if (appliedFilters.from) q.from = new Date(appliedFilters.from).toISOString()
    if (appliedFilters.to) {
      const to = new Date(appliedFilters.to)
      to.setUTCHours(23, 59, 59, 999)
      q.to = to.toISOString()
    }
    return q
  }

  const { data, isLoading, isError, error } = useAuditLog(buildQuery())

  const entries: AuditLogEntry[] = (data?.data ?? []) as AuditLogEntry[]
  const nextCursor = data?.nextCursor
  const hasPrev = cursorStack.length > 0
  const hasNext = !!nextCursor

  const filterDirty =
    JSON.stringify(filters) !== JSON.stringify(appliedFilters)

  const hasAppliedFilters =
    appliedFilters.resourceType !== '' ||
    appliedFilters.action !== '' ||
    appliedFilters.from !== '' ||
    appliedFilters.to !== ''

  function resetNavigation() {
    setCursorStack([])
    setCurrentCursor(undefined)
    setExpandedRowId(null)
  }

  function applyFilters() {
    setAppliedFilters({ ...filters })
    resetNavigation()
  }

  function resetFilters() {
    setFilters(EMPTY_FILTERS)
    setAppliedFilters(EMPTY_FILTERS)
    resetNavigation()
  }

  function goNext() {
    if (nextCursor) {
      setCursorStack((s) => [...s, currentCursor ?? ''])
      setCurrentCursor(nextCursor)
      setExpandedRowId(null)
    }
  }

  function goPrev() {
    if (cursorStack.length > 0) {
      const prev = cursorStack[cursorStack.length - 1]
      setCursorStack((s) => s.slice(0, -1))
      setCurrentCursor(prev || undefined)
      setExpandedRowId(null)
    }
  }

  const handleRowClick = useCallback((id: string) => {
    setExpandedRowId((prev) => (prev === id ? null : id))
  }, [])

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold">Audit Log</h2>
      </div>

      {/* Filter bar */}
      <Card>
        <CardContent className="p-4">
          <div className="flex flex-col gap-3">
            <div className="flex flex-wrap gap-3">
              <div className="flex flex-col gap-1.5 min-w-0" style={{ width: 180 }}>
                <label htmlFor="audit-resource-type" className="text-xs font-medium text-muted-foreground">
                  Resource Type
                </label>
                <select
                  id="audit-resource-type"
                  value={filters.resourceType}
                  onChange={(e) => setFilters((f) => ({ ...f, resourceType: e.target.value }))}
                  className="h-9 rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                >
                  {RESOURCE_TYPES.map((rt) => (
                    <option key={rt.value} value={rt.value}>
                      {rt.label}
                    </option>
                  ))}
                </select>
              </div>

              <div className="flex flex-col gap-1.5 min-w-0" style={{ width: 140 }}>
                <label htmlFor="audit-action" className="text-xs font-medium text-muted-foreground">
                  Action
                </label>
                <select
                  id="audit-action"
                  value={filters.action}
                  onChange={(e) => setFilters((f) => ({ ...f, action: e.target.value }))}
                  className="h-9 rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                >
                  {ACTIONS.map((a) => (
                    <option key={a.value} value={a.value}>
                      {a.label}
                    </option>
                  ))}
                </select>
              </div>

              <div className="flex flex-col gap-1.5 min-w-0" style={{ width: 160 }}>
                <label htmlFor="audit-from" className="text-xs font-medium text-muted-foreground">
                  From
                </label>
                <Input
                  id="audit-from"
                  type="date"
                  value={filters.from}
                  onChange={(e) => setFilters((f) => ({ ...f, from: e.target.value }))}
                />
              </div>

              <div className="flex flex-col gap-1.5 min-w-0" style={{ width: 160 }}>
                <label htmlFor="audit-to" className="text-xs font-medium text-muted-foreground">
                  To
                </label>
                <Input
                  id="audit-to"
                  type="date"
                  value={filters.to}
                  onChange={(e) => setFilters((f) => ({ ...f, to: e.target.value }))}
                />
              </div>
            </div>

            <div className="flex items-center gap-2">
              <Button
                size="sm"
                onClick={applyFilters}
                disabled={!filterDirty}
              >
                <Search className="h-3.5 w-3.5" />
                Apply
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={resetFilters}
                disabled={
                  JSON.stringify(filters) === JSON.stringify(EMPTY_FILTERS) &&
                  JSON.stringify(appliedFilters) === JSON.stringify(EMPTY_FILTERS)
                }
              >
                <RotateCcw className="h-3.5 w-3.5" />
                Reset
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Error state */}
      {isError && (
        <Card>
          <CardContent className="p-8 text-center">
            <p className="text-destructive font-medium">
              Failed to load audit log
            </p>
            <p className="text-sm text-muted-foreground mt-1">
              {error instanceof Error ? error.message : 'Unknown error'}
            </p>
          </CardContent>
        </Card>
      )}

      {/* Loading state */}
      {isLoading && (
        <Card>
          <CardContent className="p-0">
            <div className="hidden md:block">
              <TableSkeleton />
            </div>
            <div className="md:hidden p-4">
              <CardSkeleton />
            </div>
          </CardContent>
        </Card>
      )}

      {/* Empty state */}
      {!isLoading && !isError && entries.length === 0 && (
        <Card>
          <CardContent className="p-8 text-center">
            <p className="text-muted-foreground">
              No audit log entries found
            </p>
            <p className="text-sm text-muted-foreground mt-1">
              {hasAppliedFilters
                ? 'Try adjusting your filters'
                : 'Audit log entries will appear here as changes are made'}
            </p>
          </CardContent>
        </Card>
      )}

      {/* Data table */}
      {!isLoading && !isError && entries.length > 0 && (
        <Card>
          <CardContent className="p-0">
            {/* Desktop table */}
            <div className="hidden md:block">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="w-44">Timestamp</TableHead>
                    <TableHead className="w-32">Actor</TableHead>
                    <TableHead className="w-24">Client</TableHead>
                    <TableHead className="w-32">Resource Type</TableHead>
                    <TableHead className="w-28">Action</TableHead>
                    <TableHead>Resource</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {entries.map((entry) => (
                    <TableRow
                      key={entry.id}
                      className="cursor-pointer"
                      onClick={() => handleRowClick(entry.id)}
                    >
                      <TableCell className="text-sm" title={formatUtcTooltip(entry.timestamp)}>
                        {formatLocalTime(entry.timestamp)}
                      </TableCell>
                      <TableCell className="text-sm">{entry.actorKeyName}</TableCell>
                      <TableCell className="text-sm text-muted-foreground">
                        {entry.clientId ? entry.clientId.slice(0, 8) : '-'}
                      </TableCell>
                      <TableCell className="text-sm capitalize">
                        {entry.resourceType.replace(/_/g, ' ')}
                      </TableCell>
                      <TableCell>
                        <Badge variant={actionBadgeVariant(entry.action)}>
                          {formatAction(entry.action)}
                        </Badge>
                      </TableCell>
                      <TableCell className="text-sm font-mono">
                        {entry.resourceId.slice(0, 12)}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>

              {/* Expanded row detail */}
              {expandedRowId && (() => {
                const entry = entries.find((e) => e.id === expandedRowId)
                if (!entry) return null
                return (
                  <JsonDiffView
                    before={entry.before}
                    after={entry.after}
                    onClose={() => setExpandedRowId(null)}
                  />
                )
              })()}
            </div>

            {/* Mobile cards */}
            <div className="md:hidden divide-y">
              {entries.map((entry) => (
                <div key={entry.id}>
                  <div
                    className="px-4 py-3 space-y-2 cursor-pointer"
                    onClick={() => handleRowClick(entry.id)}
                  >
                    <div className="flex items-center justify-between">
                      <Badge variant={actionBadgeVariant(entry.action)}>
                        {formatAction(entry.action)}
                      </Badge>
                      <span className="text-xs text-muted-foreground" title={formatUtcTooltip(entry.timestamp)}>
                        {formatLocalTime(entry.timestamp)}
                      </span>
                    </div>
                    <div className="flex items-center gap-2 text-sm">
                      <span className="font-medium">{entry.actorKeyName}</span>
                    </div>
                    <div className="flex items-center gap-2 text-xs text-muted-foreground">
                      <span className="capitalize">{entry.resourceType.replace(/_/g, ' ')}</span>
                      <span className="font-mono">{entry.resourceId.slice(0, 12)}</span>
                      {entry.clientId && <span>Client: {entry.clientId.slice(0, 8)}</span>}
                    </div>
                    {expandedRowId === entry.id ? (
                      <ChevronUp className="h-4 w-4 ml-auto text-muted-foreground" />
                    ) : (
                      <ChevronDown className="h-4 w-4 ml-auto text-muted-foreground" />
                    )}
                  </div>
                  {expandedRowId === entry.id && (
                    <JsonDiffView
                      before={entry.before}
                      after={entry.after}
                      onClose={() => setExpandedRowId(null)}
                    />
                  )}
                </div>
              ))}
            </div>

            {/* Pagination */}
            <div className="flex flex-col sm:flex-row items-center justify-between gap-3 px-4 py-3 border-t">
              <div className="flex items-center gap-3">
                <span className="text-sm text-muted-foreground">
                  {cursorStack.length > 0 ? `Page ${cursorStack.length + 1}` : 'Page 1'}
                  {` (${entries.length} shown)`}
                </span>
                <div className="flex items-center gap-1.5">
                  <label htmlFor="per-page" className="text-xs text-muted-foreground">
                    Per page
                  </label>
                  <select
                    id="per-page"
                    value={perPage}
                    onChange={(e) => {
                      setPerPage(Number(e.target.value))
                      resetNavigation()
                    }}
                    className="h-7 rounded border border-input bg-transparent px-1.5 text-xs shadow-sm"
                  >
                    {PER_PAGE_OPTIONS.map((n) => (
                      <option key={n} value={n}>
                        {n}
                      </option>
                    ))}
                  </select>
                </div>
              </div>
              <div className="flex items-center gap-1">
                <Button
                  size="sm"
                  variant="outline"
                  onClick={goPrev}
                  disabled={!hasPrev}
                >
                  <ChevronLeft className="h-4 w-4" />
                  Prev
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={goNext}
                  disabled={!hasNext}
                >
                  Next
                  <ChevronRight className="h-4 w-4" />
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  )
}
