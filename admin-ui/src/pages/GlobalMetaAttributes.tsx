import { useEffect, useMemo, useState } from 'react'
import { ChevronLeft, ChevronRight, Pencil, Plus, Trash2 } from 'lucide-react'

import {
  useAffixesList,
  useBlueprintsList,
  useCreateGlobalMetaAttribute,
  useDeleteGlobalMetaAttribute,
  useGlobalMetaAttributesList,
  useUpdateGlobalMetaAttribute,
} from '@/api/generated'
import type { CreateGlobalMetaAttributeRequest, GlobalMetaAttribute, ValueType } from '@/api/generated'

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
import { useToast } from '@/hooks/use-toast'

const PER_PAGE = 20

function pluralize(count: number, singular: string, plural?: string): string {
  if (count === 1) return singular
  return plural ?? `${singular}s`
}

function getPreview(gma: GlobalMetaAttribute): string {
  const p = gma.payload
  switch (gma.valueType) {
    case 'single':
      return String(p.value ?? '\u2014')
    case 'enum': {
      const values = p.values as string[] | undefined
      return values?.join(', ') ?? '\u2014'
    }
    case 'range':
      return `${p.min ?? '?'} \u2013 ${p.max ?? '?'}`
    case 'string': {
      const min = p.minLength
      const max = p.maxLength
      if (min != null && max != null) return `${min} \u2013 ${max} characters`
      if (min != null) return `\u2265 ${min} characters`
      if (max != null) return `\u2264 ${max} characters`
      return '\u2014'
    }
    case 'boolean':
      return String(p.value ?? '\u2014')
    default:
      return '\u2014'
  }
}

const VALUE_TYPE_OPTIONS: { value: ValueType | 'all'; label: string }[] = [
  { value: 'all', label: 'All' },
  { value: 'single', label: 'Single' },
  { value: 'enum', label: 'Enum' },
  { value: 'range', label: 'Range' },
  { value: 'string', label: 'String' },
  { value: 'boolean', label: 'Boolean' },
]

function TableSkeleton() {
  return (
    <div className="space-y-2">
      {Array.from({ length: 8 }).map((_, i) => (
        <div key={i} className="flex items-center gap-4 p-3 border-b">
          <Skeleton className="h-4 w-40" />
          <Skeleton className="h-5 w-16 rounded-md" />
          <Skeleton className="h-4 w-32" />
          <Skeleton className="h-4 w-16" />
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
            <Skeleton className="h-5 w-16 rounded-md" />
            <Skeleton className="h-4 w-1/2" />
          </CardContent>
        </Card>
      ))}
    </div>
  )
}

