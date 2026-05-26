import { useCallback, useState } from 'react'
import {
  DndContext,
  type DragEndEvent,
  KeyboardSensor,
  PointerSensor,
  closestCenter,
  useSensor,
  useSensors,
} from '@dnd-kit/core'
import {
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { GripVertical, Plus, Trash2 } from 'lucide-react'

import {
  useAffixesList,
  useCreateBlueprint,
  useGlobalMetaAttributesList,
  useUpdateBlueprint,
} from '@/api/generated'
import type {
  Affix,
  AffixPoolEntry,
  Blueprint,
  BlueprintAffixConfig,
  BlueprintAttribute,
  CreateBlueprintRequest,
  GlobalMetaAttribute,
  InlineAttributeDef,
  RefAttribute,
  ValueType,
} from '@/api/generated'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { Input } from '@/components/ui/input'
import { Skeleton } from '@/components/ui/skeleton'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { useToast } from '@/hooks/use-toast'
import { createBlueprintSchema } from '@/lib/blueprint-schema'

type AttributeEntry = {
  key: string
  attribute: BlueprintAttribute
}

const VALUE_TYPES: ValueType[] = ['single', 'enum', 'range', 'string', 'boolean']
const errorClass = 'text-[0.8rem] font-medium text-destructive'

function getValueTypeDisplay(attr: BlueprintAttribute): string {
  if ('$ref_id' in attr) return 'global'
  return attr.valueType
}

function isRefAttribute(attr: BlueprintAttribute): attr is RefAttribute {
  return '$ref_id' in attr
}

function getAttributePreview(attr: BlueprintAttribute): string {
  if ('$ref_id' in attr) return '\u2014'
  switch (attr.valueType) {
    case 'single':
      return String(attr.value)
    case 'enum':
      return attr.values.join(', ')
    case 'range':
      return `${attr.min} \u2013 ${attr.max}`
    case 'string': {
      const parts: string[] = []
      if (attr.minLength !== undefined) parts.push(`min: ${attr.minLength}`)
      if (attr.maxLength !== undefined) parts.push(`max: ${attr.maxLength}`)
      return parts.length > 0 ? parts.join(', ') : '\u2014'
    }
    case 'boolean':
      return String(attr.value)
    default:
      return '\u2014'
  }
}

function SortableRow({
  id,
  children,
}: {
  id: string
  children: React.ReactNode
}) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id })

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  }

  return (
    <TableRow
      ref={setNodeRef}
      style={style}
      className={isDragging ? 'z-50' : ''}
    >
      <TableCell className="w-8">
        <button
          type="button"
          className="cursor-grab touch-none"
          {...attributes}
          {...listeners}
          aria-label="Drag to reorder"
        >
          <GripVertical className="h-4 w-4 text-muted-foreground" />
        </button>
      </TableCell>
      {children}
    </TableRow>
  )
}

function buildDistribution(distType: string, stdDev: string, rate: string) {
  if (distType === 'none') return undefined
  if (distType === 'uniform') return { type: 'uniform' } as const
  if (distType === 'normal')
    return { type: 'normal', stdDev: Number(stdDev) || 1 } as const
  if (distType === 'exponential')
    return { type: 'exponential', rate: Number(rate) || 1 } as const
  return undefined
}

function reorderPool(
  items: AffixPoolEntry[],
  setter: (items: AffixPoolEntry[]) => void,
  prefix: string,
): (event: DragEndEvent) => void {
  return (event) => {
    const { active, over } = event
    if (!over || active.id === over.id) return

    const updated = [...items]
    const oldIdx = Number((active.id as string).replace(`${prefix}-`, ''))
    const newIdx = Number((over.id as string).replace(`${prefix}-`, ''))

    if (
      !isNaN(oldIdx) &&
      !isNaN(newIdx) &&
      oldIdx >= 0 &&
      newIdx >= 0 &&
      oldIdx < items.length &&
      newIdx < items.length
    ) {
      const [moved] = updated.splice(oldIdx, 1)
      updated.splice(newIdx, 0, moved)
      setter(updated)
    }
  }
}

