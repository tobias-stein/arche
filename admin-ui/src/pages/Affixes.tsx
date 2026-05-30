import { useEffect, useMemo, useState } from 'react'
import type { MouseEvent as ReactMouseEvent } from 'react'
import { Link } from 'react-router-dom'
import { ChevronLeft, ChevronRight, Pencil, Plus, Trash2 } from 'lucide-react'

import {
  useAffixesList,
  useBatchAssignAffixes,
  useBatchDeleteAffixes,
  useBlueprintsList,
  useCreateAffix,
  useDeleteAffix,
  useUpdateAffix,
} from '@/api/generated'
import type {
  Affix,
  AffixAttribute,
  Blueprint,
} from '@/api/generated'
import { AffixCreateEditDialog } from './AffixCreateEditDialog'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { ConfirmDialog } from '@/components/ConfirmDialog'
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
import { formatDate } from '@/lib/utils'
import { useToast } from '@/hooks/use-toast'

const PER_PAGE = 20

function pluralize(count: number, singular: string, plural?: string): string {
  if (count === 1) return singular
  return plural ?? `${singular}s`
}

function getAttributeName(attribute: AffixAttribute): string {
  if ('$ref_id' in attribute && attribute.$ref_id) {
    return attribute.$ref_id
  }
  return 'name' in attribute ? attribute.name : '\u2014'
}

function getValueTypeDisplay(attribute: AffixAttribute): string {
  if ('$ref_id' in attribute) {
    return 'global'
  }
  return attribute.value_type
}

function isRefAttribute(attr: AffixAttribute): boolean {
  return '$ref_id' in attr
}

