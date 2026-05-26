import { useMemo, useState } from 'react'
import { useNavigate, useParams, Link } from 'react-router-dom'
import {
  ArrowLeft,
  ChevronDown,
  ChevronRight,
  Clock,
  FileText,
  Link2,
  Pencil,
} from 'lucide-react'
import {
  useAffixesList,
  useAuditLog,
  useBlueprintsList,
  useGlobalMetaAttribute,
  useUpdateGlobalMetaAttribute,
} from '@/api/generated/hooks'
import type {
  GlobalMetaAttribute,
  AuditLogEntry,
} from '@/api/generated/types'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Skeleton } from '@/components/ui/skeleton'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { useToast } from '@/hooks/use-toast'

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
        <span className="text-muted-foreground">by {entry.actorKeyName}</span>
        <span className="ml-auto text-xs text-muted-foreground">
          {formatDateTime(entry.timestamp)}
        </span>
      </button>
      {expanded && (
        <div className="border-t p-3 space-y-3">
          {entry.before && entry.after ? (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              <div>
                <p className="text-xs font-medium text-destructive mb-1">Before</p>
                <pre className="text-xs font-mono bg-muted rounded-md p-2 overflow-auto max-h-64 whitespace-pre-wrap break-all">
                  {formatJson(entry.before)}
                </pre>
              </div>
              <div>
                <p className="text-xs font-medium text-green-600 dark:text-green-400 mb-1">After</p>
                <pre className="text-xs font-mono bg-muted rounded-md p-2 overflow-auto max-h-64 whitespace-pre-wrap break-all">
                  {formatJson(entry.after)}
                </pre>
              </div>
            </div>
          ) : entry.after ? (
            <div>
              <p className="text-xs font-medium text-green-600 dark:text-green-400 mb-1">Created</p>
              <pre className="text-xs font-mono bg-muted rounded-md p-2 overflow-auto max-h-64 whitespace-pre-wrap break-all">
                {formatJson(entry.after)}
              </pre>
            </div>
          ) : entry.before ? (
            <div>
              <p className="text-xs font-medium text-destructive mb-1">Deleted</p>
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

export default function GlobalMetaAttributeDetail() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [showEditDialog, setShowEditDialog] = useState(false)

  const {
    data: gma,
    isLoading,
    isError,
  } = useGlobalMetaAttribute(id ?? '')

  const { data: bpData } = useBlueprintsList({ perPage: 500 })
  const { data: affData } = useAffixesList({ perPage: 500 })

  const {
    data: auditData,
    isLoading: auditLoading,
  } = useAuditLog({ resourceType: 'global_meta_attribute' })

  const auditEntries = ((auditData?.data ?? []) as AuditLogEntry[]).filter(
    (entry) => entry.resourceId === id,
  )

  const referencedBy = useMemo(() => {
    const blueprints: { id: string; name: string; key: string }[] = []
    const affixes: { id: string; name: string }[] = []

    const bpList = (bpData?.data ?? []) as { id: string; name: string; attributes: Record<string, unknown> }[]
    const affList = (affData?.data ?? []) as { id: string; name: string; attribute: unknown }[]

    for (const bp of bpList) {
      const attrs = bp.attributes
      if (!attrs) continue
      for (const [key, v] of Object.entries(attrs)) {
        if (typeof v === 'object' && v !== null && '$ref_id' in (v as Record<string, unknown>)) {
          const refId = (v as Record<string, unknown>).$ref_id as string
          if (refId === id) {
            blueprints.push({ id: bp.id, name: bp.name, key })
          }
        }
      }
    }

    for (const aff of affList) {
      const attr = aff.attribute
      if (typeof attr === 'object' && attr !== null && '$ref_id' in (attr as Record<string, unknown>)) {
        const refId = (attr as Record<string, unknown>).$ref_id as string
        if (refId === id) {
          affixes.push({ id: aff.id, name: aff.name })
        }
      }
    }

    return { blueprints, affixes }
  }, [bpData?.data, affData?.data, id])

  if (isLoading) {
    return <DetailSkeleton />
  }

  if (isError || !gma) {
    return (
      <div className="space-y-4">
        <div className="flex items-center gap-3">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => navigate('/global-meta-attributes')}
            aria-label="Back to global meta attributes"
          >
            <ArrowLeft className="h-5 w-5" />
          </Button>
          <h2 className="text-2xl font-bold">Global Meta Attribute not found</h2>
        </div>
        <Card>
          <CardContent className="p-8 text-center">
            <p className="text-muted-foreground">
              The global meta attribute with ID &quot;{id}&quot; does not exist or
              has been deleted.
            </p>
            <Button
              variant="outline"
              className="mt-4"
              onClick={() => navigate('/global-meta-attributes')}
            >
              Back to Global Meta Attributes
            </Button>
          </CardContent>
        </Card>
      </div>
    )
  }

  const totalRefs = referencedBy.blueprints.length + referencedBy.affixes.length

  return (
    <div className="space-y-6">
      <div className="flex items-start gap-3 flex-wrap">
        <Button
          variant="ghost"
          size="icon"
          onClick={() => navigate('/global-meta-attributes')}
          aria-label="Back to global meta attributes"
        >
          <ArrowLeft className="h-5 w-5" />
        </Button>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-3 flex-wrap">
            <h2 className="text-2xl font-bold truncate">{gma.name}</h2>
            <Badge variant="outline">{gma.valueType}</Badge>
          </div>
          {gma.description && (
            <p className="text-muted-foreground mt-1">{gma.description}</p>
          )}
        </div>
        <Button onClick={() => setShowEditDialog(true)}>
          <Pencil className="h-4 w-4" />
          Edit
        </Button>
      </div>

      <Tabs defaultValue="definition">
        <TabsList>
          <TabsTrigger value="definition">
            <FileText className="h-3.5 w-3.5" />
            Definition
          </TabsTrigger>
          <TabsTrigger value="referenced-by">
            <Link2 className="h-3.5 w-3.5" />
            Referenced By
            {totalRefs > 0 && (
              <span className="ml-1.5 rounded-full bg-muted-foreground/20 px-1.5 py-0.5 text-xs">
                {totalRefs}
              </span>
            )}
          </TabsTrigger>
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

        <TabsContent value="definition" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Attribute Definition</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div>
                  <p className="text-sm text-muted-foreground">Name</p>
                  <p className="font-medium">{gma.name}</p>
                </div>
                <div>
                  <p className="text-sm text-muted-foreground">Value Type</p>
                  <Badge variant="outline">{gma.valueType}</Badge>
                </div>
                <div>
                  <p className="text-sm text-muted-foreground">Preview</p>
                  <p className="font-mono text-sm">{getPreview(gma)}</p>
                </div>
                <div>
                  <p className="text-sm text-muted-foreground">Description</p>
                  <p>
                    {gma.description || (
                      <span className="text-muted-foreground/60">No description</span>
                    )}
                  </p>
                </div>
              </div>
              <div>
                <p className="text-sm text-muted-foreground mb-1">Payload</p>
                <pre className="text-xs font-mono bg-muted rounded-md p-3 overflow-auto max-h-64 whitespace-pre-wrap break-all">
                  {formatJson(gma.payload)}
                </pre>
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
                  {gma.id}
                </code>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-muted-foreground">Client ID:</span>
                <code className="font-mono text-xs bg-muted px-1.5 py-0.5 rounded">
                  {gma.clientId}
                </code>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-muted-foreground">Created:</span>
                <span>{formatDateTime(gma.createdAt)}</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-muted-foreground">Updated:</span>
                <span>{formatDateTime(gma.updatedAt)}</span>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="referenced-by">
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Referenced By</CardTitle>
            </CardHeader>
            <CardContent>
              {totalRefs === 0 ? (
                <p className="text-sm text-muted-foreground text-center py-8">
                  This global meta attribute is not referenced by any blueprints
                  or affixes
                </p>
              ) : (
                <div className="space-y-4">
                  {referencedBy.blueprints.length > 0 && (
                    <div>
                      <p className="text-sm font-medium mb-2">
                        Blueprints ({referencedBy.blueprints.length})
                      </p>
                      <div className="space-y-1">
                        {referencedBy.blueprints.map((bp) => (
                          <div
                            key={`${bp.id}-${bp.key}`}
                            className="flex items-center gap-2 text-sm"
                          >
                            <Link
                              to={`/blueprints/${bp.id}`}
                              className="text-primary hover:underline"
                            >
                              {bp.name}
                            </Link>
                            <Badge variant="secondary" className="text-xs font-mono">
                              {bp.key}
                            </Badge>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                  {referencedBy.affixes.length > 0 && (
                    <div>
                      <p className="text-sm font-medium mb-2">
                        Affixes ({referencedBy.affixes.length})
                      </p>
                      <div className="space-y-1">
                        {referencedBy.affixes.map((aff) => (
                          <div key={aff.id} className="flex items-center gap-2 text-sm">
                            <Link
                              to={`/affixes/${aff.id}`}
                              className="text-primary hover:underline"
                            >
                              {aff.name}
                            </Link>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              )}
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
                  No audit log entries for this global meta attribute
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

      <EditGlobalMetaAttributeDialog
        key={gma.id}
        gma={gma}
        open={showEditDialog}
        onOpenChange={setShowEditDialog}
      />
    </div>
  )
}
