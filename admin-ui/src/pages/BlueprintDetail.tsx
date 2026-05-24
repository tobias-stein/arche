import { useMemo, useState } from 'react'
import { useNavigate, useParams, Link } from 'react-router-dom'
import {
  ArrowLeft,
  ChevronDown,
  ChevronRight,
  Clock,
  Copy,
  Pencil,
} from 'lucide-react'
import {
  useBlueprint,
  useCreateBlueprint,
  useAffixesList,
  useAuditLog,
} from '@/api/generated/hooks'
import type {
  Affix,
  AffixPoolEntry,
  BlueprintAttribute,
  AuditLogEntry,
  InlineAttributeDef,
  RefAttribute,
} from '@/api/generated/types'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { BlueprintFormModal } from '@/components/BlueprintFormModal'
import { useToast } from '@/hooks/use-toast'

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

function formatJson(obj: unknown): string {
  return JSON.stringify(obj, null, 2)
}

function isRefAttribute(attr: BlueprintAttribute): attr is RefAttribute {
  return '$ref_id' in attr
}

function getValueTypeDisplay(attr: BlueprintAttribute): string {
  if ('$ref_id' in attr) return 'global'
  return attr.valueType
}

function getAttributePreview(attr: BlueprintAttribute): string {
  if ('$ref_id' in attr) return '\u2014'
  const inline = attr as InlineAttributeDef
  switch (inline.valueType) {
    case 'single':
      return String(inline.value)
    case 'enum':
      return inline.values.join(', ')
    case 'range':
      return `${inline.min} \u2013 ${inline.max}`
    case 'string': {
      const parts: string[] = []
      if (inline.minLength !== undefined) parts.push(`min: ${inline.minLength}`)
      if (inline.maxLength !== undefined) parts.push(`max: ${inline.maxLength}`)
      return parts.length > 0 ? parts.join(', ') : '\u2014'
    }
    case 'boolean':
      return String(inline.value)
    default:
      return '\u2014'
  }
}

function AttributeRow({ name, attribute }: { name: string; attribute: BlueprintAttribute }) {
  const isGlobal = isRefAttribute(attribute)
  return (
    <TableRow>
      <TableCell className="font-medium">{name}</TableCell>
      <TableCell>
        <Badge variant="outline">{getValueTypeDisplay(attribute)}</Badge>
      </TableCell>
      <TableCell className="text-muted-foreground">
        {isGlobal ? (
          <span className="text-muted-foreground/60">{getAttributePreview(attribute)}</span>
        ) : (
          getAttributePreview(attribute)
        )}
      </TableCell>
      <TableCell>
        {isGlobal ? (
          <Link
            to={`/global-meta-attributes/${(attribute as RefAttribute).$ref_id}`}
            className="inline-flex items-center"
          >
            <Badge variant="secondary" className="hover:bg-secondary/80 cursor-pointer">
              global
            </Badge>
          </Link>
        ) : (
          <Badge>inline</Badge>
        )}
      </TableCell>
    </TableRow>
  )
}

