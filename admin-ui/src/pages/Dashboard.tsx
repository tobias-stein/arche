import { useCallback, useMemo, useRef, useState } from 'react'
import {
  AlertTriangle,
  Box,
  ChevronRight,
  Dices,
  Loader2,
  Plus,
  Puzzle,
  RotateCcw,
  Trash2,
  X,
} from 'lucide-react'

import {
  useAffixesList,
  useBlueprintsList,
  useGenerate,
  useGlobalMetaAttributesList,
} from '@/api/generated'
import type {
  Affix,
  AffixConstraints,
  ConstraintValue,
  GenerateRequest,
  GenerateResponse,
} from '@/api/generated'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Skeleton } from '@/components/ui/skeleton'
import { useToast } from '@/hooks/use-toast'
import { computeWarnings, type Warning } from '@/lib/warnings'
import { Link } from 'react-router-dom'

interface ConstraintRow {
  id: number
  key: string
  operator: string
  value: string
}

const SEVERITY_STYLES: Record<string, { border: string; badge: 'destructive' | 'default' | 'outline' }> = {
  error: { border: 'border-l-destructive', badge: 'destructive' },
  warning: { border: 'border-l-amber-500', badge: 'default' },
  info: { border: 'border-l-blue-500', badge: 'outline' },
}

const OPERATORS: Record<string, string[]> = {
  single: ['eq', 'gte', 'lte'],
  range: ['eq', 'gte', 'lte'],
  string: ['eq', 'contains'],
  boolean: ['eq'],
  enum: ['eq', 'in'],
  ref: ['eq', 'gte', 'lte', 'contains', 'in'],
  unknown: ['eq', 'gte', 'lte', 'contains', 'in'],
}

function getValueTypeFromAttribute(attr: unknown): string {
  if (!attr || typeof attr !== 'object') return 'unknown'
  const a = attr as Record<string, unknown>
  if ('valueType' in a && typeof a.valueType === 'string') return a.valueType
  if ('$ref_id' in a) return 'ref'
  return 'unknown'
}

function buildConstraints(rows: ConstraintRow[]): Record<string, ConstraintValue> | null {
  const valid = rows.filter((r) => r.key && r.operator)
  if (valid.length === 0) return null
  const result: Record<string, ConstraintValue> = {}
  for (const row of valid) {
    const op = row.operator
    if (op === 'gte' || op === 'lte') {
      result[row.key] = { [op]: Number(row.value) || 0 }
    } else if (op === 'in') {
      result[row.key] = { in: row.value.split(',').map((s) => s.trim()).filter(Boolean) }
    } else if (op === 'contains') {
      result[row.key] = { contains: row.value }
    } else {
      const num = Number(row.value)
      if (row.value === 'true') result[row.key] = true
      else if (row.value === 'false') result[row.key] = false
      else if (!isNaN(num) && row.value.trim() !== '') result[row.key] = num
      else result[row.key] = row.value
    }
  }
  return result
}

function AffixCheckboxList({
  affixes,
  selectedIds,
  onToggle,
  label,
}: {
  affixes: Affix[]
  selectedIds: string[]
  onToggle: (id: string) => void
  label: string
}) {
  const [search, setSearch] = useState('')

  const filtered = useMemo(() => {
    if (!search.trim()) return affixes
    const q = search.toLowerCase()
    return affixes.filter((a) => a.name.toLowerCase().includes(q))
  }, [affixes, search])

  return (
    <div className="space-y-2">
      <Label className="text-xs text-muted-foreground">{label}</Label>
      {affixes.length > 0 && (
        <Input
          placeholder="Filter affixes..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="h-8 text-sm"
        />
      )}
      <div className="max-h-32 overflow-y-auto space-y-1 border bg-background rounded-md p-2">
        {filtered.length === 0 && (
          <p className="text-xs text-muted-foreground text-center py-2">No affixes found</p>
        )}
        {filtered.map((a) => (
          <label
            key={a.id}
            className="flex items-center gap-2 text-sm cursor-pointer hover:bg-accent rounded px-1 py-0.5"
          >
            <input
              type="checkbox"
              checked={selectedIds.includes(a.id)}
              onChange={() => onToggle(a.id)}
              className="h-3.5 w-3.5"
            />
            <span className="truncate">{a.name}</span>
          </label>
        ))}
      </div>
    </div>
  )
}

