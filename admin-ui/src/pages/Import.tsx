import { useRef, useState } from 'react'
import {
  AlertTriangle,
  Check,
  FileArchive,
  Loader2,
  Upload,
} from 'lucide-react'

import { useResolveImport } from '@/api/generated'
import { createApiError } from '@/api/generated/errors'
import type {
  ConflictDetail,
  ImportSuccessResponse,
  ResolutionStrategy,
  ResourceResolution,
} from '@/api/generated'
import { useAuth } from '@/stores/auth'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Progress } from '@/components/ui/progress'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
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

const BASE_URL =
  import.meta.env.VITE_ARCHE_API_URL || 'http://localhost:3000'

type WizardStep = 'select' | 'uploading' | 'success' | 'conflicts'

interface ConflictInfo {
  importToken: string
  type: string
  title: string
  status: number
  detail?: string
  conflicts: ConflictDetail[]
}

const STEP_LABELS = ['Select File', 'Upload', 'Result'] as const

function stepIndex(s: WizardStep): number {
  switch (s) {
    case 'select':
      return 0
    case 'uploading':
      return 0
    case 'success':
      return 2
    case 'conflicts':
      return 2
  }
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

function isZip(file: File): boolean {
  return (
    file.name.toLowerCase().endsWith('.zip') ||
    file.type === 'application/zip' ||
    file.type === 'application/x-zip-compressed'
  )
}

export default function ImportPage() {
  const { toast } = useToast()
  const { apiKey } = useAuth()
  const resolveMutation = useResolveImport()

  const [step, setStep] = useState<WizardStep>('select')
  const [file, setFile] = useState<File | null>(null)
  const [dragOver, setDragOver] = useState(false)
  const [importResult, setImportResult] = useState<ImportSuccessResponse | null>(null)
  const [conflictInfo, setConflictInfo] = useState<ConflictInfo | null>(null)

  const [attributeResolutions, setAttributeResolutions] = useState<
    Record<string, Record<string, ResolutionStrategy>>
  >({})
  const [resourceStrategies, setResourceStrategies] = useState<
    Record<string, ResolutionStrategy>
  >({})

  const fileInputRef = useRef<HTMLInputElement>(null)

  const currentStepIndex = stepIndex(step)
  const stepPercent = Math.round((currentStepIndex / (STEP_LABELS.length - 1)) * 100)

  function allResolved(): boolean {
    if (!conflictInfo) return false
    return conflictInfo.conflicts.every((c) => {
      const res = attributeResolutions[c.resource_id]
      if (!res) return false
      return c.attributes.every((a) => res[a.key] !== undefined)
    })
  }

  function handleFile(newFile: File) {
    if (!isZip(newFile)) {
      toast({
        title: 'Invalid file type',
        description: 'Only .zip files are supported.',
        variant: 'destructive',
      })
      return
    }
    setFile(newFile)
  }

  function handleDrop(e: React.DragEvent) {
    e.preventDefault()
    setDragOver(false)
    const dropped = e.dataTransfer.files[0]
    if (dropped) handleFile(dropped)
  }

  function handleDragOver(e: React.DragEvent) {
    e.preventDefault()
    setDragOver(true)
  }

  function handleDragLeave(e: React.DragEvent) {
    e.preventDefault()
    setDragOver(false)
  }

  function handleBrowseClick() {
    fileInputRef.current?.click()
  }

  function handleInputChange(e: React.ChangeEvent<HTMLInputElement>) {
    const selected = e.target.files?.[0]
    if (selected) handleFile(selected)
  }

  async function handleUpload() {
    if (!file) return
    setStep('uploading')

    const formData = new FormData()
    formData.append('file', file)

    try {
      const response = await fetch(`${BASE_URL}/api/import`, {
        method: 'POST',
        headers: {
          ...(apiKey ? { 'X-API-Key': apiKey } : {}),
          Accept: 'application/json',
        },
        body: formData,
      })

      const data = await response.json()

      if (!response.ok) {
        if (response.status === 409) {
          const info: ConflictInfo = {
            importToken: data.import_token ?? '',
            type: data.type ?? '/errors/import-conflict',
            title: data.title ?? 'Import Conflict',
            status: data.status ?? 409,
            detail: data.detail,
            conflicts: data.conflicts ?? [],
          }
          setConflictInfo(info)
          setStep('conflicts')
          return
        }
        throw createApiError(data)
      }

      setImportResult(data as ImportSuccessResponse)
      setStep('success')
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to upload file'
      toast({
        title: 'Import failed',
        description: message,
        variant: 'destructive',
      })
      setStep('select')
    }
  }

  function setAttrResolution(
    resourceId: string,
    attrKey: string,
    strategy: ResolutionStrategy,
  ) {
    setAttributeResolutions((prev) => ({
      ...prev,
      [resourceId]: { ...prev[resourceId], [attrKey]: strategy },
    }))
  }

  function applyToResource(resourceId: string, strategy: ResolutionStrategy) {
    if (!conflictInfo) return
    const res = conflictInfo.conflicts.find(
      (c) => c.resource_id === resourceId,
    )
    if (!res) return
    const attrs: Record<string, ResolutionStrategy> = {}
    for (const a of res.attributes) {
      attrs[a.key] = strategy
    }
    setAttributeResolutions((prev) => ({ ...prev, [resourceId]: attrs }))
    setResourceStrategies((prev) => ({ ...prev, [resourceId]: strategy }))
  }

  function applyToAllRemaining(strategy: ResolutionStrategy) {
    if (!conflictInfo) return
    const newAttrResolutions = { ...attributeResolutions }
    const newResourceStrategies = { ...resourceStrategies }
    for (const c of conflictInfo.conflicts) {
      newResourceStrategies[c.resource_id] = strategy
      const attrs: Record<string, ResolutionStrategy> = {}
      for (const a of c.attributes) {
        attrs[a.key] = strategy
      }
      newAttrResolutions[c.resource_id] = attrs
    }
    setAttributeResolutions(newAttrResolutions)
    setResourceStrategies(newResourceStrategies)
  }

  function getAttrStrategy(resourceId: string, attrKey: string): ResolutionStrategy {
    return attributeResolutions[resourceId]?.[attrKey] ?? 'keepOld'
  }

  async function handleResolve() {
    if (!conflictInfo) return

    const resolutions: Record<string, ResourceResolution> = {}
    for (const c of conflictInfo.conflicts) {
      const attrs = attributeResolutions[c.resource_id]
      if (!attrs) {
        resolutions[c.resource_id] = { strategy: 'keepOld' }
        continue
      }
      const strategies = new Set(Object.values(attrs))
      if (strategies.size === 1) {
        resolutions[c.resource_id] = {
          strategy: strategies.values().next().value!,
        }
      } else {
        resolutions[c.resource_id] = {
          strategy: 'perAttribute',
          attributes: attrs,
        }
      }
    }

    try {
      const result = await resolveMutation.mutateAsync({
        import_token: conflictInfo.importToken,
        resolutions,
      })
      toast({
        title: 'Import complete',
        description: `${result.resources_imported} resources imported across ${result.clients_created} clients.`,
        variant: 'success',
      })
      setImportResult(result)
      setStep('success')
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Resolution failed'
      toast({
        title: 'Resolution failed',
        description: message,
        variant: 'destructive',
      })
    }
  }

  function reset() {
    setStep('select')
    setFile(null)
    setImportResult(null)
    setConflictInfo(null)
    setAttributeResolutions({})
    setResourceStrategies({})
  }

  return (
    <div className="space-y-4">
      <h2 className="text-2xl font-bold">Import</h2>

      <Card>
        <CardContent className="pt-6">
          <div className="mb-6">
            <div className="flex items-center justify-between">
              {STEP_LABELS.map((label, i) => (
                <div
                  key={label}
                  className="flex flex-col items-center gap-1.5 flex-1"
                >
                  <div
                    className={`flex h-8 w-8 items-center justify-center rounded-full text-sm font-medium border-2 transition-colors ${
                      i < currentStepIndex
                        ? 'border-primary bg-primary text-primary-foreground'
                        : i === currentStepIndex
                          ? 'border-primary text-primary'
                          : 'border-muted-foreground/30 text-muted-foreground'
                    }`}
                    aria-current={i === currentStepIndex ? 'step' : undefined}
                  >
                    {i < currentStepIndex ? (
                      <Check className="h-4 w-4" />
                    ) : (
                      i + 1
                    )}
                  </div>
                  <span className="text-xs text-muted-foreground hidden sm:block">
                    {label}
                  </span>
                </div>
              ))}
            </div>
            <Progress value={stepPercent} className="mt-3" />
          </div>

          {step === 'select' && (
            <>
              <div
                onDrop={handleDrop}
                onDragOver={handleDragOver}
                onDragLeave={handleDragLeave}
                onClick={handleBrowseClick}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') handleBrowseClick()
                }}
                className={`relative flex cursor-pointer flex-col items-center justify-center rounded-lg border-2 border-dashed p-8 text-center transition-colors ${
                  dragOver
                    ? 'border-primary bg-primary/5'
                    : 'border-muted-foreground/25 hover:border-primary/50'
                }`}
                role="button"
                tabIndex={0}
                aria-label="Drop zone for ZIP file upload"
              >
                <FileArchive className="h-10 w-10 text-muted-foreground mb-3" />
                <p className="text-sm font-medium">
                  Drag and drop a ZIP file here
                </p>
                <p className="text-xs text-muted-foreground mt-1">
                  or click to browse
                </p>
                <input
                  ref={fileInputRef}
                  type="file"
                  accept=".zip,application/zip,application/x-zip-compressed"
                  onChange={handleInputChange}
                  className="hidden"
                  aria-label="Browse files"
                />
              </div>

              {file && (
                <div className="mt-4 flex items-center gap-3 rounded-lg border bg-muted/50 p-3">
                  <FileArchive className="h-8 w-8 text-primary shrink-0" />
                  <div className="min-w-0 flex-1">
                    <p className="text-sm font-medium truncate">
                      {file.name}
                    </p>
                    <p className="text-xs text-muted-foreground">
                      {formatSize(file.size)}
                    </p>
                  </div>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={(e) => {
                      e.stopPropagation()
                      setFile(null)
                    }}
                    aria-label="Remove file"
                  >
                    Remove
                  </Button>
                </div>
              )}

              {!file && (
                <Button
                  variant="outline"
                  className="mt-3 w-full"
                  onClick={handleBrowseClick}
                >
                  Browse files
                </Button>
              )}

              <Button
                className="mt-4 w-full"
                disabled={!file}
                onClick={handleUpload}
              >
                <Upload className="h-4 w-4" />
                Upload and Import
              </Button>
            </>
          )}

          {step === 'uploading' && (
            <div className="flex flex-col items-center justify-center py-12">
              <Loader2 className="h-8 w-8 animate-spin text-primary" />
              <p className="mt-4 text-sm text-muted-foreground">
                Uploading and parsing archive&hellip;
              </p>
              <Skeleton className="mt-4 h-2 w-48" />
            </div>
          )}

          {step === 'success' && importResult && (
            <div className="flex flex-col items-center py-6">
              <div className="flex h-12 w-12 items-center justify-center rounded-full bg-primary/10">
                <Check className="h-6 w-6 text-primary" />
              </div>
              <CardTitle className="mt-4">Import Successful</CardTitle>
              <p className="mt-1 text-sm text-muted-foreground text-center">
                {importResult.clients_created} clients created,&nbsp;
                {importResult.resources_imported} resources imported.
              </p>
              <div className="mt-4 flex gap-3">
                <Button variant="outline" onClick={reset}>
                  Start New Import
                </Button>
              </div>
            </div>
          )}

          {step === 'conflicts' && conflictInfo && (
            <div className="space-y-4">
              <div className="flex items-start gap-3 rounded-lg border border-amber-500/50 bg-amber-500/10 p-3">
                <AlertTriangle className="h-5 w-5 text-amber-500 shrink-0 mt-0.5" />
                <div>
                  <p className="text-sm font-medium">
                    {conflictInfo.title}
                  </p>
                  {conflictInfo.detail && (
                    <p className="text-xs text-muted-foreground mt-0.5">
                      {conflictInfo.detail}
                    </p>
                  )}
                </div>
              </div>

              <div className="flex items-center gap-2">
                <span className="text-xs text-muted-foreground">
                  Apply to all remaining:
                </span>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => applyToAllRemaining('keepOld')}
                >
                  Keep Existing
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => applyToAllRemaining('keepNew')}
                >
                  Use Imported
                </Button>
              </div>

              <div className="space-y-4">
                {conflictInfo.conflicts.map((conflict) => (
                  <ConflictRow
                    key={conflict.resource_id}
                    conflict={conflict}
                    getAttrStrategy={getAttrStrategy}
                    setAttrResolution={setAttrResolution}
                    resourceStrategy={
                      resourceStrategies[conflict.resource_id]
                    }
                    applyToResource={applyToResource}
                  />
                ))}
              </div>

              <div className="flex gap-2 justify-end">
                <Button variant="outline" onClick={reset}>
                  Cancel
                </Button>
                <Button
                  disabled={!allResolved() || resolveMutation.isPending}
                  onClick={handleResolve}
                >
                  {resolveMutation.isPending ? (
                    <>
                      <Loader2 className="h-4 w-4 animate-spin" />
                      Applying&hellip;
                    </>
                  ) : (
                    <>
                      <Check className="h-4 w-4" />
                      Apply Resolutions
                    </>
                  )}
                </Button>
              </div>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}

