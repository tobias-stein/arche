import { useMemo, useState } from 'react'
import { useNavigate, useParams, Link } from 'react-router-dom'
import {
  ArrowLeft,
  ChevronDown,
  ChevronRight,
  Clock,
  Link2,
  Pencil,
} from 'lucide-react'
import {
  useAffix,
  useAuditLog,
  useGlobalMetaAttributesList,
  useUpdateAffix,
} from '@/api/generated/hooks'
import type {
  AffixAttribute,
  AuditLogEntry,
  GlobalMetaAttribute,
} from '@/api/generated/types'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { useToast } from '@/hooks/use-toast'
import { AffixCreateEditDialog } from './AffixCreateEditDialog'

function formatDateTime(dateStr: string): string {
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

function AttributePreview({ attribute, gmaMap }: { attribute: AffixAttribute; gmaMap: Map<string, { name: string; value_type: string }> }) {
  if ('$ref_id' in attribute) {
    const gma = gmaMap.get(attribute.$ref_id)
    return (
      <div className="flex items-center gap-2">
        <Badge variant="secondary">Global</Badge>
        <Badge variant="outline">{gma?.value_type ?? 'unknown'}</Badge>
        <span className="inline-flex items-center gap-1">
          <Link
            to={`/global-meta-attributes/${attribute.$ref_id}`}
            className="text-sm text-primary underline hover:no-underline"
          >
            {gma?.name ?? attribute.$ref_id}
          </Link>
          <Link2 className="h-3.5 w-3.5 text-muted-foreground shrink-0" />
        </span>
      </div>
    )
  }

  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2 flex-wrap">
        <Badge>Inline</Badge>
        <Badge variant="outline">{attribute.value_type}</Badge>
        {'name' in attribute && (
          <span className="text-sm font-medium">{attribute.name}</span>
        )}
      </div>
      {'description' in attribute && attribute.description && (
        <p className="text-sm text-muted-foreground">{attribute.description}</p>
      )}
      <div className="rounded-md border bg-muted/30 p-3">
        <pre className="text-xs font-mono whitespace-pre-wrap break-all">
          {formatJson(attribute)}
        </pre>
      </div>
    </div>
  )
}

function AuditLogDiff({ entry }: { entry: AuditLogEntry }) {
  const [expanded, setExpanded] = useState(false)

  return (
    <div className="border bg-background rounded-md">
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
          by {entry.actor_key_name}
        </span>
        <span className="ml-auto text-xs text-muted-foreground">
          {formatDateTime(entry.timestamp)}
        </span>
      </button>
      {expanded && (
        <div className="border-t bg-background p-3 space-y-3">
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
        <Skeleton className="h-6 w-16 rounded-md" />
        <Skeleton className="h-9 w-20 rounded-md ml-auto" />
      </div>
      <Skeleton className="h-4 w-64" />
      <Card>
        <CardHeader>
          <Skeleton className="h-5 w-24" />
        </CardHeader>
        <CardContent>
          <Skeleton className="h-20 w-full" />
        </CardContent>
      </Card>
    </div>
  )
}

export default function AffixDetail() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const { toast } = useToast()
  const [showEditDialog, setShowEditDialog] = useState(false)

  const {
    data: affix,
    isLoading,
    isError,
  } = useAffix(id ?? '')

  const updateMutation = useUpdateAffix()

  const { data: gmaData } = useGlobalMetaAttributesList({ per_page: 500 })
  const gmaMap = useMemo(() => {
    const map = new Map<string, { name: string; value_type: string }>()
    for (const gma of (gmaData?.data ?? []) as GlobalMetaAttribute[]) {
      map.set(gma.id, { name: gma.name, value_type: gma.value_type })
    }
    return map
  }, [gmaData?.data])

  const {
    data: auditData,
    isLoading: auditLoading,
  } = useAuditLog({ resource_type: 'affix' })

  const auditEntries = ((auditData?.data ?? []) as AuditLogEntry[]).filter(
    (entry) => entry.resource_id === id,
  )

  if (isLoading) {
    return <DetailSkeleton />
  }

  if (isError || !affix) {
    return (
      <div className="space-y-4">
        <div className="flex items-center gap-3">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => navigate('/affixes')}
            aria-label="Back to affixes"
          >
            <ArrowLeft className="h-5 w-5" />
          </Button>
          <h2 className="text-2xl font-bold">Affix not found</h2>
        </div>
        <Card>
          <CardContent className="p-8 text-center">
            <p className="text-muted-foreground">
              The affix with ID &quot;{id}&quot; does not exist or has been
              deleted.
            </p>
            <Button
              variant="outline"
              className="mt-4"
              onClick={() => navigate('/affixes')}
            >
              Back to Affixes
            </Button>
          </CardContent>
        </Card>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-start gap-3 flex-wrap">
        <Button
          variant="ghost"
          size="icon"
          onClick={() => navigate('/affixes')}
          aria-label="Back to affixes"
        >
          <ArrowLeft className="h-5 w-5" />
        </Button>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-3 flex-wrap">
            <h2 className="text-2xl font-bold truncate">{affix.name}</h2>
            <Badge
              variant={
                affix.location === 'prefix' ? 'default' : 'secondary'
              }
            >
              {affix.location}
            </Badge>
          </div>
          {affix.description && (
            <p className="text-muted-foreground mt-1">{affix.description}</p>
          )}
        </div>
        <Button onClick={() => setShowEditDialog(true)}>
          <Pencil className="h-4 w-4" />
          Edit
        </Button>
      </div>

      {/* Tabs */}
      <Tabs defaultValue="details">
        <TabsList>
          <TabsTrigger value="details">Details</TabsTrigger>
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

        <TabsContent value="details" className="space-y-4">
          {/* Attribute */}
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Attribute</CardTitle>
            </CardHeader>
            <CardContent>
              <AttributePreview attribute={affix.attribute} gmaMap={gmaMap} />
            </CardContent>
          </Card>

          {/* Metadata */}
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Metadata</CardTitle>
            </CardHeader>
            <CardContent className="space-y-2 text-sm">
              <div className="flex items-center gap-2">
                <span className="text-muted-foreground">ID:</span>
                <code className="font-mono text-xs bg-muted px-1.5 py-0.5 rounded">
                  {affix.id}
                </code>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-muted-foreground">Client ID:</span>
                <code className="font-mono text-xs bg-muted px-1.5 py-0.5 rounded">
                  {affix.client_id}
                </code>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-muted-foreground">Created:</span>
                <span>{formatDateTime(affix.created_at)}</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-muted-foreground">Updated:</span>
                <span>{formatDateTime(affix.updated_at)}</span>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

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
                  No audit log entries for this affix
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
      <AffixCreateEditDialog
        key={affix.id}
        affix={affix}
        open={showEditDialog}
        onOpenChange={setShowEditDialog}
        onSubmit={async (data) => {
          await updateMutation.mutateAsync({
            id: affix.id,
            request: {
              name: data.name,
              type: data.location,
              description: data.description,
              attribute: data.attribute,
            },
          })
          toast({ title: 'Affix updated' })
          setShowEditDialog(false)
        }}
        isPending={updateMutation.isPending}
      />
    </div>
  )
}
