import { useState, useMemo } from 'react'
import { Download, Loader2 } from 'lucide-react'

import { useClientsList, useExportData } from '@/api/generated'
import type { ClientResponse, ExportRequest } from '@/api/generated'

import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
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

export default function ExportPage() {
  const { toast } = useToast()
  const { data, isLoading, isError, error } = useClientsList({ perPage: 500 })
  const exportMutation = useExportData()

  const clients = useMemo<ClientResponse[]>(
    () => data?.data ?? [],
    [data?.data],
  )

  const [selectedIds, setSelectedIds] = useState<Set<string> | null>(null)
  const [includeApiKeys, setIncludeApiKeys] = useState(false)
  const [includeAuditLog, setIncludeAuditLog] = useState(false)
  const [inlineGlobalRefs, setInlineGlobalRefs] = useState(false)

  const effectiveSelectedIds = useMemo(() => {
    if (selectedIds !== null) return selectedIds
    return new Set(clients.map((c) => c.id))
  }, [selectedIds, clients])

  const allSelected =
    clients.length > 0 && clients.every((c) => effectiveSelectedIds.has(c.id))

  function toggleClient(id: string) {
    setSelectedIds((prev) => {
      const base = prev ?? new Set(clients.map((c) => c.id))
      const next = new Set(base)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  }

  function toggleAll() {
    if (allSelected) {
      setSelectedIds(new Set())
    } else {
      setSelectedIds(new Set(clients.map((c) => c.id)))
    }
  }

  async function handleExport() {
    const request: ExportRequest = {
      clientIds: Array.from(effectiveSelectedIds),
      includeApiKeys,
      includeAuditLog,
      inlineGlobalRefs,
    }
    try {
      const buffer = await exportMutation.mutateAsync(request)
      const blob = new Blob([buffer], { type: 'application/zip' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `arche-export-${new Date().toISOString().slice(0, 10)}.zip`
      document.body.appendChild(a)
      a.click()
      document.body.removeChild(a)
      URL.revokeObjectURL(url)
      toast({ title: 'Export complete' })
    } catch (err) {
      toast({
        title: 'Export failed',
        description: err instanceof Error ? err.message : 'Unknown error',
        variant: 'destructive',
      })
    }
  }

  return (
    <div className="space-y-4">
      <h2 className="text-2xl font-bold">Export</h2>

      {isLoading && (
        <Card>
          <CardContent className="p-4 space-y-2">
            {Array.from({ length: 5 }).map((_, i) => (
              <div key={i} className="flex items-center gap-3">
                <Skeleton className="h-4 w-4 rounded-sm" />
                <Skeleton className="h-4 w-40" />
              </div>
            ))}
          </CardContent>
        </Card>
      )}

      {isError && (
        <Card>
          <CardContent className="p-8 text-center">
            <p className="text-destructive font-medium">
              Failed to load clients
            </p>
            <p className="text-sm text-muted-foreground mt-1">
              {error instanceof Error ? error.message : 'Unknown error'}
            </p>
          </CardContent>
        </Card>
      )}

      {!isLoading && !isError && clients.length === 0 && (
        <Card>
          <CardContent className="p-8 text-center">
            <p className="text-muted-foreground">No clients found</p>
          </CardContent>
        </Card>
      )}

      {!isLoading && !isError && clients.length > 0 && (
        <>
          <Card>
            <CardContent className="p-0">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="w-10">
                      <input
                        type="checkbox"
                        checked={allSelected}
                        onChange={toggleAll}
                        aria-label="Select all clients"
                        className="rounded"
                      />
                    </TableHead>
                    <TableHead>Client Name</TableHead>
                    <TableHead>Created</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {clients.map((client) => (
                    <TableRow key={client.id}>
                      <TableCell>
                        <input
                          type="checkbox"
                          checked={effectiveSelectedIds.has(client.id)}
                          onChange={() => toggleClient(client.id)}
                          aria-label={`Select ${client.name}`}
                          className="rounded"
                        />
                      </TableCell>
                      <TableCell className="font-medium">
                        {client.name}
                      </TableCell>
                      <TableCell className="text-muted-foreground text-sm">
                        {new Date(client.createdAt).toLocaleDateString()}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </CardContent>
          </Card>

          <Card>
            <CardContent className="p-4 space-y-4">
              <h3 className="font-medium">Export Options</h3>

              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={includeApiKeys}
                  onChange={(e) => setIncludeApiKeys(e.target.checked)}
                  className="rounded"
                />
                <span className="text-sm">Export API keys</span>
              </label>

              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={includeAuditLog}
                  onChange={(e) => setIncludeAuditLog(e.target.checked)}
                  className="rounded"
                />
                <span className="text-sm">Export audit log</span>
              </label>

              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={inlineGlobalRefs}
                  onChange={(e) => setInlineGlobalRefs(e.target.checked)}
                  className="rounded"
                />
                <span className="text-sm">Inline global references</span>
              </label>
            </CardContent>
          </Card>

          <Button
            onClick={handleExport}
            disabled={
              exportMutation.isPending || effectiveSelectedIds.size === 0
            }
            className="w-full sm:w-auto"
          >
            {exportMutation.isPending ? (
              <>
                <Loader2 className="h-4 w-4 animate-spin" />
                Preparing export...
              </>
            ) : (
              <>
                <Download className="h-4 w-4" />
                Export {effectiveSelectedIds.size}{' '}
                {effectiveSelectedIds.size === 1 ? 'client' : 'clients'}
              </>
            )}
          </Button>
        </>
      )}
    </div>
  )
}