function ConflictRow({
  conflict,
  getAttrStrategy,
  setAttrResolution,
  resourceStrategy,
  applyToResource,
}: {
  conflict: ConflictDetail
  getAttrStrategy: (resourceId: string, attrKey: string) => ResolutionStrategy
  setAttrResolution: (
    resourceId: string,
    attrKey: string,
    strategy: ResolutionStrategy,
  ) => void
  resourceStrategy?: ResolutionStrategy
  applyToResource: (resourceId: string, strategy: ResolutionStrategy) => void
}) {
  return (
    <Card>
      <CardHeader className="pb-2">
        <div className="flex items-center gap-2 flex-wrap">
          <Badge variant="secondary">{conflict.resource_type}</Badge>
          <span className="font-medium">{conflict.resource_name}</span>
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="flex items-center gap-2">
          <span className="text-xs text-muted-foreground">
            Apply to all attributes:
          </span>
          <Select
            value={resourceStrategy ?? ''}
            onValueChange={(v) =>
              applyToResource(conflict.resource_id, v as ResolutionStrategy)
            }
          >
            <SelectTrigger className="h-7 text-xs w-[160px]">
              <SelectValue placeholder="Choose&hellip;" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="keepOld">Keep Existing</SelectItem>
              <SelectItem value="keepNew">Use Imported</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div className="rounded-md border bg-background">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Attribute</TableHead>
                <TableHead>Existing Value</TableHead>
                <TableHead>Imported Value</TableHead>
                <TableHead className="w-[150px]">Resolution</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {conflict.attributes.map((attr) => {
                const current = getAttrStrategy(conflict.resource_id, attr.key)
                return (
                  <TableRow key={attr.key}>
                    <TableCell className="font-mono text-xs">
                      {attr.key}
                    </TableCell>
                    <TableCell className="text-xs max-w-[200px] truncate">
                      {formatValue(attr.oldValue)}
                    </TableCell>
                    <TableCell className="text-xs max-w-[200px] truncate">
                      {formatValue(attr.newValue)}
                    </TableCell>
                    <TableCell>
                      <Select
                        value={current}
                        onValueChange={(v) =>
                          setAttrResolution(
                            conflict.resource_id,
                            attr.key,
                            v as ResolutionStrategy,
                          )
                        }
                      >
                        <SelectTrigger className="h-7 text-xs w-full">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="keepOld">
                            Keep Existing
                          </SelectItem>
                          <SelectItem value="keepNew">
                            Use Imported
                          </SelectItem>
                        </SelectContent>
                      </Select>
                    </TableCell>
                  </TableRow>
                )
              })}
            </TableBody>
          </Table>
        </div>
      </CardContent>
    </Card>
  )
}

function formatValue(value: unknown): string {
  if (value === null || value === undefined) return '-'
  if (typeof value === 'object') return JSON.stringify(value)
  return String(value)
}