export default function Dashboard() {
  const { toast } = useToast()

  const { data: blueprintsData, isLoading: bpLoading, isError: bpError } = useBlueprintsList({
    perPage: 500,
  })
  const { data: affixesData, isLoading: affLoading } = useAffixesList({
    perPage: 500,
  })
  const { data: gmaData } = useGlobalMetaAttributesList({
    perPage: 500,
  })
  const generateMutation = useGenerate()
  const [dismissedWarnings, setDismissedWarnings] = useState<Set<string>>(
    new Set(),
  )

  const blueprints = useMemo(() => blueprintsData?.data ?? [], [blueprintsData])
  const affixes = useMemo(() => affixesData?.data ?? [], [affixesData])
  const globalMetaAttrs = useMemo(() => gmaData?.data ?? [], [gmaData])
  const warnings = useMemo(
    () => computeWarnings(blueprints, affixes, globalMetaAttrs),
    [blueprints, affixes, globalMetaAttrs],
  )

  const visibleWarnings = warnings.filter((w) => !dismissedWarnings.has(w.id))
  const warningCount = warnings.length
  const blueprintCount = blueprintsData?.total ?? 0
  const affixCount = affixesData?.total ?? 0

  const archetypes = useMemo(
    () => [...new Set(blueprints.map((b) => b.archetype).filter(Boolean))].sort(),
    [blueprints],
  )

  const allAttributeKeys = useMemo(() => {
    const keys = new Set<string>()
    for (const bp of blueprints) {
      for (const key of bp.attributeOrder) {
        keys.add(key)
      }
    }
    return [...keys].sort()
  }, [blueprints])

  function lookupAttrType(attrKey: string): string {
    for (const bp of blueprints) {
      const attr = bp.attributes[attrKey]
      if (attr) return getValueTypeFromAttribute(attr)
    }
    return 'unknown'
  }

  const [archetype, setArchetype] = useState<string>('')
  const [constraints, setConstraints] = useState<ConstraintRow[]>([])
  const constraintIdRef = useRef(1)
  const [minPrefixes, setMinPrefixes] = useState(0)
  const [maxPrefixes, setMaxPrefixes] = useState(0)
  const [minSuffixes, setMinSuffixes] = useState(0)
  const [maxSuffixes, setMaxSuffixes] = useState(0)
  const [requireIds, setRequireIds] = useState<string[]>([])
  const [blockIds, setBlockIds] = useState<string[]>([])
  const [seed, setSeed] = useState<string>('')
  const [result, setResult] = useState<GenerateResponse | null>(null)
  const [genError, setGenError] = useState<string | null>(null)

  const addConstraint = useCallback(() => {
    const id = constraintIdRef.current++
    setConstraints((prev) => [...prev, { id, key: '', operator: 'eq', value: '' }])
  }, [])

  const updateConstraint = useCallback(
    (id: number, field: keyof ConstraintRow, value: string) => {
      setConstraints((prev) =>
        prev.map((c) => (c.id === id ? { ...c, [field]: value } : c)),
      )
    },
    [],
  )

  const removeConstraint = useCallback((id: number) => {
    setConstraints((prev) => prev.filter((c) => c.id !== id))
  }, [])

  function toggleAffix(id: string, list: 'require' | 'block') {
    if (list === 'require') {
      setRequireIds((prev) =>
        prev.includes(id) ? prev.filter((x) => x !== id) : [...prev, id],
      )
    } else {
      setBlockIds((prev) =>
        prev.includes(id) ? prev.filter((x) => x !== id) : [...prev, id],
      )
    }
  }

  function handleClear() {
    setArchetype('')
    setConstraints([])
    constraintIdRef.current = 1
    setMinPrefixes(0)
    setMaxPrefixes(0)
    setMinSuffixes(0)
    setMaxSuffixes(0)
    setRequireIds([])
    setBlockIds([])
    setSeed('')
    setResult(null)
    setGenError(null)
  }

  async function handleGenerate() {
    setResult(null)
    setGenError(null)

    const affixConstraints: AffixConstraints = {
      minPrefixes,
      maxPrefixes,
      minSuffixes,
      maxSuffixes,
      require: requireIds.length > 0 ? requireIds : undefined,
      block: blockIds.length > 0 ? blockIds : undefined,
    }

    const request: GenerateRequest = {
      archetype: !archetype || archetype === 'any' ? null : archetype,
      seed: seed.trim() ? Number(seed.trim()) : null,
      constraints: buildConstraints(constraints),
      affixes: affixConstraints,
    }

    try {
      const res = await generateMutation.mutateAsync(request)
      setResult(res)
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Generation failed'
      setGenError(msg)
      toast({
        title: 'Generation failed',
        description: msg,
        variant: 'destructive',
      })
    }
  }

  const isLoading = bpLoading || affLoading

  if (bpError) {
    return (
      <Card>
        <CardContent className="p-4">
          <p className="text-destructive text-sm">
            Failed to load blueprints. Check your API connection.
          </p>
        </CardContent>
      </Card>
    )
  }

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold">Dashboard</h2>

      {/* Stat Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Blueprints</CardTitle>
            <Box className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            {bpLoading ? (
              <Skeleton className="h-8 w-16" />
            ) : (
              <div className="text-2xl font-bold">{blueprintCount}</div>
            )}
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Affixes</CardTitle>
            <Puzzle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            {affLoading ? (
              <Skeleton className="h-8 w-16" />
            ) : (
              <div className="text-2xl font-bold">{affixCount}</div>
            )}
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Warnings</CardTitle>
            <AlertTriangle
              className={`h-4 w-4 ${
                warningCount > 0
                  ? 'text-amber-500'
                  : 'text-muted-foreground'
              }`}
            />
          </CardHeader>
          <CardContent>
            {isLoading ? (
              <Skeleton className="h-8 w-16" />
            ) : (
              <div className="text-2xl font-bold">{warningCount}</div>
            )}
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Dices className="h-5 w-5" />
            Quick Generate
          </CardTitle>
          <CardDescription>
            Configure constraints and generate a random item from matching blueprints.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          {isLoading && (
            <div className="space-y-4">
              <Skeleton className="h-10 w-full" />
              <Skeleton className="h-10 w-full" />
              <Skeleton className="h-24 w-full" />
              <Skeleton className="h-10 w-40" />
            </div>
          )}

          {!isLoading && (
            <>
              {/* Archetype */}
              <div className="space-y-2">
                <Label htmlFor="archetype">Archetype</Label>
                <Select value={archetype} onValueChange={setArchetype}>
                  <SelectTrigger id="archetype">
                    <SelectValue placeholder="Any archetype" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="any">Any</SelectItem>
                    {archetypes.map((a) => (
                      <SelectItem key={a} value={a}>
                        {a}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              {/* Constraints */}
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <Label>Attribute Constraints</Label>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={addConstraint}
                    disabled={allAttributeKeys.length === 0}
                  >
                    <Plus className="h-3.5 w-3.5 mr-1" />
                    Add constraint
                  </Button>
                </div>

                {allAttributeKeys.length === 0 && (
                  <p className="text-xs text-muted-foreground">
                    No attributes available. Create blueprints with attributes first.
                  </p>
                )}

                {constraints.map((row) => {
                  const attrType = lookupAttrType(row.key)
                  const ops = OPERATORS[attrType] ?? OPERATORS.unknown

                  return (
                    <div
                      key={row.id}
                      className="flex flex-col sm:flex-row items-start gap-2"
                    >
                      <Select
                        value={row.key}
                        onValueChange={(v) => updateConstraint(row.id, 'key', v)}
                      >
                        <SelectTrigger className="w-full sm:w-40">
                          <SelectValue placeholder="Attribute" />
                        </SelectTrigger>
                        <SelectContent>
                          {allAttributeKeys.map((k) => (
                            <SelectItem key={k} value={k}>
                              {k}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>

                      <Select
                        value={row.operator}
                        onValueChange={(v) =>
                          updateConstraint(row.id, 'operator', v)
                        }
                      >
                        <SelectTrigger className="w-full sm:w-28">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          {ops.map((op) => (
                            <SelectItem key={op} value={op}>
                              {op}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>

                      {row.operator === 'in' ? (
                        <Input
                          placeholder="val1, val2, val3"
                          value={row.value}
                          onChange={(e) =>
                            updateConstraint(row.id, 'value', e.target.value)
                          }
                          className="flex-1"
                        />
                      ) : (
                        <Input
                          placeholder={
                            attrType === 'boolean'
                              ? 'true/false'
                              : attrType === 'single' || attrType === 'range'
                                ? 'Number'
                                : 'Value'
                          }
                          value={row.value}
                          onChange={(e) =>
                            updateConstraint(row.id, 'value', e.target.value)
                          }
                          className="flex-1"
                        />
                      )}

                      <Button
                        variant="ghost"
                        size="icon"
                        aria-label="Remove constraint"
                        onClick={() => removeConstraint(row.id)}
                        className="shrink-0"
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </div>
                  )
                })}
              </div>

              {/* Affix Controls */}
              <div className="space-y-4">
                <Label>Affix Controls</Label>

                <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
                  <div className="space-y-1.5">
                    <Label htmlFor="min-prefixes" className="text-xs">
                      Min Prefixes
                    </Label>
                    <Input
                      id="min-prefixes"
                      type="number"
                      min={0}
                      value={minPrefixes}
                      onChange={(e) =>
                        setMinPrefixes(Math.max(0, Number(e.target.value) || 0))
                      }
                      className="h-8"
                    />
                  </div>
                  <div className="space-y-1.5">
                    <Label htmlFor="max-prefixes" className="text-xs">
                      Max Prefixes
                    </Label>
                    <Input
                      id="max-prefixes"
                      type="number"
                      min={0}
                      value={maxPrefixes}
                      onChange={(e) =>
                        setMaxPrefixes(Math.max(0, Number(e.target.value) || 0))
                      }
                      className="h-8"
                    />
                  </div>
                  <div className="space-y-1.5">
                    <Label htmlFor="min-suffixes" className="text-xs">
                      Min Suffixes
                    </Label>
                    <Input
                      id="min-suffixes"
                      type="number"
                      min={0}
                      value={minSuffixes}
                      onChange={(e) =>
                        setMinSuffixes(Math.max(0, Number(e.target.value) || 0))
                      }
                      className="h-8"
                    />
                  </div>
                  <div className="space-y-1.5">
                    <Label htmlFor="max-suffixes" className="text-xs">
                      Max Suffixes
                    </Label>
                    <Input
                      id="max-suffixes"
                      type="number"
                      min={0}
                      value={maxSuffixes}
                      onChange={(e) =>
                        setMaxSuffixes(Math.max(0, Number(e.target.value) || 0))
                      }
                      className="h-8"
                    />
                  </div>
                </div>

                <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                  <AffixCheckboxList
                    affixes={affixes}
                    selectedIds={requireIds}
                    onToggle={(id) => toggleAffix(id, 'require')}
                    label="Require affixes"
                  />
                  <AffixCheckboxList
                    affixes={affixes}
                    selectedIds={blockIds}
                    onToggle={(id) => toggleAffix(id, 'block')}
                    label="Block affixes"
                  />
                </div>
              </div>

              {/* Seed */}
              <div className="space-y-2">
                <Label htmlFor="seed">Seed (optional)</Label>
                <Input
                  id="seed"
                  type="text"
                  placeholder="Leave empty for random"
                  value={seed}
                  onChange={(e) => setSeed(e.target.value)}
                />
              </div>

              {/* Buttons */}
              <div className="flex flex-wrap items-center gap-2">
                <Button
                  onClick={handleGenerate}
                  disabled={generateMutation.isPending || blueprints.length === 0}
                >
                  {generateMutation.isPending ? (
                    <>
                      <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                      Generating...
                    </>
                  ) : (
                    <>
                      <Dices className="h-4 w-4 mr-2" />
                      Generate
                    </>
                  )}
                </Button>
                <Button variant="outline" onClick={handleClear}>
                  <RotateCcw className="h-4 w-4 mr-2" />
                  Clear
                </Button>
              </div>

              {blueprints.length === 0 && (
                <p className="text-xs text-muted-foreground">
                  No blueprints found for the current client. Create blueprints to start generating.
                </p>
              )}
            </>
          )}
        </CardContent>
      </Card>

      {/* Warnings Section */}
      <div className="space-y-4">
        <h3 className="text-lg font-semibold flex items-center gap-2">
          <AlertTriangle
            className={`h-5 w-5 ${
              warningCount > 0 ? 'text-amber-500' : 'text-muted-foreground'
            }`}
          />
          Configuration Warnings
          {!isLoading && (
            <Badge variant="outline" className="ml-1 font-mono">
              {visibleWarnings.length}
            </Badge>
          )}
        </h3>

        {isLoading && (
          <div className="space-y-3">
            <Skeleton className="h-20 w-full" />
            <Skeleton className="h-20 w-full" />
          </div>
        )}

        {!isLoading && visibleWarnings.length === 0 && (
          <Card>
            <CardContent className="p-6 text-center">
              <p className="text-muted-foreground">
                No warnings found. Your configuration looks healthy.
              </p>
            </CardContent>
          </Card>
        )}

        {!isLoading &&
          visibleWarnings.map((warning: Warning) => {
            const style = SEVERITY_STYLES[warning.severity] ?? SEVERITY_STYLES.info
            return (
            <Card
              key={warning.id}
              className={`border-l-4 ${style.border}`}
            >
              <CardHeader className="pb-2">
                <div className="flex items-start justify-between gap-2">
                  <div className="space-y-1">
                    <CardTitle className="text-base flex items-center gap-2">
                      <Badge
                        variant={style.badge}
                        className="text-xs shrink-0"
                      >
                        {warning.severity}
                      </Badge>
                      {warning.title}
                    </CardTitle>
                    <CardDescription>{warning.description}</CardDescription>
                  </div>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-8 w-8 shrink-0"
                    aria-label="Dismiss warning"
                    onClick={() =>
                      setDismissedWarnings((prev) => {
                        const next = new Set(prev)
                        next.add(warning.id)
                        return next
                      })
                    }
                  >
                    <X className="h-4 w-4" />
                  </Button>
                </div>
              </CardHeader>
              <CardContent className="pt-0">
                <Link
                  to={`/${warning.resourceType === 'blueprint' ? 'blueprints' : warning.resourceType === 'affix' ? 'affixes' : 'global-meta-attributes'}/${warning.resourceId}`}
                  className="inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground transition-colors"
                >
                  View {warning.resourceName}
                  <ChevronRight className="h-3.5 w-3.5" />
                </Link>
              </CardContent>
            </Card>
            )
          })
        }
      </div>

      {/* Result Display */}
      {result && (
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Generated Result</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <div>
                <Label className="text-xs text-muted-foreground">Name</Label>
                <p className="text-xl font-semibold">{result.name}</p>
              </div>
              <div>
                <Label className="text-xs text-muted-foreground">Seed</Label>
                <p className="text-sm font-mono">{result.seed}</p>
              </div>
              <div>
                <Label className="text-xs text-muted-foreground">Blueprint ID</Label>
                <p className="text-sm font-mono">{result.blueprintId}</p>
              </div>
            </div>

            {Object.keys(result.blueprintAttributes).length > 0 && (
              <div>
                <Label className="text-xs text-muted-foreground mb-2 block">
                  Blueprint Attributes
                </Label>
                <div className="flex flex-wrap gap-2">
                  {Object.entries(result.blueprintAttributes).map(([key, value]) => (
                    <Badge key={key} variant="secondary">
                      {key}: {JSON.stringify(value)}
                    </Badge>
                  ))}
                </div>
              </div>
            )}

            {result.affixAttributes.length > 0 && (
              <div>
                <Label className="text-xs text-muted-foreground mb-2 block">
                  Affix Attributes
                </Label>
                <div className="space-y-2">
                  {result.affixAttributes.map((entry) => (
                    <Card key={entry.affixId} className="p-3">
                      <div className="flex items-center gap-2 mb-1">
                        <Badge variant="outline">{entry.affixName}</Badge>
                      </div>
                      <div className="flex flex-wrap gap-1">
                        {Object.entries(entry)
                          .filter(([k]) => k !== 'affixId' && k !== 'affixName')
                          .map(([key, value]) => (
                            <Badge key={key} variant="secondary" className="text-xs">
                              {key}: {JSON.stringify(value)}
                            </Badge>
                          ))}
                      </div>
                    </Card>
                  ))}
                </div>
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {/* Error Display */}
      {genError && !result && (
        <Card>
          <CardContent className="p-4">
            <p className="text-destructive text-sm">{genError}</p>
          </CardContent>
        </Card>
      )}
    </div>
  )
}