function TableSkeleton() {
  return (
    <div className="space-y-2">
      {Array.from({ length: 8 }).map((_, i) => (
        <div key={i} className="flex items-center gap-4 p-3 border-b">
          <Skeleton className="h-4 w-4 rounded-sm" />
          <Skeleton className="h-4 w-40" />
          <Skeleton className="h-5 w-16 rounded-md" />
          <Skeleton className="h-4 w-24" />
          <Skeleton className="h-5 w-20 rounded-md" />
          <Skeleton className="h-4 w-28" />
          <div className="flex gap-2 ml-auto">
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

function BlueprintPickerDialog({
  open,
  onOpenChange,
  selectedAffixIds,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  selectedAffixIds: string[]
}) {
  const { toast } = useToast()
  const { data: bpData, isLoading } = useBlueprintsList(
    { per_page: 100 },
  )
  const batchAssignMutation = useBatchAssignAffixes()

  const [selectedBpIds, setSelectedBpIds] = useState<Set<string>>(new Set())

  const blueprints: Blueprint[] = (bpData?.data ?? []) as Blueprint[]

  const allSelected =
    blueprints.length > 0 && blueprints.every((b) => selectedBpIds.has(b.id))

  function toggleBlueprint(id: string) {
    setSelectedBpIds((prev) => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  }

  function toggleAll() {
    if (allSelected) {
      setSelectedBpIds(new Set())
    } else {
      setSelectedBpIds(new Set(blueprints.map((b) => b.id)))
    }
  }

  async function handleAssign() {
    if (selectedBpIds.size === 0) return
    try {
      await batchAssignMutation.mutateAsync({
        blueprint_ids: Array.from(selectedBpIds),
        affix_ids: Array.from(selectedAffixIds),
        weight: 1,
      })
      toast({
        title: `Assigned ${pluralize(selectedAffixIds.length, 'affix', 'affixes')} to ${pluralize(selectedBpIds.size, 'blueprint')}`,
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
          <DialogTitle>Assign to Blueprints</DialogTitle>
        </DialogHeader>
        <div className="max-h-64 overflow-y-auto border rounded-md">
          {isLoading ? (
            <div className="p-4 space-y-2">
              {Array.from({ length: 5 }).map((_, i) => (
                <Skeleton key={i} className="h-6 w-full" />
              ))}
            </div>
          ) : blueprints.length === 0 ? (
            <p className="p-4 text-sm text-muted-foreground">
              No blueprints found
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
              {blueprints.map((bp) => (
                <label
                  key={bp.id}
                  className="flex items-center gap-2 px-3 py-2 text-sm hover:bg-muted/50 cursor-pointer"
                >
                  <input
                    type="checkbox"
                    checked={selectedBpIds.has(bp.id)}
                    onChange={() => toggleBlueprint(bp.id)}
                    className="rounded"
                  />
                  {bp.name}
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
            disabled={selectedBpIds.size === 0 || batchAssignMutation.isPending}
          >
            {batchAssignMutation.isPending
              ? 'Assigning...'
              : `Assign (${pluralize(selectedAffixIds.length, 'affix', 'affixes')})`}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

export default function Affixes() {
  const { toast } = useToast()

  const [page, setPage] = useState(1)
  const [search, setSearch] = useState('')
  const [debouncedSearch, setDebouncedSearch] = useState('')
  const [typeFilter, setTypeFilter] = useState<'all' | 'prefix' | 'suffix'>('all')
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set())
  const [lastCheckedIdx, setLastCheckedIdx] = useState<number | null>(null)

  const [editingAffix, setEditingAffix] = useState<Affix | null>(null)
  const [showEditDialog, setShowEditDialog] = useState(false)

  const [showCreateDialog, setShowCreateDialog] = useState(false)
  const createMutation = useCreateAffix()
  const updateMutation = useUpdateAffix()

  const [deletingAffix, setDeletingAffix] = useState<Affix | null>(null)
  const deleteMutation = useDeleteAffix()

  const [showBatchDelete, setShowBatchDelete] = useState(false)
  const batchDeleteMutation = useBatchDeleteAffixes()

  const [showBlueprintPicker, setShowBlueprintPicker] = useState(false)
  const [pickerKey, setPickerKey] = useState(0)

  useEffect(() => {
    const timeout = setTimeout(() => {
      setDebouncedSearch(search)
      setPage(1)
    }, 300)
    return () => clearTimeout(timeout)
  }, [search])

  const {
    data,
    isLoading,
    isError,
    error,
  } = useAffixesList({
    page,
    per_page: PER_PAGE,
    search: debouncedSearch || undefined,
  })

  const allItems = useMemo<Affix[]>(
    () => (data?.data ?? []) as Affix[],
    [data?.data],
  )

  const filteredItems = useMemo(
    () =>
      typeFilter === 'all'
        ? allItems
        : allItems.filter((a) => a.location === typeFilter),
    [allItems, typeFilter],
  )

  const totalCount = data?.total ?? 0
  const totalPages = Math.max(1, Math.ceil(totalCount / PER_PAGE))

  const selectedCount = selectedIds.size

  const paginationText =
    totalCount > 0
      ? `Page ${page} of ${totalPages} (${totalCount} total)`
      : filteredItems.length > 0
        ? pluralize(filteredItems.length, 'result')
        : 'No results'

  function clearSelection() {
    setSelectedIds(new Set())
    setLastCheckedIdx(null)
  }

  function handleCheckClick(affix: Affix, idx: number, e: ReactMouseEvent) {
    e.stopPropagation()
    setSelectedIds((prev) => {
      const next = new Set(prev)

      if (e.shiftKey && lastCheckedIdx !== null) {
        const start = Math.min(lastCheckedIdx, idx)
        const end = Math.max(lastCheckedIdx, idx)
        for (let i = start; i <= end; i++) {
          if (i < filteredItems.length) {
            next.add(filteredItems[i].id)
          }
        }
      } else {
        if (next.has(affix.id)) next.delete(affix.id)
        else next.add(affix.id)
      }

      return next
    })
    setLastCheckedIdx(idx)
  }

  function handleSelectAll() {
    if (filteredItems.length === 0) return
    const allSelected = filteredItems.every((a) => selectedIds.has(a.id))
    setSelectedIds((prev) => {
      const next = new Set(prev)
      filteredItems.forEach((a) => {
        if (allSelected) next.delete(a.id)
        else next.add(a.id)
      })
      return next
    })
  }

  function handleEdit(affix: Affix) {
    setEditingAffix(affix)
    setShowEditDialog(true)
  }

  async function handleDelete() {
    if (!deletingAffix) return
    try {
      await deleteMutation.mutateAsync(deletingAffix.id)
      toast({ title: `Deleted "${deletingAffix.name}"` })
      setDeletingAffix(null)
      setSelectedIds((prev) => {
        const next = new Set(prev)
        next.delete(deletingAffix.id)
        return next
      })
    } catch {
      toast({ title: 'Failed to delete affix', variant: 'destructive' })
    }
  }

  async function handleBatchDelete() {
    if (selectedCount === 0) return
    try {
      await batchDeleteMutation.mutateAsync({
        ids: Array.from(selectedIds),
      })
      toast({ title: `Deleted ${pluralize(selectedCount, 'affix', 'affixes')}` })
      setShowBatchDelete(false)
      clearSelection()
    } catch {
      toast({ title: 'Failed to batch delete', variant: 'destructive' })
    }
  }

  const allPageSelected =
    filteredItems.length > 0 &&
    filteredItems.every((a) => selectedIds.has(a.id))

  const selectAllLabel = allPageSelected ? 'Deselect all' : 'Select all'

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold">Affixes</h2>
        <Button onClick={() => setShowCreateDialog(true)}>
          <Plus className="h-4 w-4" />
          Create Affix
        </Button>
      </div>

      {/* Filter bar */}
      <div className="flex flex-col sm:flex-row gap-3">
        <div className="flex items-center gap-2">
          <label htmlFor="type-filter" className="text-sm font-medium">
            Type
          </label>
          <select
            id="type-filter"
            value={typeFilter}
            onChange={(e) => {
              setTypeFilter(e.target.value as 'all' | 'prefix' | 'suffix')
              setPage(1)
              setSelectedIds(new Set())
              setLastCheckedIdx(null)
            }}
            className="h-9 rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
          >
            <option value="all">All</option>
            <option value="prefix">Prefix</option>
            <option value="suffix">Suffix</option>
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
                setPickerKey(k => k + 1)
                setShowBlueprintPicker(true)
              }}
            >
              Assign to Blueprints
            </Button>
          </div>
        </div>
      )}

      {/* Error state */}
      {isError && (
        <Card>
          <CardContent className="p-8 text-center">
            <p className="text-destructive font-medium">
              Failed to load affixes
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
      {!isLoading && !isError && allItems.length === 0 && (
        <Card>
          <CardContent className="p-8 text-center">
            <p className="text-muted-foreground">
              {debouncedSearch
                ? `No affixes matching "${debouncedSearch}"`
                : 'No affixes found'}
            </p>
            <p className="text-sm text-muted-foreground mt-1">
              Create your first affix to get started
            </p>
          </CardContent>
        </Card>
      )}

      {/* Data table */}
      {!isLoading && !isError && allItems.length > 0 && (
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
                    <TableHead>Name</TableHead>
                    <TableHead>Type</TableHead>
                    <TableHead>Attribute Name</TableHead>
                    <TableHead>Value Type</TableHead>
                    <TableHead>Updated</TableHead>
                    <TableHead className="w-20 text-right">Actions</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {filteredItems.map((affix, idx) => (
                    <TableRow
                      key={affix.id}
                      data-state={
                        selectedIds.has(affix.id) ? 'selected' : undefined
                      }
                    >
                      <TableCell>
                        <input
                          type="checkbox"
                          checked={selectedIds.has(affix.id)}
                          onClick={(e) => handleCheckClick(affix, idx, e)}
                          onChange={() => {}}
                          aria-label={`Select ${affix.name}`}
                          className="rounded"
                        />
                      </TableCell>
                      <TableCell className="font-medium">
                        <Link
                          to={`/affixes/${affix.id}`}
                          className="text-primary hover:underline"
                        >
                          {affix.name}
                        </Link>
                      </TableCell>
                      <TableCell>
                        <Badge
                          variant={
                            affix.location === 'prefix'
                              ? 'default'
                              : 'secondary'
                          }
                        >
                          {affix.location}
                        </Badge>
                      </TableCell>
                      <TableCell className="text-muted-foreground">
                        {isRefAttribute(affix.attribute) ? (
                          <span className="font-mono text-xs">
                            {getAttributeName(affix.attribute)}
                          </span>
                        ) : (
                          getAttributeName(affix.attribute)
                        )}
                      </TableCell>
                      <TableCell>
                        <Badge variant="outline">
                          {getValueTypeDisplay(affix.attribute)}
                        </Badge>
                      </TableCell>
                      <TableCell className="text-muted-foreground text-sm">
                        {formatDate(affix.updated_at)}
                      </TableCell>
                      <TableCell className="text-right">
                        <div className="flex justify-end gap-1">
                          <Button
                            size="icon"
                            variant="ghost"
                            onClick={() => handleEdit(affix)}
                            aria-label={`Edit ${affix.name}`}
                          >
                            <Pencil className="h-4 w-4" />
                          </Button>
                          <Button
                            size="icon"
                            variant="ghost"
                            onClick={() => setDeletingAffix(affix)}
                            aria-label={`Delete ${affix.name}`}
                          >
                            <Trash2 className="h-4 w-4" />
                          </Button>
                        </div>
                      </TableCell>
                    </TableRow>
                  ))}
                  {filteredItems.length === 0 && (
                    <TableRow>
                      <TableCell
                        colSpan={7}
                        className="text-center py-8 text-muted-foreground"
                      >
                        No {typeFilter} affixes on this page
                      </TableCell>
                    </TableRow>
                  )}
                </TableBody>
              </Table>
            </div>

            {/* Mobile cards */}
            <div className="md:hidden divide-y">
              {filteredItems.map((affix, idx) => (
                <div
                  key={affix.id}
                  className={`px-4 py-3 space-y-2 ${
                    selectedIds.has(affix.id) ? 'bg-muted/50' : ''
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <input
                        type="checkbox"
                        checked={selectedIds.has(affix.id)}
                        onClick={(e) => handleCheckClick(affix, idx, e)}
                        onChange={() => {}}
                        aria-label={`Select ${affix.name}`}
                        className="rounded"
                      />
                      <span className="font-medium">
                        <Link
                          to={`/affixes/${affix.id}`}
                          className="text-primary hover:underline"
                        >
                          {affix.name}
                        </Link>
                      </span>
                    </div>
                    <div className="flex gap-1">
                      <Button
                        size="icon"
                        variant="ghost"
                        className="h-8 w-8"
                        onClick={() => handleEdit(affix)}
                        aria-label={`Edit ${affix.name}`}
                      >
                        <Pencil className="h-3.5 w-3.5" />
                      </Button>
                      <Button
                        size="icon"
                        variant="ghost"
                        className="h-8 w-8"
                        onClick={() => setDeletingAffix(affix)}
                        aria-label={`Delete ${affix.name}`}
                      >
                        <Trash2 className="h-3.5 w-3.5" />
                      </Button>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <Badge
                      variant={
                        affix.location === 'prefix' ? 'default' : 'secondary'
                      }
                      className="text-xs"
                    >
                      {affix.location}
                    </Badge>
                    <Badge variant="outline" className="text-xs">
                      {getValueTypeDisplay(affix.attribute)}
                    </Badge>
                    <span className="text-xs text-muted-foreground">
                      {formatDate(affix.updated_at)}
                    </span>
                  </div>
                  <p className="text-sm text-muted-foreground">
                    {getAttributeName(affix.attribute)}
                  </p>
                </div>
              ))}
            </div>

            {/* Pagination */}
            <div className="flex items-center justify-between px-4 py-3 border-t">
              <span className="text-sm text-muted-foreground">
                {paginationText}
              </span>
              <div className="flex items-center gap-2">
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => setPage((p) => Math.max(1, p - 1))}
                  disabled={page <= 1}
                >
                  <ChevronLeft className="h-4 w-4" />
                  Previous
                </Button>
                <span className="text-sm tabular-nums min-w-[3rem] text-center">
                  {page}
                </span>
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
        open={deletingAffix !== null}
        onOpenChange={(open) => {
          if (!open) setDeletingAffix(null)
        }}
        title="Delete Affix"
        description={`Are you sure you want to delete "${deletingAffix?.name}"? This action cannot be undone.`}
        confirmLabel="Delete"
        variant="destructive"
        loading={deleteMutation.isPending}
        onConfirm={handleDelete}
      />

      {/* Batch delete confirmation */}
      <ConfirmDialog
        open={showBatchDelete}
        onOpenChange={setShowBatchDelete}
        title="Batch Delete Affixes"
        description={`Are you sure you want to delete ${pluralize(selectedCount, 'affix', 'affixes')}? This action cannot be undone.`}
        confirmLabel="Delete All"
        variant="destructive"
        loading={batchDeleteMutation.isPending}
        onConfirm={handleBatchDelete}
      />

      {/* Edit dialog */}
      <AffixCreateEditDialog
        key={editingAffix?.id ?? 'edit-new'}
        affix={editingAffix}
        open={showEditDialog}
        onOpenChange={setShowEditDialog}
        onSubmit={async (data) => {
          if (!editingAffix) return
          try {
            await updateMutation.mutateAsync({
              id: editingAffix.id,
              request: {
                name: data.name,
                type: data.location,
                description: data.description,
                attribute: data.attribute,
              },
            })
            toast({ title: 'Affix updated' })
            setShowEditDialog(false)
          } catch {
            toast({
              title: 'Failed to update affix',
              variant: 'destructive',
            })
          }
        }}
        isPending={updateMutation.isPending}
      />

      {/* Create dialog */}
      <AffixCreateEditDialog
        affix={null}
        open={showCreateDialog}
        onOpenChange={setShowCreateDialog}
        onSubmit={async (data) => {
          try {
            await createMutation.mutateAsync({
              name: data.name,
              type: data.location,
              description: data.description,
              attribute: data.attribute,
            })
            toast({ title: 'Affix created' })
            setShowCreateDialog(false)
          } catch {
            toast({
              title: 'Failed to create affix',
              variant: 'destructive',
            })
          }
        }}
        isPending={createMutation.isPending}
      />

      {/* Blueprint picker dialog */}
      <BlueprintPickerDialog
        key={pickerKey}
        open={showBlueprintPicker}
        onOpenChange={setShowBlueprintPicker}
        selectedAffixIds={Array.from(selectedIds)}
      />
    </div>
  )
}
