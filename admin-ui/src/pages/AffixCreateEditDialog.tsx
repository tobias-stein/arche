import { useEffect, useMemo, useState } from 'react'
import { ChevronDown, Search } from 'lucide-react'
import { useGlobalMetaAttributesList } from '@/api/generated/hooks'
import type {
  Affix,
  AffixAttribute,
  AffixInlineAttributeDef,
  AffixLocation,
  GlobalMetaAttribute,
  RefAttribute,
  ValueType,
} from '@/api/generated/types'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'

const VALUE_TYPES: ValueType[] = ['single', 'enum', 'range', 'string', 'boolean']

function AffixAttributeForm({
  attribute,
  source,
  onAttributeChange,
  onSourceChange,
}: {
  attribute: AffixAttribute | null
  source: 'inline' | 'global'
  onAttributeChange: (attr: AffixAttribute | null) => void
  onSourceChange: (source: 'inline' | 'global') => void
}) {
  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <label className="text-sm font-medium">Attribute Source</label>
        <div className="flex rounded-lg border p-1 gap-1">
          <button
            type="button"
            onClick={() => onSourceChange('inline')}
            className={`flex-1 rounded-md px-3 py-1.5 text-sm font-medium transition-colors ${
              source === 'inline'
                ? 'bg-primary text-primary-foreground shadow'
                : 'text-muted-foreground hover:text-foreground'
            }`}
          >
            Inline
          </button>
          <button
            type="button"
            onClick={() => onSourceChange('global')}
            className={`flex-1 rounded-md px-3 py-1.5 text-sm font-medium transition-colors ${
              source === 'global'
                ? 'bg-primary text-primary-foreground shadow'
                : 'text-muted-foreground hover:text-foreground'
            }`}
          >
            From Library
          </button>
        </div>
      </div>

      {source === 'inline' ? (
        <InlineAttributeForm attribute={attribute} onChange={onAttributeChange} />
      ) : (
        <GlobalAttributePicker
          refId={attribute && '$ref_id' in attribute ? attribute.$ref_id : ''}
          onChange={onAttributeChange}
        />
      )}
    </div>
  )
}

function getDistribution(
  attr: AffixAttribute | null,
): { use: boolean; type: 'uniform' | 'normal' | 'exponential'; stdDev: string; rate: string } {
  if (!attr || '$ref_id' in attr) return { use: false, type: 'uniform', stdDev: '', rate: '' }
  const a = attr as Record<string, unknown>
  const dist = a.distribution as Record<string, unknown> | undefined
  if (!dist) return { use: false, type: 'uniform', stdDev: '', rate: '' }
  let distType: 'uniform' | 'normal' | 'exponential' = 'uniform'
  if (dist.type === 'normal' || dist.type === 'exponential') distType = dist.type
  return {
    use: true,
    type: distType,
    stdDev: distType === 'normal' ? String(dist.stdDev ?? '') : '',
    rate: distType === 'exponential' ? String(dist.rate ?? '') : '',
  }
}