function EditGlobalMetaAttributeDialog({
  gma,
  open,
  onOpenChange,
}: {
  gma: GlobalMetaAttribute | null
  open: boolean
  onOpenChange: (open: boolean) => void
}) {
  const { toast } = useToast()
  const updateMutation = useUpdateGlobalMetaAttribute()

  const [name, setName] = useState(gma?.name ?? '')
  const [description, setDescription] = useState(gma?.description ?? '')

  const handleSubmit = async () => {
    if (!gma || !name.trim()) return
    try {
      await updateMutation.mutateAsync({
        id: gma.id,
        request: {
          name: name.trim(),
          description: description.trim() || null,
          valueType: gma.valueType,
          ...gma.payload,
        } as Parameters<typeof updateMutation.mutateAsync>[0]['request'],
      })
      toast({ title: 'Global meta attribute updated' })
      onOpenChange(false)
    } catch {
      toast({ title: 'Failed to update global meta attribute', variant: 'destructive' })
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Edit Global Meta Attribute</DialogTitle>
        </DialogHeader>
        <div className="space-y-4">
          <div className="space-y-2">
            <label htmlFor="edit-gma-name" className="text-sm font-medium">
              Name
            </label>
            <Input
              id="edit-gma-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Attribute name"
            />
          </div>
          <div className="space-y-2">
            <label htmlFor="edit-gma-desc" className="text-sm font-medium">
              Description
            </label>
            <Input
              id="edit-gma-desc"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Optional description"
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button
            onClick={handleSubmit}
            disabled={!name.trim() || updateMutation.isPending}
          >
            {updateMutation.isPending ? 'Saving...' : 'Save'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

function CreateGlobalMetaAttributeDialog({
  open,
  onOpenChange,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
}) {
  const { toast } = useToast()
  const createMutation = useCreateGlobalMetaAttribute()

  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [valueType, setValueType] = useState<ValueType>('single')
  const [singleValue, setSingleValue] = useState('')
  const [enumValues, setEnumValues] = useState('')
  const [rangeMin, setRangeMin] = useState('')
  const [rangeMax, setRangeMax] = useState('')
  const [strMinLen, setStrMinLen] = useState('')
  const [strMaxLen, setStrMaxLen] = useState('')
  const [boolVal, setBoolVal] = useState(false)
  const [useDist, setUseDist] = useState(false)
  const [distType, setDistType] = useState<'uniform' | 'normal' | 'exponential'>('uniform')
  const [distStdDev, setDistStdDev] = useState('')
  const [distRate, setDistRate] = useState('')

  function resetForm() {
    setName('')
    setDescription('')
    setValueType('single')
    setSingleValue('')
    setEnumValues('')
    setRangeMin('')
    setRangeMax('')
    setStrMinLen('')
    setStrMaxLen('')
    setBoolVal(false)
    setUseDist(false)
    setDistType('uniform')
    setDistStdDev('')
    setDistRate('')
  }

  function buildDistribution(): { type: string; stdDev?: number; rate?: number } | null {
    if (distType === 'uniform') return { type: 'uniform' }
    if (distType === 'normal') {
      const sd = parseFloat(distStdDev)
      if (isNaN(sd) || sd <= 0) return null
      return { type: 'normal', stdDev: sd }
    }
    const rate = parseFloat(distRate)
    if (isNaN(rate) || rate <= 0) return null
    return { type: 'exponential', rate }
  }

  function buildRequest(): CreateGlobalMetaAttributeRequest | null {
    if (!name.trim()) return null

    switch (valueType) {
      case 'single': {
        const val = parseFloat(singleValue)
        if (isNaN(val)) return null
        const req: CreateGlobalMetaAttributeRequest = {
          name: name.trim(),
          description: description.trim() || null,
          valueType: 'single',
          value: val,
        }
        if (useDist) {
          const dist = buildDistribution()
          if (!dist) return null
          ;(req as Record<string, unknown>).distribution = dist
        }
        return req
      }
      case 'enum': {
        const vals = enumValues.split(',').map((v) => v.trim()).filter(Boolean)
        if (vals.length === 0) return null
        return {
          name: name.trim(),
          description: description.trim() || null,
          valueType: 'enum',
          values: vals,
        }
      }
      case 'range': {
        const min = parseFloat(rangeMin)
        const max = parseFloat(rangeMax)
        if (isNaN(min) || isNaN(max) || min > max) return null
        const req: CreateGlobalMetaAttributeRequest = {
          name: name.trim(),
          description: description.trim() || null,
          valueType: 'range',
          min,
          max,
        }
        if (useDist) {
          const dist = buildDistribution()
          if (!dist) return null
          ;(req as Record<string, unknown>).distribution = dist
        }
        return req
      }
      case 'string': {
        const minL = strMinLen ? parseInt(strMinLen, 10) : undefined
        const maxL = strMaxLen ? parseInt(strMaxLen, 10) : undefined
        if (minL !== undefined && isNaN(minL)) return null
        if (maxL !== undefined && isNaN(maxL)) return null
        return {
          name: name.trim(),
          description: description.trim() || null,
          valueType: 'string',
          ...(minL !== undefined ? { minLength: minL } : {}),
          ...(maxL !== undefined ? { maxLength: maxL } : {}),
        }
      }
      case 'boolean':
        return {
          name: name.trim(),
          description: description.trim() || null,
          valueType: 'boolean',
          value: boolVal,
        }
      default:
        return null
    }
  }

  const request = buildRequest()
  const canSubmit = request !== null

  const handleSubmit = async () => {
    if (!request) return
    try {
      await createMutation.mutateAsync(request)
      toast({ title: 'Global meta attribute created' })
      onOpenChange(false)
    } catch {
      toast({ title: 'Failed to create global meta attribute', variant: 'destructive' })
    }
  }

  return (
    <Dialog
      open={open}
      onOpenChange={(o) => {
        if (!o) resetForm()
        onOpenChange(o)
      }}
    >
      <DialogContent className="max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Create Global Meta Attribute</DialogTitle>
        </DialogHeader>
        <div className="space-y-4">
          <div className="space-y-2">
            <label htmlFor="create-gma-name" className="text-sm font-medium">
              Name
            </label>
            <Input
              id="create-gma-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Attribute name"
            />
          </div>
          <div className="space-y-2">
            <label htmlFor="create-gma-desc" className="text-sm font-medium">
              Description
            </label>
            <Input
              id="create-gma-desc"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Optional"
            />
          </div>
          <div className="space-y-2">
            <label htmlFor="create-gma-vtype" className="text-sm font-medium">
              Value Type
            </label>
            <select
              id="create-gma-vtype"
              value={valueType}
              onChange={(e) => setValueType(e.target.value as ValueType)}
              className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
            >
              {VALUE_TYPE_OPTIONS.filter((o) => o.value !== 'all').map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
          </div>

          {valueType === 'single' && (
            <div className="space-y-2">
              <label htmlFor="create-gma-value" className="text-sm font-medium">
                Value
              </label>
              <Input
                id="create-gma-value"
                type="number"
                value={singleValue}
                onChange={(e) => setSingleValue(e.target.value)}
                placeholder="0"
              />
            </div>
          )}

          {valueType === 'enum' && (
            <div className="space-y-2">
              <label htmlFor="create-gma-values" className="text-sm font-medium">
                Values (comma-separated)
              </label>
              <Input
                id="create-gma-values"
                value={enumValues}
                onChange={(e) => setEnumValues(e.target.value)}
                placeholder="fire, ice, lightning"
              />
            </div>
          )}

          {valueType === 'range' && (
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-2">
                <label htmlFor="create-gma-min" className="text-sm font-medium">
                  Min
                </label>
                <Input
                  id="create-gma-min"
                  type="number"
                  value={rangeMin}
                  onChange={(e) => setRangeMin(e.target.value)}
                  placeholder="0"
                />
              </div>
              <div className="space-y-2">
                <label htmlFor="create-gma-max" className="text-sm font-medium">
                  Max
                </label>
                <Input
                  id="create-gma-max"
                  type="number"
                  value={rangeMax}
                  onChange={(e) => setRangeMax(e.target.value)}
                  placeholder="100"
                />
              </div>
            </div>
          )}

          {valueType === 'string' && (
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-2">
                <label htmlFor="create-gma-minlen" className="text-sm font-medium">
                  Min Length
                </label>
                <Input
                  id="create-gma-minlen"
                  type="number"
                  value={strMinLen}
                  onChange={(e) => setStrMinLen(e.target.value)}
                  placeholder="0"
                />
              </div>
              <div className="space-y-2">
                <label htmlFor="create-gma-maxlen" className="text-sm font-medium">
                  Max Length
                </label>
                <Input
                  id="create-gma-maxlen"
                  type="number"
                  value={strMaxLen}
                  onChange={(e) => setStrMaxLen(e.target.value)}
                  placeholder="100"
                />
              </div>
            </div>
          )}

          {valueType === 'boolean' && (
            <div className="space-y-2">
              <label className="text-sm font-medium block">Value</label>
              <div className="flex items-center gap-3">
                <label className="flex items-center gap-2 text-sm">
                  <input
                    type="radio"
                    name="create-bool-val"
                    checked={boolVal}
                    onChange={() => setBoolVal(true)}
                    className="h-4 w-4"
                  />
                  True
                </label>
                <label className="flex items-center gap-2 text-sm">
                  <input
                    type="radio"
                    name="create-bool-val"
                    checked={!boolVal}
                    onChange={() => setBoolVal(false)}
                    className="h-4 w-4"
                  />
                  False
                </label>
              </div>
            </div>
          )}

          {(valueType === 'single' || valueType === 'range') && (
            <div className="space-y-3 border rounded-md p-3">
              <label className="flex items-center gap-2 text-sm font-medium">
                <input
                  type="checkbox"
                  checked={useDist}
                  onChange={(e) => setUseDist(e.target.checked)}
                  className="rounded"
                />
                Enable Distribution
              </label>
              {useDist && (
                <>
                  <div className="space-y-2">
                    <label htmlFor="create-dist-type" className="text-sm font-medium">
                      Distribution Type
                    </label>
                    <select
                      id="create-dist-type"
                      value={distType}
                      onChange={(e) =>
                        setDistType(e.target.value as 'uniform' | 'normal' | 'exponential')
                      }
                      className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                    >
                      <option value="uniform">Uniform</option>
                      <option value="normal">Normal</option>
                      <option value="exponential">Exponential</option>
                    </select>
                  </div>
                  {distType === 'normal' && (
                    <div className="space-y-2">
                      <label htmlFor="create-dist-stddev" className="text-sm font-medium">
                        Std Dev
                      </label>
                      <Input
                        id="create-dist-stddev"
                        type="number"
                        step="0.01"
                        value={distStdDev}
                        onChange={(e) => setDistStdDev(e.target.value)}
                        placeholder="1.0"
                      />
                    </div>
                  )}
                  {distType === 'exponential' && (
                    <div className="space-y-2">
                      <label htmlFor="create-dist-rate" className="text-sm font-medium">
                        Rate
                      </label>
                      <Input
                        id="create-dist-rate"
                        type="number"
                        step="0.01"
                        value={distRate}
                        onChange={(e) => setDistRate(e.target.value)}
                        placeholder="1.0"
                      />
                    </div>
                  )}
                </>
              )}
            </div>
          )}
        </div>
        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => onOpenChange(false)}
            disabled={createMutation.isPending}
          >
            Cancel
          </Button>
          <Button onClick={handleSubmit} disabled={!canSubmit || createMutation.isPending}>
            {createMutation.isPending ? 'Creating...' : 'Create'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

export default function GlobalMetaAttributes() {
  const { toast } = useToast()

  const [page, setPage] = useState(1)
  const [search, setSearch] = useState('')
  const [debouncedSearch, setDebouncedSearch] = useState('')
  const [typeFilter, setTypeFilter] = useState<ValueType | 'all'>('all')

  const [editingGma, setEditingGma] = useState<GlobalMetaAttribute | null>(null)
  const [showEditDialog, setShowEditDialog] = useState(false)

  const [showCreateDialog, setShowCreateDialog] = useState(false)

  const [deletingGma, setDeletingGma] = useState<GlobalMetaAttribute | null>(null)
  const deleteMutation = useDeleteGlobalMetaAttribute()

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
  } = useGlobalMetaAttributesList({
    page,
    perPage: PER_PAGE,
    search: debouncedSearch || undefined,
  })

  const allItems = useMemo<GlobalMetaAttribute[]>(
    () => (data?.data ?? []) as GlobalMetaAttribute[],
    [data?.data],
  )

  const filteredItems = useMemo(
    () =>
      typeFilter === 'all'
        ? allItems
        : allItems.filter((a) => a.valueType === typeFilter),
    [allItems, typeFilter],
  )

  const totalCount = data?.total ?? 0
  const totalPages = Math.max(1, Math.ceil(totalCount / PER_PAGE))

  let paginationText: string
  if (totalCount > 0) {
    paginationText = `Page ${page} of ${totalPages} (${totalCount} total)`
  } else if (filteredItems.length > 0) {
    paginationText = pluralize(filteredItems.length, 'result')
  } else {
    paginationText = 'No results'
  }

  // Fetch blueprints and affixes for usage count computation
  const { data: bpData } = useBlueprintsList({ perPage: 500 })
  const { data: affData } = useAffixesList({ perPage: 500 })

  const usageCounts = useMemo(() => {
    const counts = new Map<string, number>()
    const bpList = (bpData?.data ?? []) as { id: string; attributes: Record<string, unknown> }[]
    const affList = (affData?.data ?? []) as { id: string; attribute: unknown }[]

    for (const bp of bpList) {
      const attrs = bp.attributes
      if (!attrs) continue
      for (const v of Object.values(attrs)) {
        if (typeof v === 'object' && v !== null && '$ref_id' in (v as Record<string, unknown>)) {
          const refId = (v as Record<string, unknown>).$ref_id as string
          counts.set(refId, (counts.get(refId) ?? 0) + 1)
        }
      }
    }
    for (const aff of affList) {
      const attr = aff.attribute
      if (typeof attr === 'object' && attr !== null && '$ref_id' in (attr as Record<string, unknown>)) {
        const refId = (attr as Record<string, unknown>).$ref_id as string
        counts.set(refId, (counts.get(refId) ?? 0) + 1)
      }
    }
    return counts
  }, [bpData?.data, affData?.data])

  function handleEdit(gma: GlobalMetaAttribute) {
    setEditingGma(gma)
    setShowEditDialog(true)
  }

  function getDeleteDescription(): string {
    if (!deletingGma) return ''
    const count = usageCounts.get(deletingGma.id) ?? 0
    return `Are you sure you want to delete "${deletingGma.name}"? This action cannot be undone.\n\nUsed by ${count} ${pluralize(count, 'blueprint/affix', 'blueprints/affixes')}.`
  }

  async function handleDelete() {
    if (!deletingGma) return
    try {
      await deleteMutation.mutateAsync(deletingGma.id)
      toast({ title: `Deleted "${deletingGma.name}"` })
      setDeletingGma(null)
    } catch {
      toast({ title: 'Failed to delete global meta attribute', variant: 'destructive' })
    }
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold">Global Meta Attributes</h2>
        <Button onClick={() => setShowCreateDialog(true)}>
          <Plus className="h-4 w-4 mr-2" />
          Create Global Meta Attribute
        </Button>
      </div>

      {/* Filter bar */}
      <div className="flex flex-col sm:flex-row gap-3">
        <div className="flex items-center gap-2">
          <label htmlFor="type-filter" className="text-sm font-medium">
            Value Type
          </label>
          <select
            id="type-filter"
            value={typeFilter}
            onChange={(e) => {
              setTypeFilter(e.target.value as ValueType | 'all')
              setPage(1)
            }}
            className="h-9 rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
          >
            {VALUE_TYPE_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
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

      {/* Error state */}
      {isError && (
        <Card>
          <CardContent className="p-8 text-center">
            <p className="text-destructive font-medium">
              Failed to load global meta attributes
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
                ? `No global meta attributes matching "${debouncedSearch}"`
                : 'No global meta attributes found'}
            </p>
            <p className="text-sm text-muted-foreground mt-1">
              Create your first global meta attribute to get started
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
                    <TableHead>Name</TableHead>
                    <TableHead>Value Type</TableHead>
                    <TableHead>Preview</TableHead>
                    <TableHead>Usage Count</TableHead>
                    <TableHead className="w-20 text-right">Actions</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {filteredItems.map((gma) => {
                    const count = usageCounts.get(gma.id) ?? 0
                    return (
                      <TableRow key={gma.id}>
                        <TableCell className="font-medium">
                          {gma.name}
                        </TableCell>
                        <TableCell>
                          <Badge variant="outline">{gma.valueType}</Badge>
                        </TableCell>
                        <TableCell className="text-muted-foreground text-sm font-mono">
                          {getPreview(gma)}
                        </TableCell>
                        <TableCell className="text-muted-foreground text-sm">
                          {count > 0
                            ? `${count} ${pluralize(count, 'reference')}`
                            : '\u2014'}
                        </TableCell>
                        <TableCell className="text-right">
                          <div className="flex justify-end gap-1">
                            <Button
                              size="icon"
                              variant="ghost"
                              onClick={() => handleEdit(gma)}
                              aria-label={`Edit ${gma.name}`}
                            >
                              <Pencil className="h-4 w-4" />
                            </Button>
                            <Button
                              size="icon"
                              variant="ghost"
                              onClick={() => setDeletingGma(gma)}
                              aria-label={`Delete ${gma.name}`}
                            >
                              <Trash2 className="h-4 w-4" />
                            </Button>
                          </div>
                        </TableCell>
                      </TableRow>
                    )
                  })}
                  {filteredItems.length === 0 && (
                    <TableRow>
                      <TableCell
                        colSpan={5}
                        className="text-center py-8 text-muted-foreground"
                      >
                        No global meta attributes on this page
                      </TableCell>
                    </TableRow>
                  )}
                </TableBody>
              </Table>
            </div>

            {/* Mobile cards */}
            <div className="md:hidden divide-y">
              {filteredItems.map((gma) => {
                const count = usageCounts.get(gma.id) ?? 0
                return (
                  <div key={gma.id} className="px-4 py-3 space-y-2">
                    <div className="flex items-center justify-between">
                      <span className="font-medium">{gma.name}</span>
                      <div className="flex gap-1">
                        <Button
                          size="icon"
                          variant="ghost"
                          className="h-8 w-8"
                          onClick={() => handleEdit(gma)}
                          aria-label={`Edit ${gma.name}`}
                        >
                          <Pencil className="h-3.5 w-3.5" />
                        </Button>
                        <Button
                          size="icon"
                          variant="ghost"
                          className="h-8 w-8"
                          onClick={() => setDeletingGma(gma)}
                          aria-label={`Delete ${gma.name}`}
                        >
                          <Trash2 className="h-3.5 w-3.5" />
                        </Button>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <Badge variant="outline" className="text-xs">
                        {gma.valueType}
                      </Badge>
                      <span className="text-xs text-muted-foreground font-mono">
                        {getPreview(gma)}
                      </span>
                    </div>
                    {count > 0 && (
                      <p className="text-xs text-muted-foreground">
                        {count} {pluralize(count, 'reference')}
                      </p>
                    )}
                  </div>
                )
              })}
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
        open={deletingGma !== null}
        onOpenChange={(open) => {
          if (!open) setDeletingGma(null)
        }}
        title="Delete Global Meta Attribute"
        description={getDeleteDescription()}
        confirmLabel="Delete"
        variant="destructive"
        loading={deleteMutation.isPending}
        onConfirm={handleDelete}
      />

      {/* Edit dialog */}
      <EditGlobalMetaAttributeDialog
        key={editingGma?.id ?? 'none'}
        gma={editingGma}
        open={showEditDialog}
        onOpenChange={setShowEditDialog}
      />

      {/* Create dialog */}
      <CreateGlobalMetaAttributeDialog
        open={showCreateDialog}
        onOpenChange={setShowCreateDialog}
      />
    </div>
  )
}