function AuditLogDiff({ entry }: { entry: AuditLogEntry }) {
  const [expanded, setExpanded] = useState(false)

  return (
    <div className="border rounded-md">
      <button
        type="button"
        onClick={() => setExpanded(!expanded)}
        className="flex w-full items-center gap-2 px-3 py-2 text-sm hover:bg-muted/50 transition-colors"
      >
        {expanded ? (
          <ChevronDown className="h-4 w-4" />
        ) : (
          <ChevronRight className="h-4 w-4" />
        )}
        <span className="font-medium">{entry.action}</span>
        <span className="text-muted-foreground">
          by {entry.actorKeyName}
        </span>
        <span className="ml-auto text-xs text-muted-foreground">
          {formatDate(entry.timestamp)}
        </span>
      </button>
      {expanded && (
        <div className="border-t p-3 space-y-3">
          {entry.before && entry.after ? (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              <div>
                <p className="text-xs font-medium text-destructive mb-1">
                  Before
                </p>
                <pre className="text-xs font-mono bg-muted rounded-md p-2 overflow-auto max-h-64 whitespace-pre-wrap break-all">
                  {formatJson(entry.before)}
                </pre>
              </div>
              <div>
                <p className="text-xs font-medium text-green-600 dark:text-green-400 mb-1">
                  After
                </p>
                <pre className="text-xs font-mono bg-muted rounded-md p-2 overflow-auto max-h-64 whitespace-pre-wrap break-all">
                  {formatJson(entry.after)}
                </pre>
              </div>
            </div>
          ) : entry.after ? (
            <div>
              <p className="text-xs font-medium text-green-600 dark:text-green-400 mb-1">
                Created
              </p>
              <pre className="text-xs font-mono bg-muted rounded-md p-2 overflow-auto max-h-64 whitespace-pre-wrap break-all">
                {formatJson(entry.after)}
              </pre>
            </div>
          ) : entry.before ? (
            <div>
              <p className="text-xs font-medium text-destructive mb-1">
                Deleted
              </p>
              <pre className="text-xs font-mono bg-muted rounded-md p-2 overflow-auto max-h-64 whitespace-pre-wrap break-all">
                {formatJson(entry.before)}
              </pre>
            </div>
          ) : null}
        </div>
      )}
    </div>
  )
}