function InlineAttributeForm({
  attribute,
  onChange,
}: {
  attribute: AffixAttribute | null
  onChange: (attr: AffixAttribute | null) => void
}) {
  const inline = attribute && !('$ref_id' in attribute)
    ? (attribute as Record<string, unknown>)
    : null

  const [name, setName] = useState(inline ? String(inline.name ?? '') : '')
  const [desc, setDesc] = useState(inline ? String(inline.description ?? '') : '')
  const [valueType, setValueType] = useState<ValueType>(
    inline ? (inline.valueType as ValueType) : 'single',
  )
  const [singleValue, setSingleValue] = useState(
    inline && inline.valueType === 'single' ? String(inline.value ?? '') : '',
  )
  const [enumValues, setEnumValues] = useState(
    inline && inline.valueType === 'enum'
      ? ((inline.values as string[] | undefined) ?? []).join(', ')
      : '',
  )
  const [rangeMin, setRangeMin] = useState(
    inline && inline.valueType === 'range' ? String(inline.min ?? '') : '',
  )
  const [rangeMax, setRangeMax] = useState(
    inline && inline.valueType === 'range' ? String(inline.max ?? '') : '',
  )
  const [strMinLen, setStrMinLen] = useState(
    inline && inline.valueType === 'string' ? String(inline.minLength ?? '') : '',
  )
  const [strMaxLen, setStrMaxLen] = useState(
    inline && inline.valueType === 'string' ? String(inline.maxLength ?? '') : '',
  )
  const [boolVal, setBoolVal] = useState(
    inline && inline.valueType === 'boolean' ? inline.value === true : false,
  )

  const initDist = getDistribution(attribute)
  const [useDist, setUseDist] = useState(initDist.use)
  const [distType, setDistType] = useState<'uniform' | 'normal' | 'exponential'>(initDist.type)
  const [distStdDev, setDistStdDev] = useState(initDist.stdDev)
  const [distRate, setDistRate] = useState(initDist.rate)

  function buildDistribution(): Record<string, unknown> | null {
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

  const buildAttribute = (): AffixAttribute | null => {
    if (!name.trim()) return null

    const base: Record<string, unknown> = { name: name.trim(), valueType }
    if (desc.trim()) base.description = desc.trim()

    switch (valueType) {
      case 'single': {
        const val = parseFloat(singleValue)
        if (isNaN(val)) return null
        base.value = val
        if (useDist) {
          const dist = buildDistribution()
          if (!dist) return null
          base.distribution = dist
        }
        break
      }
      case 'enum': {
        const vals = enumValues.split(',').map((v) => v.trim()).filter(Boolean)
        if (vals.length === 0) return null
        base.values = vals
        break
      }
      case 'range': {
        const min = parseFloat(rangeMin)
        const max = parseFloat(rangeMax)
        if (isNaN(min) || isNaN(max) || min > max) return null
        base.min = min
        base.max = max
        if (useDist) {
          const dist = buildDistribution()
          if (!dist) return null
          base.distribution = dist
        }
        break
      }
      case 'string': {
        const minL = strMinLen ? parseInt(strMinLen) : undefined
        const maxL = strMaxLen ? parseInt(strMaxLen) : undefined
        if (minL !== undefined) base.minLength = minL
        if (maxL !== undefined) base.maxLength = maxL
        break
      }
      case 'boolean':
        base.value = boolVal
        break
    }

    return base as unknown as AffixInlineAttributeDef
  }

  useEffect(() => {
    onChange(buildAttribute())
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [name, desc, valueType, singleValue, enumValues, rangeMin, rangeMax, strMinLen, strMaxLen, boolVal, useDist, distType, distStdDev, distRate])

  return (
    <div className="space-y-3">
      <div className="space-y-2">
        <label htmlFor="attr-name" className="text-sm font-medium">
          Attribute Name
        </label>
        <Input
          id="attr-name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="e.g. damage"
        />
      </div>
      <div className="space-y-2">
        <label htmlFor="attr-desc" className="text-sm font-medium">
          Attribute Description
        </label>
        <Input
          id="attr-desc"
          value={desc}
          onChange={(e) => setDesc(e.target.value)}
          placeholder="Optional"
        />
      </div>
      <div className="space-y-2">
        <label htmlFor="attr-vtype" className="text-sm font-medium">
          Value Type
        </label>
        <select
          id="attr-vtype"
          value={valueType}
          onChange={(e) => setValueType(e.target.value as ValueType)}
          className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
        >
          {VALUE_TYPES.map((vt) => (
            <option key={vt} value={vt}>
              {vt.charAt(0).toUpperCase() + vt.slice(1)}
            </option>
          ))}
        </select>
      </div>

      {valueType === 'single' && (
        <div className="space-y-2">
          <label htmlFor="attr-value" className="text-sm font-medium">
            Value
          </label>
          <Input
            id="attr-value"
            type="number"
            value={singleValue}
            onChange={(e) => setSingleValue(e.target.value)}
            placeholder="0"
          />
        </div>
      )}

      {valueType === 'enum' && (
        <div className="space-y-2">
          <label htmlFor="attr-values" className="text-sm font-medium">
            Values (comma-separated)
          </label>
          <Input
            id="attr-values"
            value={enumValues}
            onChange={(e) => setEnumValues(e.target.value)}
            placeholder="fire, ice, lightning"
          />
        </div>
      )}

      {valueType === 'range' && (
        <div className="grid grid-cols-2 gap-3">
          <div className="space-y-2">
            <label htmlFor="attr-min" className="text-sm font-medium">
              Min
            </label>
            <Input
              id="attr-min"
              type="number"
              value={rangeMin}
              onChange={(e) => setRangeMin(e.target.value)}
              placeholder="0"
            />
          </div>
          <div className="space-y-2">
            <label htmlFor="attr-max" className="text-sm font-medium">
              Max
            </label>
            <Input
              id="attr-max"
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
            <label htmlFor="attr-minlen" className="text-sm font-medium">
              Min Length
            </label>
            <Input
              id="attr-minlen"
              type="number"
              value={strMinLen}
              onChange={(e) => setStrMinLen(e.target.value)}
              placeholder="0"
            />
          </div>
          <div className="space-y-2">
            <label htmlFor="attr-maxlen" className="text-sm font-medium">
              Max Length
            </label>
            <Input
              id="attr-maxlen"
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
                name="bool-val"
                checked={boolVal}
                onChange={() => setBoolVal(true)}
                className="h-4 w-4"
              />
              True
            </label>
            <label className="flex items-center gap-2 text-sm">
              <input
                type="radio"
                name="bool-val"
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
                <label htmlFor="dist-type" className="text-sm font-medium">
                  Distribution Type
                </label>
                <select
                  id="dist-type"
                  value={distType}
                  onChange={(e) => setDistType(e.target.value as 'uniform' | 'normal' | 'exponential')}
                  className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                >
                  <option value="uniform">Uniform</option>
                  <option value="normal">Normal</option>
                  <option value="exponential">Exponential</option>
                </select>
              </div>
              {distType === 'normal' && (
                <div className="space-y-2">
                  <label htmlFor="dist-stddev" className="text-sm font-medium">
                    Std Dev
                  </label>
                  <Input
                    id="dist-stddev"
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
                  <label htmlFor="dist-rate" className="text-sm font-medium">
                    Rate
                  </label>
                  <Input
                    id="dist-rate"
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
  )
}

function GlobalAttributePicker({
  refId,
  onChange,
}: {
  refId: string
  onChange: (attr: AffixAttribute | null) => void
}) {
  const { data, isLoading } = useGlobalMetaAttributesList({ perPage: 200 })
  const [search, setSearch] = useState('')
  const [open, setOpen] = useState(false)

  const items = useMemo(
    () => (data?.data ?? []) as GlobalMetaAttribute[],
    [data?.data],
  )

  const filtered = useMemo(
    () =>
      search.trim()
        ? items.filter((gma) =>
            gma.name.toLowerCase().includes(search.toLowerCase()),
          )
        : items,
    [items, search],
  )

  const selected = items.find((gma) => gma.id === refId)

  useEffect(() => {
    if (refId) {
      onChange({ $ref_id: refId } as RefAttribute)
    } else {
      onChange(null)
    }
  }, [refId, onChange])

  return (
    <div className="space-y-2">
      <label className="text-sm font-medium">Global Meta Attribute</label>
      <div className="relative">
        <button
          type="button"
          onClick={() => setOpen(!open)}
          className="flex h-9 w-full items-center justify-between rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors hover:bg-accent hover:text-accent-foreground"
        >
          <span className={selected ? 'truncate' : 'text-muted-foreground'}>
            {selected ? selected.name : 'Select a global attribute...'}
          </span>
          <ChevronDown className="h-4 w-4 opacity-50" />
        </button>
        {open && (
          <div className="absolute top-full left-0 right-0 z-50 mt-1 rounded-md border bg-popover shadow-md">
            <div className="flex items-center gap-2 px-3 py-2 border-b">
              <Search className="h-4 w-4 text-muted-foreground" />
              <input
                type="text"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                placeholder="Search..."
                className="flex-1 bg-transparent text-sm outline-none placeholder:text-muted-foreground"
                autoFocus
              />
            </div>
            <div className="max-h-48 overflow-y-auto">
              {isLoading ? (
                <div className="p-3 text-sm text-muted-foreground">Loading...</div>
              ) : filtered.length === 0 ? (
                <div className="p-3 text-sm text-muted-foreground">
                  {search ? 'No matches' : 'No global attributes found'}
                </div>
              ) : (
                filtered.map((gma) => (
                  <button
                    key={gma.id}
                    type="button"
                    onClick={() => {
                      onChange({ $ref_id: gma.id } as RefAttribute)
                      setOpen(false)
                    }}
                    className={`flex w-full items-center gap-2 px-3 py-2 text-sm hover:bg-accent hover:text-accent-foreground ${
                      gma.id === refId ? 'bg-accent' : ''
                    }`}
                  >
                    <span className="truncate">{gma.name}</span>
                    <Badge variant="outline" className="ml-auto shrink-0 text-xs">
                      {gma.valueType}
                    </Badge>
                  </button>
                ))
              )}
            </div>
          </div>
        )}
      </div>
      {selected && (
        <button
          type="button"
          onClick={() => {
            onChange(null)
          }}
          className="text-xs text-muted-foreground underline hover:text-foreground"
        >
          Remove selection
        </button>
      )}
    </div>
  )
}

interface AffixCreateEditDialogProps {
  affix: Affix | null
  open: boolean
  onOpenChange: (open: boolean) => void
  onSubmit: (data: {
    name: string
    location: AffixLocation
    description: string | null
    attribute: AffixAttribute
  }) => Promise<void>
  isPending?: boolean
}

export function AffixCreateEditDialog({
  affix,
  open,
  onOpenChange,
  onSubmit,
  isPending = false,
}: AffixCreateEditDialogProps) {
  const isEdit = affix !== null

  const [name, setName] = useState(affix?.name ?? '')
  const [location, setLocation] = useState<AffixLocation>(affix?.location ?? 'prefix')
  const [description, setDescription] = useState(affix?.description ?? '')
  const [attribute, setAttribute] = useState<AffixAttribute | null>(
    affix?.attribute ?? null,
  )
  const [attrSource, setAttrSource] = useState<'inline' | 'global'>(
    affix?.attribute
      ? '$ref_id' in affix.attribute
        ? 'global'
        : 'inline'
      : 'inline',
  )

  const attributeValid =
    (attrSource === 'inline' &&
      attribute !== null &&
      !('$ref_id' in attribute)) ||
    (attrSource === 'global' &&
      attribute !== null &&
      '$ref_id' in attribute)

  const canSubmit = name.trim().length > 0 && attributeValid

  const handleSubmit = async () => {
    if (!canSubmit || !attribute) return
    await onSubmit({
      name: name.trim(),
      location,
      description: description.trim() || null,
      attribute,
    })
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{isEdit ? 'Edit Affix' : 'Create Affix'}</DialogTitle>
        </DialogHeader>
        <div className="space-y-4">
          <div className="space-y-2">
            <label htmlFor="affix-name" className="text-sm font-medium">
              Name
            </label>
            <Input
              id="affix-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Affix name"
            />
          </div>
          <div className="space-y-2">
            <label htmlFor="affix-type" className="text-sm font-medium">
              Type
            </label>
            <select
              id="affix-type"
              value={location}
              onChange={(e) => setLocation(e.target.value as AffixLocation)}
              className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
            >
              <option value="prefix">Prefix</option>
              <option value="suffix">Suffix</option>
            </select>
          </div>
          <div className="space-y-2">
            <label htmlFor="affix-desc" className="text-sm font-medium">
              Description
            </label>
            <Input
              id="affix-desc"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Optional description"
            />
          </div>
          <AffixAttributeForm
            attribute={attribute}
            source={attrSource}
            onAttributeChange={setAttribute}
            onSourceChange={(src) => {
              setAttrSource(src)
              setAttribute(null)
            }}
          />
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={isPending}>
            Cancel
          </Button>
          <Button onClick={handleSubmit} disabled={!canSubmit || isPending}>
            {isPending ? 'Saving...' : isEdit ? 'Save' : 'Create'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