function InlineAttributeForm({
  onSubmit,
  onCancel,
}: {
  onSubmit: (key: string, attr: InlineAttributeDef) => void
  onCancel: () => void
}) {
  const [attrKey, setAttrKey] = useState('')
  const [valueType, setValueType] = useState<ValueType>('single')
  const [singleValue, setSingleValue] = useState('')
  const [enumValuesText, setEnumValuesText] = useState('')
  const [rangeMin, setRangeMin] = useState('')
  const [rangeMax, setRangeMax] = useState('')
  const [stringMinLen, setStringMinLen] = useState('')
  const [stringMaxLen, setStringMaxLen] = useState('')
  const [boolValue, setBoolValue] = useState('true')
  const [distType, setDistType] = useState<string>('none')
  const [distStdDev, setDistStdDev] = useState('')
  const [distRate, setDistRate] = useState('')
  const [description, setDescription] = useState('')
  const [error, setError] = useState('')

  const handleSubmit = () => {
    if (!attrKey.trim()) {
      setError('Attribute key is required')
      return
    }

    let payload: InlineAttributeDef
    switch (valueType) {
      case 'single': {
        const v = Number(singleValue)
        if (isNaN(v)) {
          setError('Value must be a number')
          return
        }
        payload = {
          valueType: 'single',
          value: v,
          distribution: buildDistribution(distType, distStdDev, distRate),
        }
        break
      }
      case 'enum': {
        const values = enumValuesText
          .split(',')
          .map((s) => s.trim())
          .filter(Boolean)
        if (values.length === 0) {
          setError('Enum must have at least one value')
          return
        }
        payload = { valueType: 'enum', values }
        break
      }
      case 'range': {
        const min = Number(rangeMin)
        const max = Number(rangeMax)
        if (isNaN(min) || isNaN(max)) {
          setError('Min and max must be numbers')
          return
        }
        payload = {
          valueType: 'range',
          min,
          max,
          distribution: buildDistribution(distType, distStdDev, distRate),
        }
        break
      }
      case 'string':
        payload = { valueType: 'string' }
        if (stringMinLen) payload.minLength = Number(stringMinLen)
        if (stringMaxLen) payload.maxLength = Number(stringMaxLen)
        break
      case 'boolean':
        payload = { valueType: 'boolean', value: boolValue === 'true' }
        break
    }

    if (description.trim()) {
      payload = { ...payload, description: description.trim() }
    }

    onSubmit(attrKey.trim(), payload)
  }

  return (
    <div className="space-y-3 rounded-md border p-4">
      <div className="space-y-2">
        <label className="text-sm font-medium" htmlFor="attr-key">
          Key
        </label>
        <Input
          id="attr-key"
          value={attrKey}
          onChange={(e) => {
            setAttrKey(e.target.value)
            setError('')
          }}
          placeholder="e.g. strength"
        />
      </div>

      <div className="space-y-2">
        <label className="text-sm font-medium" htmlFor="attr-value-type">
          Value Type
        </label>
        <Select
          value={valueType}
          onValueChange={(v) => setValueType(v as ValueType)}
        >
          <SelectTrigger id="attr-value-type">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {VALUE_TYPES.map((vt) => (
              <SelectItem key={vt} value={vt}>
                {vt}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      {valueType === 'single' && (
        <div className="space-y-2">
          <label className="text-sm font-medium" htmlFor="single-value">
            Value
          </label>
          <Input
            id="single-value"
            type="number"
            value={singleValue}
            onChange={(e) => setSingleValue(e.target.value)}
            placeholder="0"
          />
        </div>
      )}

      {valueType === 'enum' && (
        <div className="space-y-2">
          <label className="text-sm font-medium" htmlFor="enum-values">
            Values (comma-separated)
          </label>
          <Input
            id="enum-values"
            value={enumValuesText}
            onChange={(e) => setEnumValuesText(e.target.value)}
            placeholder="red, green, blue"
          />
        </div>
      )}

      {valueType === 'range' && (
        <div className="grid grid-cols-2 gap-3">
          <div className="space-y-2">
            <label className="text-sm font-medium" htmlFor="range-min">
              Min
            </label>
            <Input
              id="range-min"
              type="number"
              value={rangeMin}
              onChange={(e) => setRangeMin(e.target.value)}
              placeholder="0"
            />
          </div>
          <div className="space-y-2">
            <label className="text-sm font-medium" htmlFor="range-max">
              Max
            </label>
            <Input
              id="range-max"
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
            <label className="text-sm font-medium" htmlFor="str-min">
              Min Length
            </label>
            <Input
              id="str-min"
              type="number"
              value={stringMinLen}
              onChange={(e) => setStringMinLen(e.target.value)}
              placeholder="0"
            />
          </div>
          <div className="space-y-2">
            <label className="text-sm font-medium" htmlFor="str-max">
              Max Length
            </label>
            <Input
              id="str-max"
              type="number"
              value={stringMaxLen}
              onChange={(e) => setStringMaxLen(e.target.value)}
              placeholder="255"
            />
          </div>
        </div>
      )}

      {valueType === 'boolean' && (
        <div className="space-y-2">
          <label className="text-sm font-medium" htmlFor="bool-value">
            Value
          </label>
          <Select value={boolValue} onValueChange={setBoolValue}>
            <SelectTrigger id="bool-value">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="true">true</SelectItem>
              <SelectItem value="false">false</SelectItem>
            </SelectContent>
          </Select>
        </div>
      )}

      {(valueType === 'single' || valueType === 'range') && (
        <div className="space-y-2">
          <label className="text-sm font-medium" htmlFor="dist-type">
            Distribution
          </label>
          <Select value={distType} onValueChange={setDistType}>
            <SelectTrigger id="dist-type">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="none">None</SelectItem>
              <SelectItem value="uniform">Uniform</SelectItem>
              <SelectItem value="normal">Normal</SelectItem>
              <SelectItem value="exponential">Exponential</SelectItem>
            </SelectContent>
          </Select>
        </div>
      )}

      {(valueType === 'single' || valueType === 'range') && distType === 'normal' && (
        <div className="space-y-2">
          <label className="text-sm font-medium" htmlFor="dist-stddev">
            Std Dev
          </label>
          <Input
            id="dist-stddev"
            type="number"
            value={distStdDev}
            onChange={(e) => setDistStdDev(e.target.value)}
            placeholder="1"
          />
        </div>
      )}

      {(valueType === 'single' || valueType === 'range') && distType === 'exponential' && (
        <div className="space-y-2">
          <label className="text-sm font-medium" htmlFor="dist-rate">
            Rate
          </label>
          <Input
            id="dist-rate"
            type="number"
            value={distRate}
            onChange={(e) => setDistRate(e.target.value)}
            placeholder="1"
          />
        </div>
      )}

      <div className="space-y-2">
        <label className="text-sm font-medium" htmlFor="attr-desc">
          Description (optional)
        </label>
        <Input
          id="attr-desc"
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          placeholder="Optional description"
        />
      </div>

      {error && (
                                                    <p className={errorClass}>{error}</p>
      )}

      <div className="flex gap-2">
        <Button type="button" onClick={handleSubmit}>
          Add
        </Button>
        <Button type="button" variant="outline" onClick={onCancel}>
          Cancel
        </Button>
      </div>
    </div>
  )
}

function GlobalAttributePickerDialog({
  open,
  onOpenChange,
  onSelect,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSelect: (gma: GlobalMetaAttribute) => void
}) {
  const [search, setSearch] = useState('')
  const { data, isLoading } = useGlobalMetaAttributesList({
    perPage: 50,
    search: search || undefined,
  })

  const items = (data?.data ?? []) as GlobalMetaAttribute[]

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>Select Global Attribute</DialogTitle>
        </DialogHeader>
        <div className="space-y-3">
          <Input
            placeholder="Search attributes..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
          <div className="max-h-64 overflow-y-auto border rounded-md">
            {isLoading ? (
              <div className="p-4 space-y-2">
                {Array.from({ length: 3 }).map((_, i) => (
                  <Skeleton key={i} className="h-8 w-full" />
                ))}
              </div>
            ) : items.length === 0 ? (
              <p className="p-4 text-sm text-muted-foreground">
                No global attributes found
              </p>
            ) : (
              items.map((gma) => (
                <button
                  type="button"
                  key={gma.id}
                  className="w-full flex items-center gap-3 px-3 py-2 text-sm hover:bg-muted/50 text-left"
                  onClick={() => {
                    onSelect(gma)
                    onOpenChange(false)
                  }}
                >
                  <span className="font-medium">{gma.name}</span>
                  <Badge variant="outline">{gma.valueType}</Badge>
                </button>
              ))
            )}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  )
}

function AffixPickerDialog({
  open,
  onOpenChange,
  affixType,
  onSelect,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  affixType: 'prefix' | 'suffix'
  onSelect: (affix: Affix) => void
}) {
  const [search, setSearch] = useState('')
  const { data, isLoading } = useAffixesList({
    perPage: 50,
    search: search || undefined,
  })

  const items = (data?.data ?? []) as Affix[]
  const typedItems = items.filter((a) => a.location === affixType)

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>
            Select {affixType === 'prefix' ? 'Prefix' : 'Suffix'}
          </DialogTitle>
        </DialogHeader>
        <div className="space-y-3">
          <Input
            placeholder="Search affixes..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
          <div className="max-h-64 overflow-y-auto border rounded-md">
            {isLoading ? (
              <div className="p-4 space-y-2">
                {Array.from({ length: 3 }).map((_, i) => (
                  <Skeleton key={i} className="h-8 w-full" />
                ))}
              </div>
            ) : typedItems.length === 0 ? (
              <p className="p-4 text-sm text-muted-foreground">
                No {affixType}es found
              </p>
            ) : (
              typedItems.map((affix) => (
                <button
                  type="button"
                  key={affix.id}
                  className="w-full flex items-center gap-3 px-3 py-2 text-sm hover:bg-muted/50 text-left"
                  onClick={() => {
                    onSelect(affix)
                    onOpenChange(false)
                  }}
                >
                  <span className="font-medium">{affix.name}</span>
                  <Badge variant="outline">{affix.location}</Badge>
                </button>
              ))
            )}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  )
}

export function BlueprintFormModal({
  open,
  onOpenChange,
  blueprint,
  duplicateFrom,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  blueprint?: Blueprint | null
  duplicateFrom?: Blueprint | null
}) {
  const { toast } = useToast()
  const isEdit = !!blueprint
  const isDuplicate = !!duplicateFrom
  const createMutation = useCreateBlueprint()
  const updateMutation = useUpdateBlueprint()

  const sourceData = duplicateFrom ?? blueprint

  const [name, setName] = useState(sourceData?.name ?? '')
  const [archetype, setArchetype] = useState(sourceData?.archetype ?? '')
  const [weight, setWeight] = useState(sourceData?.weight ?? 1)
  const [description, setDescription] = useState(sourceData?.description ?? '')
  const [attributes, setAttributes] = useState<Record<string, BlueprintAttribute>>(
    sourceData?.attributes ?? {},
  )
  const [attributeOrder, setAttributeOrder] = useState<string[]>(
    sourceData?.attributeOrder ?? [],
  )
  const [minPrefixes, setMinPrefixes] = useState(
    sourceData?.minPrefixes ?? 0,
  )
  const [maxPrefixes, setMaxPrefixes] = useState(
    sourceData?.maxPrefixes ?? 0,
  )
  const [minSuffixes, setMinSuffixes] = useState(
    sourceData?.minSuffixes ?? 0,
  )
  const [maxSuffixes, setMaxSuffixes] = useState(
    sourceData?.maxSuffixes ?? 0,
  )
  const extendedSource = sourceData as (typeof sourceData) & {
    prefixes?: AffixPoolEntry[]
    suffixes?: AffixPoolEntry[]
  }
  const [prefixes, setPrefixes] = useState<AffixPoolEntry[]>(
    extendedSource?.prefixes ?? [],
  )
  const [suffixes, setSuffixes] = useState<AffixPoolEntry[]>(
    extendedSource?.suffixes ?? [],
  )

  const [errors, setErrors] = useState<Record<string, string>>({})

  const [showInlineForm, setShowInlineForm] = useState(false)
  const [showGlobalPicker, setShowGlobalPicker] = useState(false)
  const [showPrefixPicker, setShowPrefixPicker] = useState(false)
  const [showSuffixPicker, setShowSuffixPicker] = useState(false)
  const [deleteAttrKey, setDeleteAttrKey] = useState<string | null>(null)
  const [deletePoolIdx, setDeletePoolIdx] = useState<{
    type: 'prefix' | 'suffix'
    idx: number
  } | null>(null)

  const clearErrors = () => setErrors({})

  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    }),
  )

  const handleAttrDragEnd = useCallback(
    (event: DragEndEvent) => {
      const { active, over } = event
      if (!over || active.id === over.id) return

      const oldIndex = attributeOrder.indexOf(active.id as string)
      const newIndex = attributeOrder.indexOf(over.id as string)

      if (oldIndex !== -1 && newIndex !== -1) {
        const newOrder = [...attributeOrder]
        const [moved] = newOrder.splice(oldIndex, 1)
        newOrder.splice(newIndex, 0, moved)
        setAttributeOrder(newOrder)
      }
    },
    [attributeOrder],
  )

  const handlePrefixDragEnd = useCallback(
    (event: DragEndEvent) => reorderPool(prefixes, setPrefixes, 'prefix')(event),
    [prefixes],
  )

  const handleSuffixDragEnd = useCallback(
    (event: DragEndEvent) => reorderPool(suffixes, setSuffixes, 'suffix')(event),
    [suffixes],
  )

  const handleAddInline = useCallback(
    (key: string, attr: InlineAttributeDef) => {
      setAttributes((prev) => ({ ...prev, [key]: attr }))
      setAttributeOrder((prev) => {
        if (!prev.includes(key)) return [...prev, key]
        return prev
      })
      setShowInlineForm(false)
    },
    [],
  )

  const handleAddGlobal = useCallback(
    (gma: GlobalMetaAttribute) => {
      const key = gma.name
      setAttributes((prev) => ({ ...prev, [key]: { $ref_id: gma.id } }))
      setAttributeOrder((prev) => {
        if (!prev.includes(key)) return [...prev, key]
        return prev
      })
    },
    [],
  )

  const handleDeleteAttribute = useCallback(() => {
    if (!deleteAttrKey) return
    const existingKeys = attributeOrder.filter((k) => k in attributes)
    if (existingKeys.length <= 1) {
      setDeleteAttrKey(null)
      return
    }

    setAttributes((prev) => {
      const next = { ...prev }
      delete next[deleteAttrKey]
      return next
    })
    setAttributeOrder((prev) => prev.filter((k) => k !== deleteAttrKey))
    setDeleteAttrKey(null)
  }, [deleteAttrKey, attributeOrder, attributes])

  const handleAddAffix = useCallback(
    (affix: Affix) => {
      const entry: AffixPoolEntry = { affixId: affix.id, weight: 1 }

      if (affix.location === 'prefix') {
        setPrefixes((prev) => [...prev, entry])
      } else {
        setSuffixes((prev) => [...prev, entry])
      }
    },
    [],
  )

  const handleWeightChange = useCallback(
    (type: 'prefix' | 'suffix', idx: number, value: string) => {
      const num = Number(value)
      if (isNaN(num)) return

      if (type === 'prefix') {
        setPrefixes((prev) =>
          prev.map((p, i) => (i === idx ? { ...p, weight: num } : p)),
        )
      } else {
        setSuffixes((prev) =>
          prev.map((s, i) => (i === idx ? { ...s, weight: num } : s)),
        )
      }
    },
    [],
  )

  const handleDeletePoolEntry = useCallback(() => {
    if (!deletePoolIdx) return

    if (deletePoolIdx.type === 'prefix') {
      setPrefixes((prev) => prev.filter((_, i) => i !== deletePoolIdx.idx))
    } else {
      setSuffixes((prev) => prev.filter((_, i) => i !== deletePoolIdx.idx))
    }
    setDeletePoolIdx(null)
  }, [deletePoolIdx])

  const handleSubmit = async () => {
    clearErrors()

    const affixes: BlueprintAffixConfig = {
      minPrefixes,
      maxPrefixes,
      minSuffixes,
      maxSuffixes,
      prefixes,
      suffixes,
    }

    const formValues = {
      name,
      archetype,
      weight,
      description: description || null,
      attributes,
      attributeOrder,
      affixes,
    }

    const result = createBlueprintSchema.safeParse(formValues)
    if (!result.success) {
      const fieldErrors: Record<string, string> = {}
      for (const issue of result.error.issues) {
        const path = issue.path.join('.')
        if (!fieldErrors[path]) {
          fieldErrors[path] = issue.message
        }
      }
      setErrors(fieldErrors)
      toast({ title: 'Please fix validation errors', variant: 'destructive' })
      return
    }

    const request = formValues as unknown as CreateBlueprintRequest
    try {
      if (isEdit && blueprint) {
        await updateMutation.mutateAsync({
          id: blueprint.id,
          request,
        })
        toast({ title: 'Blueprint updated' })
      } else {
        await createMutation.mutateAsync(request)
        toast({ title: 'Blueprint created' })
      }
      onOpenChange(false)
    } catch {
      toast({
        title: isEdit
          ? 'Failed to update blueprint'
          : 'Failed to create blueprint',
        variant: 'destructive',
      })
    }
  }

  const resetForm = useCallback(() => {
    setName(sourceData?.name ?? '')
    setArchetype(sourceData?.archetype ?? '')
    setWeight(sourceData?.weight ?? 1)
    setDescription(sourceData?.description ?? '')
    setAttributes(sourceData?.attributes ?? {})
    setAttributeOrder(sourceData?.attributeOrder ?? [])
    setMinPrefixes(sourceData?.minPrefixes ?? 0)
    setMaxPrefixes(sourceData?.maxPrefixes ?? 0)
    setMinSuffixes(sourceData?.minSuffixes ?? 0)
    setMaxSuffixes(sourceData?.maxSuffixes ?? 0)
    const ext = sourceData as (typeof sourceData) & {
      prefixes?: AffixPoolEntry[]
      suffixes?: AffixPoolEntry[]
    }
    setPrefixes(ext?.prefixes ?? [])
    setSuffixes(ext?.suffixes ?? [])
    setErrors({})
    setShowInlineForm(false)
  }, [sourceData])

  const isPending = createMutation.isPending || updateMutation.isPending

  const modalTitle = isDuplicate
    ? 'Duplicate Blueprint'
    : isEdit
      ? 'Edit Blueprint'
      : 'Create Blueprint'

  const submitLabel = isPending
    ? isEdit
      ? 'Saving...'
      : 'Creating...'
    : isEdit
      ? 'Save Changes'
      : 'Create Blueprint'

  const attrEntries: AttributeEntry[] = attributeOrder
    .filter((key) => key in attributes)
    .map((key) => ({ key, attribute: attributes[key] }))

  const attrSortableIds = attributeOrder.filter((key) => key in attributes)

  return (
    <>
      <Dialog
        open={open}
        onOpenChange={(newOpen) => {
          if (!newOpen) resetForm()
          onOpenChange(newOpen)
        }}
      >
        <DialogContent className="max-w-2xl max-h-[85vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>{modalTitle}</DialogTitle>
            <DialogDescription>
              {isEdit
                ? 'Modify the blueprint configuration below.'
                : 'Fill in the details below to define a new blueprint.'}
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4">
            <Tabs defaultValue="general">
              <TabsList className="grid w-full grid-cols-3">
                <TabsTrigger value="general">General</TabsTrigger>
                <TabsTrigger value="attributes">Attributes</TabsTrigger>
                <TabsTrigger value="affixes">Affixes</TabsTrigger>
              </TabsList>

              <TabsContent value="general" className="space-y-4">
                <div className="space-y-2">
                  <label className="text-sm font-medium" htmlFor="bp-name">
                    Name *
                  </label>
                  <Input
                    id="bp-name"
                    value={name}
                    onChange={(e) => {
                      setName(e.target.value)
                      clearErrors()
                    }}
                    placeholder="Blueprint name"
                  />
                  {errors.name && (
                    <p className={errorClass}>
                      {errors.name}
                    </p>
                  )}
                </div>

                <div className="space-y-2">
                  <label className="text-sm font-medium" htmlFor="bp-archetype">
                    Archetype *
                  </label>
                  <Input
                    id="bp-archetype"
                    value={archetype}
                    onChange={(e) => {
                      setArchetype(e.target.value)
                      clearErrors()
                    }}
                    placeholder="e.g. weapon, character"
                  />
                  {errors.archetype && (
                    <p className={errorClass}>
                      {errors.archetype}
                    </p>
                  )}
                </div>

                <div className="space-y-2">
                  <label className="text-sm font-medium" htmlFor="bp-weight">
                    Weight *
                  </label>
                  <Input
                    id="bp-weight"
                    type="number"
                    min={1}
                    value={weight}
                    onChange={(e) => {
                      setWeight(Number(e.target.value))
                      clearErrors()
                    }}
                  />
                  {errors.weight && (
                    <p className={errorClass}>
                      {errors.weight}
                    </p>
                  )}
                </div>

                <div className="space-y-2">
                  <label className="text-sm font-medium" htmlFor="bp-desc">
                    Description (optional)
                  </label>
                  <Input
                    id="bp-desc"
                    value={description ?? ''}
                    onChange={(e) => setDescription(e.target.value)}
                    placeholder="Optional description"
                  />
                </div>
              </TabsContent>

              <TabsContent value="attributes" className="space-y-4">
                <div className="flex items-center justify-between">
                  <h4 className="text-sm font-medium">Attributes</h4>
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button type="button" variant="outline" size="sm">
                        <Plus className="h-4 w-4" />
                        Add Attribute
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                      <DropdownMenuItem
                        onClick={() => setShowInlineForm(true)}
                      >
                        Inline Attribute
                      </DropdownMenuItem>
                      <DropdownMenuItem
                        onClick={() => setShowGlobalPicker(true)}
                      >
                        From Global Library
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </div>

                {showInlineForm && (
                  <InlineAttributeForm
                    onSubmit={handleAddInline}
                    onCancel={() => setShowInlineForm(false)}
                  />
                )}

                {attrEntries.length > 0 ? (
                  <div className="border rounded-md">
                    <Table>
                      <TableHeader>
                        <TableRow>
                          <TableHead className="w-8" />
                          <TableHead>Attribute Name</TableHead>
                          <TableHead>Value Type</TableHead>
                          <TableHead>Preview</TableHead>
                          <TableHead>Source</TableHead>
                          <TableHead className="w-20 text-right">
                            Actions
                          </TableHead>
                        </TableRow>
                      </TableHeader>
                      <DndContext
                        sensors={sensors}
                        collisionDetection={closestCenter}
                        onDragEnd={handleAttrDragEnd}
                      >
                        <TableBody>
                          <SortableContext
                            items={attrSortableIds}
                            strategy={verticalListSortingStrategy}
                          >
                            {attrEntries.map((entry) => (
                              <SortableRow key={entry.key} id={entry.key}>
                                <TableCell className="font-medium">
                                  {entry.key}
                                </TableCell>
                                <TableCell>
                                  <Badge variant="outline">
                                    {getValueTypeDisplay(entry.attribute)}
                                  </Badge>
                                </TableCell>
                                <TableCell className="text-muted-foreground text-sm">
                                  {getAttributePreview(entry.attribute)}
                                </TableCell>
                                <TableCell>
                                  <Badge
                                    variant={
                                      isRefAttribute(entry.attribute)
                                        ? 'secondary'
                                        : 'default'
                                    }
                                  >
                                    {isRefAttribute(entry.attribute)
                                      ? 'Global'
                                      : 'Inline'}
                                  </Badge>
                                </TableCell>
                                <TableCell className="text-right">
                                  <Button
                                    type="button"
                                    size="icon"
                                    variant="ghost"
                                    onClick={() =>
                                      setDeleteAttrKey(entry.key)
                                    }
                                    aria-label={`Delete attribute ${entry.key}`}
                                  >
                                    <Trash2 className="h-4 w-4" />
                                  </Button>
                                </TableCell>
                              </SortableRow>
                            ))}
                          </SortableContext>
                        </TableBody>
                      </DndContext>
                    </Table>
                  </div>
                ) : !showInlineForm ? (
                  <div className="text-center py-8 border rounded-md">
                    <p className="text-muted-foreground text-sm">
                      No attributes added yet
                    </p>
                  </div>
                ) : null}
              </TabsContent>

              <TabsContent value="affixes" className="space-y-6">
                <div className="space-y-4">
                  <h4 className="text-sm font-medium">Prefix Counts</h4>
                  <div className="grid grid-cols-2 gap-4">
                    <div className="space-y-2">
                      <label className="text-sm font-medium" htmlFor="bp-min-prefix">
                        Min Prefixes
                      </label>
                      <Input
                        id="bp-min-prefix"
                        type="number"
                        min={0}
                        value={minPrefixes}
                        onChange={(e) => {
                          setMinPrefixes(Number(e.target.value))
                          clearErrors()
                        }}
                      />
                      {errors['affixes.minPrefixes'] && (
                        <p className={errorClass}>
                          {errors['affixes.minPrefixes']}
                        </p>
                      )}
                    </div>
                    <div className="space-y-2">
                      <label className="text-sm font-medium" htmlFor="bp-max-prefix">
                        Max Prefixes
                      </label>
                      <Input
                        id="bp-max-prefix"
                        type="number"
                        min={0}
                        value={maxPrefixes}
                        onChange={(e) => {
                          setMaxPrefixes(Number(e.target.value))
                          clearErrors()
                        }}
                      />
                      {errors['affixes.maxPrefixes'] && (
                        <p className={errorClass}>
                          {errors['affixes.maxPrefixes']}
                        </p>
                      )}
                    </div>
                  </div>
                </div>

                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <h4 className="text-sm font-medium">Prefix Pool</h4>
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      onClick={() => setShowPrefixPicker(true)}
                    >
                      <Plus className="h-4 w-4" />
                      Add Prefix
                    </Button>
                  </div>

                  {prefixes.length > 0 ? (
                    <div className="border rounded-md">
                      <Table>
                        <TableHeader>
                          <TableRow>
                            <TableHead className="w-8" />
                            <TableHead>Affix ID</TableHead>
                            <TableHead className="w-24">Weight</TableHead>
                            <TableHead className="w-16 text-right">
                              Actions
                            </TableHead>
                          </TableRow>
                        </TableHeader>
                        <DndContext
                          sensors={sensors}
                          collisionDetection={closestCenter}
                          onDragEnd={handlePrefixDragEnd}
                        >
                          <TableBody>
                            <SortableContext
                              items={prefixes.map((_, i) => `prefix-${i}`)}
                              strategy={verticalListSortingStrategy}
                            >
                              {prefixes.map((p, idx) => (
                                <SortableRow
                                  key={`prefix-${idx}`}
                                  id={`prefix-${idx}`}
                                >
                                  <TableCell className="font-mono text-xs">
                                    {p.affixId}
                                  </TableCell>
                                  <TableCell>
                                    <Input
                                      type="number"
                                      min={0}
                                      value={p.weight}
                                      onChange={(e) =>
                                        handleWeightChange(
                                          'prefix',
                                          idx,
                                          e.target.value,
                                        )
                                      }
                                      className="h-8 w-20"
                                    />
                                  </TableCell>
                                  <TableCell className="text-right">
                                    <Button
                                      type="button"
                                      size="icon"
                                      variant="ghost"
                                      onClick={() =>
                                        setDeletePoolIdx({
                                          type: 'prefix',
                                          idx,
                                        })
                                      }
                                      aria-label={`Remove prefix`}
                                    >
                                      <Trash2 className="h-4 w-4" />
                                    </Button>
                                  </TableCell>
                                </SortableRow>
                              ))}
                            </SortableContext>
                          </TableBody>
                        </DndContext>
                      </Table>
                    </div>
                  ) : (
                    <div className="text-center py-4 border rounded-md">
                      <p className="text-muted-foreground text-sm">
                        No prefixes in the pool
                      </p>
                    </div>
                  )}
                </div>

                <div className="space-y-4">
                  <h4 className="text-sm font-medium">Suffix Counts</h4>
                  <div className="grid grid-cols-2 gap-4">
                    <div className="space-y-2">
                      <label className="text-sm font-medium" htmlFor="bp-min-suffix">
                        Min Suffixes
                      </label>
                      <Input
                        id="bp-min-suffix"
                        type="number"
                        min={0}
                        value={minSuffixes}
                        onChange={(e) => {
                          setMinSuffixes(Number(e.target.value))
                          clearErrors()
                        }}
                      />
                      {errors['affixes.minSuffixes'] && (
                        <p className={errorClass}>
                          {errors['affixes.minSuffixes']}
                        </p>
                      )}
                    </div>
                    <div className="space-y-2">
                      <label className="text-sm font-medium" htmlFor="bp-max-suffix">
                        Max Suffixes
                      </label>
                      <Input
                        id="bp-max-suffix"
                        type="number"
                        min={0}
                        value={maxSuffixes}
                        onChange={(e) => {
                          setMaxSuffixes(Number(e.target.value))
                          clearErrors()
                        }}
                      />
                      {errors['affixes.maxSuffixes'] && (
                        <p className={errorClass}>
                          {errors['affixes.maxSuffixes']}
                        </p>
                      )}
                    </div>
                  </div>
                </div>

                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <h4 className="text-sm font-medium">Suffix Pool</h4>
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      onClick={() => setShowSuffixPicker(true)}
                    >
                      <Plus className="h-4 w-4" />
                      Add Suffix
                    </Button>
                  </div>

                  {suffixes.length > 0 ? (
                    <div className="border rounded-md">
                      <Table>
                        <TableHeader>
                          <TableRow>
                            <TableHead className="w-8" />
                            <TableHead>Affix ID</TableHead>
                            <TableHead className="w-24">Weight</TableHead>
                            <TableHead className="w-16 text-right">
                              Actions
                            </TableHead>
                          </TableRow>
                        </TableHeader>
                        <DndContext
                          sensors={sensors}
                          collisionDetection={closestCenter}
                          onDragEnd={handleSuffixDragEnd}
                        >
                          <TableBody>
                            <SortableContext
                              items={suffixes.map((_, i) => `suffix-${i}`)}
                              strategy={verticalListSortingStrategy}
                            >
                              {suffixes.map((s, idx) => (
                                <SortableRow
                                  key={`suffix-${idx}`}
                                  id={`suffix-${idx}`}
                                >
                                  <TableCell className="font-mono text-xs">
                                    {s.affixId}
                                  </TableCell>
                                  <TableCell>
                                    <Input
                                      type="number"
                                      min={0}
                                      value={s.weight}
                                      onChange={(e) =>
                                        handleWeightChange(
                                          'suffix',
                                          idx,
                                          e.target.value,
                                        )
                                      }
                                      className="h-8 w-20"
                                    />
                                  </TableCell>
                                  <TableCell className="text-right">
                                    <Button
                                      type="button"
                                      size="icon"
                                      variant="ghost"
                                      onClick={() =>
                                        setDeletePoolIdx({
                                          type: 'suffix',
                                          idx,
                                        })
                                      }
                                      aria-label={`Remove suffix`}
                                    >
                                      <Trash2 className="h-4 w-4" />
                                    </Button>
                                  </TableCell>
                                </SortableRow>
                              ))}
                            </SortableContext>
                          </TableBody>
                        </DndContext>
                      </Table>
                    </div>
                  ) : (
                    <div className="text-center py-4 border rounded-md">
                      <p className="text-muted-foreground text-sm">
                        No suffixes in the pool
                      </p>
                    </div>
                  )}
                </div>
              </TabsContent>
            </Tabs>

            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                onClick={() => onOpenChange(false)}
                disabled={isPending}
              >
                Cancel
              </Button>
              <Button onClick={handleSubmit} disabled={isPending}>
                {submitLabel}
              </Button>
            </DialogFooter>
          </div>
        </DialogContent>
      </Dialog>

      <ConfirmDialog
        open={deleteAttrKey !== null}
        onOpenChange={(open) => {
          if (!open) setDeleteAttrKey(null)
        }}
        title="Delete Attribute"
        description={`Are you sure you want to delete the attribute "${deleteAttrKey}"? ${
          attributeOrder.filter((k) => k in attributes).length <= 1
            ? 'This is the last attribute and cannot be deleted.'
            : ''
        }`}
        confirmLabel={
          attributeOrder.filter((k) => k in attributes).length <= 1
            ? 'Cannot Delete'
            : 'Delete'
        }
        variant="destructive"
        disabled={
          attributeOrder.filter((k) => k in attributes).length <= 1
        }
        onConfirm={handleDeleteAttribute}
      />

      <ConfirmDialog
        open={deletePoolIdx !== null}
        onOpenChange={(open) => {
          if (!open) setDeletePoolIdx(null)
        }}
        title="Remove from Pool"
        description={
          deletePoolIdx
            ? `Are you sure you want to remove this ${deletePoolIdx.type} from the pool?`
            : ''
        }
        confirmLabel="Remove"
        variant="destructive"
        onConfirm={handleDeletePoolEntry}
      />

      <GlobalAttributePickerDialog
        open={showGlobalPicker}
        onOpenChange={setShowGlobalPicker}
        onSelect={handleAddGlobal}
      />

      <AffixPickerDialog
        open={showPrefixPicker}
        onOpenChange={setShowPrefixPicker}
        affixType="prefix"
        onSelect={handleAddAffix}
      />

      <AffixPickerDialog
        open={showSuffixPicker}
        onOpenChange={setShowSuffixPicker}
        affixType="suffix"
        onSelect={handleAddAffix}
      />
    </>
  )
}