function DetailSkeleton() {
  return (
    <div className="space-y-6">
      <div className="flex items-center gap-3">
        <Skeleton className="h-8 w-8 rounded-md" />
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-6 w-20 rounded-md" />
        <Skeleton className="h-6 w-16 rounded-md" />
        <div className="flex gap-2 ml-auto">
          <Skeleton className="h-9 w-20 rounded-md" />
          <Skeleton className="h-9 w-24 rounded-md" />
        </div>
      </div>
      <Skeleton className="h-4 w-64" />
      <Skeleton className="h-10 w-80" />
      <Card>
        <CardHeader>
          <Skeleton className="h-5 w-24" />
        </CardHeader>
        <CardContent>
          <div className="space-y-2">
            {Array.from({ length: 4 }).map((_, i) => (
              <div key={i} className="flex items-center gap-4 py-1">
                <Skeleton className="h-4 w-32" />
                <Skeleton className="h-5 w-16 rounded-md" />
                <Skeleton className="h-4 w-24" />
                <Skeleton className="h-5 w-14 rounded-md" />
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

interface PoolTableProps {
  entries: AffixPoolEntry[]
  affixMap: Map<string, Affix>
  label: string
}

function PoolTable({ entries, affixMap, label }: PoolTableProps) {
  if (entries.length === 0) {
    return (
      <div className="text-center py-8 border rounded-md">
        <p className="text-muted-foreground text-sm">
          No {label.toLowerCase()}s configured
        </p>
      </div>
    )
  }

  return (
    <div className="border rounded-md">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead className="w-24">Weight</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {entries.map((entry) => {
            const affix = affixMap.get(entry.affixId)
            return (
              <TableRow key={entry.affixId}>
                <TableCell>
                  {affix ? (
                    <Link
                      to={`/affixes/${entry.affixId}`}
                      className="text-primary hover:underline"
                    >
                      {affix.name}
                    </Link>
                  ) : (
                    <code className="font-mono text-xs text-muted-foreground">
                      {entry.affixId}
                    </code>
                  )}
                </TableCell>
                <TableCell className="tabular-nums">{entry.weight}</TableCell>
              </TableRow>
            )
          })}
        </TableBody>
      </Table>
    </div>
  )
}

export default function BlueprintDetail() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const { toast } = useToast()
  const [showEditDialog, setShowEditDialog] = useState(false)

  const {
    data: blueprint,
    isLoading,
    isError,
  } = useBlueprint(id ?? '')

  const createMutation = useCreateBlueprint()

  const { data: affixData } = useAffixesList({ perPage: 200 })

  const affixMap = useMemo(() => {
    const map = new Map<string, Affix>()
    const affixes = (affixData?.data ?? []) as Affix[]
    for (const a of affixes) {
      map.set(a.id, a)
    }
    return map
  }, [affixData])

  const {
    data: auditData,
    isLoading: auditLoading,
  } = useAuditLog({ resourceType: 'blueprint' })

  const auditEntries = ((auditData?.data ?? []) as AuditLogEntry[]).filter(
    (entry) => entry.resourceId === id,
  )

  if (isLoading) {
    return <DetailSkeleton />
  }

  if (isError || !blueprint) {
    return (
      <div className="space-y-4">
        <div className="flex items-center gap-3">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => navigate('/blueprints')}
            aria-label="Back to blueprints"
          >
            <ArrowLeft className="h-5 w-5" />
          </Button>
          <h2 className="text-2xl font-bold">Blueprint not found</h2>
        </div>
        <Card>
          <CardContent className="p-8 text-center">
            <p className="text-muted-foreground">
              The blueprint with ID &quot;{id}&quot; does not exist or has been
              deleted.
            </p>
            <Button
              variant="outline"
              className="mt-4"
              onClick={() => navigate('/blueprints')}
            >
              Back to Blueprints
            </Button>
          </CardContent>
        </Card>
      </div>
    )
  }

  const extended = blueprint as typeof blueprint & {
    prefixes?: AffixPoolEntry[]
    suffixes?: AffixPoolEntry[]
  }

  const prefixes = extended.prefixes ?? []
  const suffixes = extended.suffixes ?? []

  const sortedAttrKeys = blueprint.attributeOrder.filter(
    (k) => k in (blueprint.attributes ?? {}),
  )

  const bp = blueprint

  async function handleDuplicate() {
    try {
      await createMutation.mutateAsync({
        name: `${bp.name} (Copy)`,
        archetype: bp.archetype,
        weight: bp.weight,
        description: bp.description,
        attributes: bp.attributes,
        attributeOrder: bp.attributeOrder,
        affixes: {
          minPrefixes: bp.minPrefixes,
          maxPrefixes: bp.maxPrefixes,
          minSuffixes: bp.minSuffixes,
          maxSuffixes: bp.maxSuffixes,
          prefixes: [],
          suffixes: [],
        },
      })
      toast({ title: `Duplicated "${bp.name}"` })
    } catch {
      toast({ title: 'Failed to duplicate blueprint', variant: 'destructive' })
    }
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-start gap-3 flex-wrap">
        <Button
          variant="ghost"
          size="icon"
          onClick={() => navigate('/blueprints')}
          aria-label="Back to blueprints"
        >
          <ArrowLeft className="h-5 w-5" />
        </Button>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-3 flex-wrap">
            <h2 className="text-2xl font-bold truncate">{blueprint.name}</h2>
            <Badge variant="secondary">{blueprint.archetype}</Badge>
            <Badge variant="outline" className="font-mono">
              Weight: {blueprint.weight}
            </Badge>
          </div>
          {blueprint.description && (
            <p className="text-muted-foreground mt-1">{blueprint.description}</p>
          )}
        </div>
        <div className="flex gap-2">
          <Button
            variant="outline"
            onClick={handleDuplicate}
            disabled={createMutation.isPending}
          >
            <Copy className="h-4 w-4" />
            Duplicate
          </Button>
          <Button onClick={() => setShowEditDialog(true)}>
            <Pencil className="h-4 w-4" />
            Edit
          </Button>
        </div>
      </div>

      {/* Tabs */}
      <Tabs defaultValue="general">
        <TabsList className="grid w-full max-w-md grid-cols-4">
          <TabsTrigger value="general">General</TabsTrigger>
          <TabsTrigger value="attributes">Attributes</TabsTrigger>
          <TabsTrigger value="affixes">Affixes</TabsTrigger>
          <TabsTrigger value="history">
            <Clock className="h-3.5 w-3.5" />
            History
            {auditEntries.length > 0 && (
              <span className="ml-1.5 rounded-full bg-muted-foreground/20 px-1.5 py-0.5 text-xs">
                {auditEntries.length}
              </span>
            )}
          </TabsTrigger>
        </TabsList>

        {/* Tab 1 — General */}
        <TabsContent value="general" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Details</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div>
                  <p className="text-sm text-muted-foreground">Name</p>
                  <p className="font-medium">{blueprint.name}</p>
                </div>
                <div>
                  <p className="text-sm text-muted-foreground">Archetype</p>
                  <Badge variant="secondary">{blueprint.archetype}</Badge>
                </div>
                <div>
                  <p className="text-sm text-muted-foreground">Weight</p>
                  <p className="font-medium tabular-nums">{blueprint.weight}</p>
                </div>
                <div>
                  <p className="text-sm text-muted-foreground">Description</p>
                  <p>{blueprint.description || <span className="text-muted-foreground/60">No description</span>}</p>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="text-base">Metadata</CardTitle>
            </CardHeader>
            <CardContent className="space-y-2 text-sm">
              <div className="flex items-center gap-2">
                <span className="text-muted-foreground">ID:</span>
                <code className="font-mono text-xs bg-muted px-1.5 py-0.5 rounded">
                  {blueprint.id}
                </code>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-muted-foreground">Client ID:</span>
                <code className="font-mono text-xs bg-muted px-1.5 py-0.5 rounded">
                  {blueprint.clientId}
                </code>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-muted-foreground">Created:</span>
                <span>{formatDate(blueprint.createdAt)}</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-muted-foreground">Updated:</span>
                <span>{formatDate(blueprint.updatedAt)}</span>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Tab 2 — Attributes */}
        <TabsContent value="attributes">
          <Card>
            <CardHeader>
              <CardTitle className="text-base">
                Attributes ({sortedAttrKeys.length})
              </CardTitle>
            </CardHeader>
            <CardContent>
              {sortedAttrKeys.length === 0 ? (
                <div className="text-center py-8">
                  <p className="text-muted-foreground text-sm">
                    No attributes defined
                  </p>
                </div>
              ) : (
                <div className="border rounded-md">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>Name</TableHead>
                        <TableHead>Value Type</TableHead>
                        <TableHead>Preview</TableHead>
                        <TableHead>Source</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {sortedAttrKeys.map((key) => (
                        <AttributeRow
                          key={key}
                          name={key}
                          attribute={blueprint.attributes[key]}
                        />
                      ))}
                    </TableBody>
                  </Table>
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        {/* Tab 3 — Affixes */}
        <TabsContent value="affixes" className="space-y-6">
          {/* Prefixes section */}
          <Card>
            <CardHeader>
              <CardTitle className="text-base">
                Prefixes (min: {blueprint.minPrefixes}, max: {blueprint.maxPrefixes})
              </CardTitle>
            </CardHeader>
            <CardContent>
              <PoolTable
                entries={prefixes}
                affixMap={affixMap}
                label="Prefix"
              />
            </CardContent>
          </Card>

          {/* Suffixes section */}
          <Card>
            <CardHeader>
              <CardTitle className="text-base">
                Suffixes (min: {blueprint.minSuffixes}, max: {blueprint.maxSuffixes})
              </CardTitle>
            </CardHeader>
            <CardContent>
              <PoolTable
                entries={suffixes}
                affixMap={affixMap}
                label="Suffix"
              />
            </CardContent>
          </Card>
        </TabsContent>

        {/* Tab 4 — History */}
        <TabsContent value="history">
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Audit Log</CardTitle>
            </CardHeader>
            <CardContent>
              {auditLoading ? (
                <div className="space-y-2">
                  {Array.from({ length: 3 }).map((_, i) => (
                    <Skeleton key={i} className="h-10 w-full" />
                  ))}
                </div>
              ) : auditEntries.length === 0 ? (
                <p className="text-sm text-muted-foreground text-center py-8">
                  No audit log entries for this blueprint
                </p>
              ) : (
                <div className="space-y-2">
                  {auditEntries.map((entry) => (
                    <AuditLogDiff key={entry.id} entry={entry} />
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      {/* Edit dialog */}
      <BlueprintFormModal
        key={blueprint.id}
        open={showEditDialog}
        onOpenChange={setShowEditDialog}
        blueprint={blueprint}
      />
    </div>
  )
}
