import { useEffect, useMemo, useState } from 'react'
import type { MouseEvent as ReactMouseEvent } from 'react'
import { Link } from 'react-router-dom'
import {
  ArrowUpDown,
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  ChevronUp,
  Copy,
  Pencil,
  Plus,
  Trash2,
} from 'lucide-react'

import {
  useAffixesList,
  useBatchAssignBlueprints,
  useBatchDeleteBlueprints,
  useBatchEditBlueprints,
  useBlueprintsList,
  useClientsList,
  useDeleteBlueprint,
} from '@/api/generated'
import type {
  Affix,
  Blueprint,
  ClientResponse,
} from '@/api/generated'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
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
import { BatchEditDialog } from '@/components/BatchEditDialog'
import { BlueprintFormModal } from '@/components/BlueprintFormModal'
import { useUi } from '@/stores/ui'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { useToast } from '@/hooks/use-toast'

const PER_PAGE_OPTIONS = [10, 25, 50, 100]
const DEFAULT_PER_PAGE = 25

type SortField = 'name' | 'weight' | 'updated'
type SortDir = 'asc' | 'desc'

function pluralize(count: number, singular: string, plural?: string): string {
  if (count === 1) return singular
  return plural ?? `${singular}s`
}

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  })
}

function formatTimestamp(dateStr: string): string {
  return new Date(dateStr).toLocaleString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

function TableSkeleton() {
  return (
    <div className="space-y-2">
      {Array.from({ length: 8 }).map((_, i) => (
        <div key={i} className="flex items-center gap-4 p-3 border-b">
          <Skeleton className="h-4 w-4 rounded-sm shrink-0" />
          <Skeleton className="h-4 w-40" />
          <Skeleton className="h-5 w-20 rounded-md" />
          <Skeleton className="h-4 w-12" />
          <Skeleton className="h-4 w-32" />
          <div className="flex gap-2 ml-auto">
            <Skeleton className="h-8 w-8 rounded-md" />
            <Skeleton className="h-8 w-8 rounded-md" />
            <Skeleton className="h-8 w-8 rounded-md" />
          </div>
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

function SortIcon({ field, currentField, dir }: { field: SortField; currentField: SortField; dir: SortDir }) {
  if (field !== currentField) {
    return <ArrowUpDown className="h-3 w-3 ml-1 opacity-40" />
  }
  return dir === 'asc' ? (
    <ChevronUp className="h-3 w-3 ml-1" />
  ) : (
    <ChevronDown className="h-3 w-3 ml-1" />
  )
}

function AffixPickerDialog({
  open,
  onOpenChange,
  selectedBlueprintIds,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  selectedBlueprintIds: string[]
}) {
  const { toast } = useToast()
  const { data: affixData, isLoading } = useAffixesList({ per_page: 200 })
  const batchAssignMutation = useBatchAssignBlueprints()

  const [selectedAffixIds, setSelectedAffixIds] = useState<Set<string>>(new Set())

  const affixes: Affix[] = (affixData?.data ?? []) as Affix[]

  const allSelected =
    affixes.length > 0 && affixes.every((a) => selectedAffixIds.has(a.id))

  function toggleAffix(id: string) {
    setSelectedAffixIds((prev) => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  }

  function toggleAll() {
    if (allSelected) {
      setSelectedAffixIds(new Set())
    } else {
      setSelectedAffixIds(new Set(affixes.map((a) => a.id)))
    }
  }

  async function handleAssign() {
    if (selectedAffixIds.size === 0) return
    try {
      await batchAssignMutation.mutateAsync({
        blueprint_ids: selectedBlueprintIds,
        affix_ids: Array.from(selectedAffixIds),
        weight: 1,
      })
      toast({
        title: `Assigned ${pluralize(selectedAffixIds.size, 'affix', 'affixes')} to ${pluralize(selectedBlueprintIds.length, 'blueprint')}`,
      })
      onOpenChange(false)
    } catch {
      toast({ title: 'Failed to assign affixes', variant: 'destructive' })
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>Assign Affixes to Blueprints</DialogTitle>
        </DialogHeader>
        <div className="max-h-64 overflow-y-auto border rounded-md">
          {isLoading ? (
            <div className="p-4 space-y-2">
              {Array.from({ length: 5 }).map((_, i) => (
                <Skeleton key={i} className="h-6 w-full" />
              ))}
            </div>
          ) : affixes.length === 0 ? (
            <p className="p-4 text-sm text-muted-foreground">
              No affixes found
            </p>
          ) : (
            <>
              <label className="flex items-center gap-2 px-3 py-2 border-b text-sm font-medium hover:bg-muted/50 cursor-pointer">
                <input
                  type="checkbox"
                  checked={allSelected}
                  onChange={toggleAll}
                  className="rounded"
                />
                Select all
              </label>
              {affixes.map((a) => (
                <label
                  key={a.id}
                  className="flex items-center gap-2 px-3 py-2 text-sm hover:bg-muted/50 cursor-pointer"
                >
                  <input
                    type="checkbox"
                    checked={selectedAffixIds.has(a.id)}
                    onChange={() => toggleAffix(a.id)}
                    className="rounded"
                  />
                  {a.name}
                </label>
              ))}
            </>
          )}
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button
            onClick={handleAssign}
            disabled={selectedAffixIds.size === 0 || batchAssignMutation.isPending}
          >
            {batchAssignMutation.isPending
              ? 'Assigning...'
              : `Assign (${pluralize(selectedAffixIds.size, 'affix', 'affixes')})`}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

export default function Blueprints() {
  const { toast } = useToast()

  const [page, setPage] = useState(1)
  const [perPage, setPerPage] = useState(DEFAULT_PER_PAGE)
  const [search, setSearch] = useState('')
  const [debouncedSearch, setDebouncedSearch] = useState('')
  const [archetypeFilter, setArchetypeFilter] = useState('all')
  const [sortField, setSortField] = useState<SortField>('name')
  const [sortDir, setSortDir] = useState<SortDir>('asc')
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set())
  const [lastCheckedIdx, setLastCheckedIdx] = useState<number | null>(null)

  const [editingBlueprint, setEditingBlueprint] = useState<Blueprint | null>(null)
  const [showEditModal, setShowEditModal] = useState(false)
  const [showCreateModal, setShowCreateModal] = useState(false)

  const [duplicatingBlueprint, setDuplicatingBlueprint] = useState<Blueprint | null>(null)
  const [showDuplicateModal, setShowDuplicateModal] = useState(false)

  const [deletingBlueprint, setDeletingBlueprint] = useState<Blueprint | null>(null)
  const deleteMutation = useDeleteBlueprint()

  const [showBatchDelete, setShowBatchDelete] = useState(false)
  const batchDeleteMutation = useBatchDeleteBlueprints()

  const [showBatchAssign, setShowBatchAssign] = useState(false)
  const [batchAssignKey, setBatchAssignKey] = useState(0)

  const [showBatchEdit, setShowBatchEdit] = useState(false)
  const batchEditMutation = useBatchEditBlueprints()

  useEffect(() => {
    const timeout = setTimeout(() => {
      setDebouncedSearch(search)
      setPage(1)
    }, 300)
    return () => clearTimeout(timeout)
  }, [search])

  const listQuery = useMemo(
    () => ({
      page,
      per_page: perPage,
      search: debouncedSearch || undefined,
      archetype: archetypeFilter !== 'all' ? archetypeFilter : undefined,
    }),
    [page, perPage, debouncedSearch, archetypeFilter],
  )

  const {
    data,
    isLoading,
    isError,
    error,
  } = useBlueprintsList(listQuery)

  const { data: allData } = useBlueprintsList({ per_page: 200 })

  const { selectedClientId } = useUi()
  const { data: clientsData } = useClientsList({ per_page: 200 })
  const clientNameMap = useMemo(() => {
    const map = new Map<string, string>()
    for (const c of (clientsData?.data ?? []) as ClientResponse[]) {
      map.set(c.id, c.name)
    }
    return map
  }, [clientsData])

  const blueprints = useMemo<Blueprint[]>(
    () => (data?.data ?? []) as Blueprint[],
    [data?.data],
  )

  const allBlueprints = useMemo<Blueprint[]>(
    () => (allData?.data ?? []) as Blueprint[],
    [allData?.data],
  )

  const archetypes = useMemo(
    () => {
      const set = new Set<string>()
      allBlueprints.forEach((b) => set.add(b.archetype))
      return Array.from(set).sort((a, b) => a.localeCompare(b))
    },
    [allBlueprints],
  )

  const sortedBlueprints = useMemo(() => {
    const items = [...blueprints]
    items.sort((a, b) => {
      let cmp = 0
      switch (sortField) {
        case 'name':
          cmp = a.name.localeCompare(b.name)
          break
        case 'weight':
          cmp = a.weight - b.weight
          break
        case 'updated':
          cmp = a.updated_at.localeCompare(b.updated_at)
          break
      }
      return sortDir === 'asc' ? cmp : -cmp
    })
    return items
  }, [blueprints, sortField, sortDir])

  const totalCount = data?.total ?? 0
  const totalPages = Math.max(1, Math.ceil(totalCount / perPage))

  const selectedCount = selectedIds.size

  const paginationText =
    totalCount > 0
      ? `Page ${page} of ${totalPages} (${totalCount} total)`
      : blueprints.length > 0
        ? `${blueprints.length} result${blueprints.length === 1 ? '' : 's'}`
        : 'No results'

  function clearSelection() {
    setSelectedIds(new Set())
    setLastCheckedIdx(null)
  }

  function handleSort(field: SortField) {
    if (sortField === field) {
      setSortDir((d) => (d === 'asc' ? 'desc' : 'asc'))
    } else {
      setSortField(field)
      setSortDir('asc')
    }
  }

  function handleCheckClick(bp: Blueprint, idx: number, e: ReactMouseEvent) {
    e.stopPropagation()
    setSelectedIds((prev) => {
      const next = new Set(prev)

      if (e.shiftKey && lastCheckedIdx !== null) {
        const start = Math.min(lastCheckedIdx, idx)
        const end = Math.max(lastCheckedIdx, idx)
        for (let i = start; i <= end; i++) {
          if (i < sortedBlueprints.length) {
            next.add(sortedBlueprints[i].id)
          }
        }
      } else {
        if (next.has(bp.id)) next.delete(bp.id)
        else next.add(bp.id)
      }

      return next
    })
    setLastCheckedIdx(idx)
  }

  function handleSelectAll() {
    if (sortedBlueprints.length === 0) return
    const allSelected = sortedBlueprints.every((b) => selectedIds.has(b.id))
    setSelectedIds((prev) => {
      const next = new Set(prev)
      sortedBlueprints.forEach((b) => {
        if (allSelected) next.delete(b.id)
        else next.add(b.id)
      })
      return next
    })
  }

  function handleEdit(bp: Blueprint) {
    setEditingBlueprint(bp)
    setShowEditModal(true)
  }

  function getCopyName(name: string): string {
    const base = `${name} (Copy)`
    const existingNames = new Set(allBlueprints.map((b) => b.name))
    if (!existingNames.has(base)) return base
    let n = 2
    while (existingNames.has(`${name} (Copy ${n})`)) n++
    return `${name} (Copy ${n})`
  }

  function handleDuplicate(bp: Blueprint) {
    const copy: Blueprint = { ...bp, name: getCopyName(bp.name) }
    setDuplicatingBlueprint(copy)
    setShowDuplicateModal(true)
  }

  async function handleDelete() {
    if (!deletingBlueprint) return
    try {
      await deleteMutation.mutateAsync(deletingBlueprint.id)
      toast({ title: `Deleted "${deletingBlueprint.name}"` })
      setDeletingBlueprint(null)
      setSelectedIds((prev) => {
        const next = new Set(prev)
        next.delete(deletingBlueprint.id)
        return next
      })
    } catch {
      toast({ title: 'Failed to delete blueprint', variant: 'destructive' })
    }
  }

  async function handleBatchDelete() {
    if (selectedCount === 0) return
    try {
      await batchDeleteMutation.mutateAsync({
        ids: Array.from(selectedIds),
      })
      toast({ title: `Deleted ${selectedCount} ${pluralize(selectedCount, 'blueprint')}` })
      setShowBatchDelete(false)
      clearSelection()
    } catch {
      toast({ title: 'Failed to batch delete', variant: 'destructive' })
    }
  }

  function handlePageSizeChange(newSize: number) {
    setPerPage(newSize)
    setPage(1)
    clearSelection()
  }

  const allPageSelected =
    sortedBlueprints.length > 0 &&
    sortedBlueprints.every((b) => selectedIds.has(b.id))

  const selectAllLabel = allPageSelected ? 'Deselect all' : 'Select all'

  const selectedBlueprints = useMemo(
    () => allBlueprints.filter((b) => selectedIds.has(b.id)),
    [allBlueprints, selectedIds],
  )

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold">Blueprints</h2>
        <Button onClick={() => setShowCreateModal(true)}>
          <Plus className="h-4 w-4" />
          Create Blueprint
        </Button>
      </div>

      {/* Filter bar */}
      <div className="flex flex-col sm:flex-row gap-3">
        <div className="flex items-center gap-2">
          <label htmlFor="archetype-filter" className="text-sm font-medium">
            Archetype
          </label>
          <select
            id="archetype-filter"
            value={archetypeFilter}
            onChange={(e) => {
              setArchetypeFilter(e.target.value)
              setPage(1)
              setSelectedIds(new Set())
              setLastCheckedIdx(null)
            }}
            className="h-9 rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
          >
            <option value="all">All</option>
            {archetypes.map((a) => (
              <option key={a} value={a}>
                {a}
              </option>
            ))}
          </select>
        </div>
        <div className="flex-1">
          <Input
            placeholder="Search by name..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>
      </div>

      {/* Batch toolbar */}
      {selectedCount > 0 && (
        <div className="flex items-center justify-between rounded-lg border bg-muted/30 px-4 py-3">
          <div className="flex items-center gap-4">
            <span className="text-sm font-medium">
              {selectedCount} selected
            </span>
            <Button
              size="sm"
              variant="ghost"
              onClick={clearSelection}
              className="h-auto p-0 text-sm underline"
            >
              Clear
            </Button>
          </div>
          <div className="flex items-center gap-2">
            <Button
              size="sm"
              variant="destructive"
              onClick={() => setShowBatchDelete(true)}
              disabled={batchDeleteMutation.isPending}
            >
              <Trash2 className="h-4 w-4" />
              Batch Delete
            </Button>
            <Button
              size="sm"
              variant="secondary"
              onClick={() => {
                setBatchAssignKey((k) => k + 1)
                setShowBatchAssign(true)
              }}
            >
              Batch Assign
            </Button>
            <Button
              size="sm"
              variant="secondary"
              onClick={() => setShowBatchEdit(true)}
              disabled={selectedCount === 0 || batchEditMutation.isPending}
            >
              Batch Edit
            </Button>
          </div>
        </div>
      )}

      {/* Error state */}
      {isError && (
        <Card>
          <CardContent className="p-8 text-center">
            <p className="text-destructive font-medium">
              Failed to load blueprints
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
      {!isLoading && !isError && allBlueprints.length === 0 && (
        <Card>
          <CardContent className="p-8 text-center">
            <p className="text-muted-foreground">
              No blueprints found
            </p>
            <p className="text-sm text-muted-foreground mt-1">
              Create your first blueprint to get started
            </p>
          </CardContent>
        </Card>
      )}

      {/* Empty filtered state */}
      {!isLoading && !isError && allBlueprints.length > 0 && blueprints.length === 0 && (
        <Card>
          <CardContent className="p-8 text-center">
            <p className="text-muted-foreground">
              {debouncedSearch || archetypeFilter !== 'all'
                ? 'No blueprints match the current filters'
                : 'No blueprints on this page'}
            </p>
            <p className="text-sm text-muted-foreground mt-1">
              Try adjusting your search or filters
            </p>
          </CardContent>
        </Card>
      )}

      {/* Data table */}
      {!isLoading && !isError && blueprints.length > 0 && (
        <Card>
          <CardContent className="p-0">
            {/* Desktop table */}
            <div className="hidden md:block">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="w-10">
                      <input
                        type="checkbox"
                        checked={allPageSelected}
                        onChange={handleSelectAll}
                        aria-label={selectAllLabel}
                        className="rounded"
                      />
                    </TableHead>
                    <TableHead>
                      <button
                        type="button"
                        className="inline-flex items-center font-medium text-xs text-muted-foreground hover:text-foreground transition-colors"
                        onClick={() => handleSort('name')}
                      >
                        Name
                        <SortIcon field={sortField} currentField="name" dir={sortDir} />
                      </button>
                    </TableHead>
                    {!selectedClientId && (
                      <TableHead>
                        <span className="text-xs text-muted-foreground font-medium">
                          Client
                        </span>
                      </TableHead>
                    )}
                    <TableHead>
                      <span className="text-xs text-muted-foreground font-medium">
                        Archetype
                      </span>
                    </TableHead>
                    <TableHead>
                      <button
                        type="button"
                        className="inline-flex items-center font-medium text-xs text-muted-foreground hover:text-foreground transition-colors"
                        onClick={() => handleSort('weight')}
                      >
                        Weight
                        <SortIcon field={sortField} currentField="weight" dir={sortDir} />
                      </button>
                    </TableHead>
                    <TableHead>
                      <button
                        type="button"
                        className="inline-flex items-center font-medium text-xs text-muted-foreground hover:text-foreground transition-colors"
                        onClick={() => handleSort('updated')}
                      >
                        Updated
                        <SortIcon field={sortField} currentField="updated" dir={sortDir} />
                      </button>
                    </TableHead>
                    <TableHead className="w-28 text-right">Actions</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {sortedBlueprints.map((bp, idx) => (
                    <TableRow
                      key={bp.id}
                      data-state={
                        selectedIds.has(bp.id) ? 'selected' : undefined
                      }
                    >
                      <TableCell>
                        <input
                          type="checkbox"
                          checked={selectedIds.has(bp.id)}
                          onClick={(e) => handleCheckClick(bp, idx, e)}
                          onChange={() => {}}
                          aria-label={`Select ${bp.name}`}
                          className="rounded"
                        />
                      </TableCell>
                      <TableCell className="font-medium">
                        <Link
                          to={`/blueprints/${bp.id}`}
                          className="text-primary hover:underline"
                        >
                          {bp.name}
                        </Link>
                      </TableCell>
                      {!selectedClientId && (
                        <TableCell className="text-muted-foreground text-sm">
                          {clientNameMap.get(bp.client_id) ?? bp.client_id.slice(0, 8)}
                        </TableCell>
                      )}
                      <TableCell>
                        <Badge variant="secondary">{bp.archetype}</Badge>
                      </TableCell>
                      <TableCell className="tabular-nums">
                        {bp.weight}
                      </TableCell>
                      <TableCell className="text-muted-foreground text-sm">
                        {formatTimestamp(bp.updated_at)}
                      </TableCell>
                      <TableCell className="text-right">
                        <div className="flex justify-end gap-1">
                          <Button
                            size="icon"
                            variant="ghost"
                            onClick={() => handleEdit(bp)}
                            aria-label={`Edit ${bp.name}`}
                          >
                            <Pencil className="h-4 w-4" />
                          </Button>
                          <Button
                            size="icon"
                            variant="ghost"
                            onClick={() => handleDuplicate(bp)}
                            aria-label={`Duplicate ${bp.name}`}
                          >
                            <Copy className="h-4 w-4" />
                          </Button>
                          <Button
                            size="icon"
                            variant="ghost"
                            onClick={() => setDeletingBlueprint(bp)}
                            aria-label={`Delete ${bp.name}`}
                          >
                            <Trash2 className="h-4 w-4" />
                          </Button>
                        </div>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>

            {/* Mobile cards */}
            <div className="md:hidden divide-y">
              {sortedBlueprints.map((bp, idx) => (
                <div
                  key={bp.id}
                  className={`px-4 py-3 space-y-2 ${
                    selectedIds.has(bp.id) ? 'bg-muted/50' : ''
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <input
                        type="checkbox"
                        checked={selectedIds.has(bp.id)}
                        onClick={(e) => handleCheckClick(bp, idx, e)}
                        onChange={() => {}}
                        aria-label={`Select ${bp.name}`}
                        className="rounded"
                      />
                      <Link
                        to={`/blueprints/${bp.id}`}
                        className="font-medium text-primary hover:underline"
                      >
                        {bp.name}
                      </Link>
                    </div>
                    <div className="flex gap-1">
                      <Button
                        size="icon"
                        variant="ghost"
                        className="h-8 w-8"
                        onClick={() => handleEdit(bp)}
                        aria-label={`Edit ${bp.name}`}
                      >
                        <Pencil className="h-3.5 w-3.5" />
                      </Button>
                      <Button
                        size="icon"
                        variant="ghost"
                        className="h-8 w-8"
                        onClick={() => handleDuplicate(bp)}
                        aria-label={`Duplicate ${bp.name}`}
                      >
                        <Copy className="h-3.5 w-3.5" />
                      </Button>
                      <Button
                        size="icon"
                        variant="ghost"
                        className="h-8 w-8"
                        onClick={() => setDeletingBlueprint(bp)}
                        aria-label={`Delete ${bp.name}`}
                      >
                        <Trash2 className="h-3.5 w-3.5" />
                      </Button>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <Badge variant="secondary" className="text-xs">
                      {bp.archetype}
                    </Badge>
                    <span className="text-xs text-muted-foreground tabular-nums">
                      Weight: {bp.weight}
                    </span>
                    {!selectedClientId && (
                      <span className="text-xs text-muted-foreground">
                        Client: {clientNameMap.get(bp.client_id) ?? bp.client_id.slice(0, 8)}
                      </span>
                    )}
                    <span className="text-xs text-muted-foreground">
                      {formatDate(bp.updated_at)}
                    </span>
                  </div>
                  {bp.description && (
                    <p className="text-sm text-muted-foreground line-clamp-2">
                      {bp.description}
                    </p>
                  )}
                </div>
              ))}
            </div>

            {/* Pagination */}
            <div className="flex flex-col sm:flex-row items-center justify-between gap-3 px-4 py-3 border-t">
              <div className="flex items-center gap-3">
                <span className="text-sm text-muted-foreground">
                  {paginationText}
                </span>
                <div className="flex items-center gap-1.5">
                  <label htmlFor="per-page" className="text-xs text-muted-foreground">
                    Per page
                  </label>
                  <select
                    id="per-page"
                    value={perPage}
                    onChange={(e) => handlePageSizeChange(Number(e.target.value))}
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
                  onClick={() => setPage((p) => Math.max(1, p - 1))}
                  disabled={page <= 1}
                >
                  <ChevronLeft className="h-4 w-4" />
                  Prev
                </Button>
                {generatePageNumbers(page, totalPages).map((p, i) =>
                  p === 'ellipsis' ? (
                    <span key={`ellipsis-${i}`} className="px-1 text-muted-foreground">
                      ...
                    </span>
                  ) : (
                    <Button
                      key={p}
                      size="sm"
                      variant={p === page ? 'default' : 'outline'}
                      onClick={() => setPage(p)}
                      className="min-w-[2rem]"
                    >
                      {p}
                    </Button>
                  ),
                )}
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
                  disabled={page >= totalPages}
                >
                  Next
                  <ChevronRight className="h-4 w-4" />
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Single delete confirmation */}
      <ConfirmDialog
        open={deletingBlueprint !== null}
        onOpenChange={(open) => {
          if (!open) setDeletingBlueprint(null)
        }}
        title="Delete Blueprint"
        description={`Are you sure you want to delete "${deletingBlueprint?.name}"? This action cannot be undone.`}
        confirmLabel="Delete"
        variant="destructive"
        loading={deleteMutation.isPending}
        onConfirm={handleDelete}
      />

      {/* Batch delete confirmation */}
      <ConfirmDialog
        open={showBatchDelete}
        onOpenChange={setShowBatchDelete}
        title="Batch Delete Blueprints"
        description={`Are you sure you want to delete ${selectedCount} ${pluralize(selectedCount, 'blueprint')}? This action cannot be undone.`}
        confirmLabel="Delete All"
        variant="destructive"
        loading={batchDeleteMutation.isPending}
        onConfirm={handleBatchDelete}
      />

      {/* Edit modal */}
      <BlueprintFormModal
        key={editingBlueprint?.id ?? 'edit'}
        open={showEditModal}
        onOpenChange={setShowEditModal}
        blueprint={editingBlueprint}
      />

      {/* Create modal */}
      <BlueprintFormModal
        open={showCreateModal}
        onOpenChange={setShowCreateModal}
      />

      {/* Duplicate modal */}
      <BlueprintFormModal
        key={duplicatingBlueprint?.id ?? 'duplicate'}
        open={showDuplicateModal}
        onOpenChange={setShowDuplicateModal}
        duplicateFrom={duplicatingBlueprint}
      />

      {/* Batch assign affix picker dialog */}
      <AffixPickerDialog
        key={batchAssignKey}
        open={showBatchAssign}
        onOpenChange={setShowBatchAssign}
        selectedBlueprintIds={Array.from(selectedIds)}
      />

      {/* Batch edit dialog */}
      <BatchEditDialog
        key={showBatchEdit ? 'open' : 'closed'}
        open={showBatchEdit}
        onOpenChange={setShowBatchEdit}
        blueprints={selectedBlueprints}
        onSuccess={clearSelection}
      />
    </div>
  )
}

function generatePageNumbers(
  current: number,
  total: number,
): (number | 'ellipsis')[] {
  if (total <= 7) {
    return Array.from({ length: total }, (_, i) => i + 1)
  }

  const pages: (number | 'ellipsis')[] = []

  pages.push(1)

  if (current > 3) {
    pages.push('ellipsis')
  }

  const start = Math.max(2, current - 1)
  const end = Math.min(total - 1, current + 1)

  for (let i = start; i <= end; i++) {
    pages.push(i)
  }

  if (current < total - 2) {
    pages.push('ellipsis')
  }

  pages.push(total)

  return pages
}
