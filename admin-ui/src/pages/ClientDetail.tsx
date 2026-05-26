import { useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import {
  ArrowLeft,
  Check,
  Copy,
  Key,
  Plus,
  ShieldAlert,
  Trash2,
} from 'lucide-react'

import {
  useClient,
  useApiKeys,
  useCreateApiKey,
  useDeleteApiKey,
  useDeleteClient,
} from '@/api/generated'
import type { Permission, CreateApiKeyResponse } from '@/api/generated'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Skeleton } from '@/components/ui/skeleton'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { formatDate } from '@/lib/utils'
import { toastSuccess, toastError } from '@/lib/toast-helpers'

const ALL_PERMISSIONS: { value: Permission; label: string }[] = [
  { value: 'read', label: 'Read' },
  { value: 'write', label: 'Write' },
  { value: 'delete', label: 'Delete' },
  { value: 'generate', label: 'Generate' },
  { value: 'admin', label: 'Admin' },
]

function permissionBadgeVariant(p: Permission) {
  switch (p) {
    case 'admin': return 'destructive' as const
    case 'delete': return 'destructive' as const
    case 'write': return 'default' as const
    case 'generate': return 'secondary' as const
    case 'read': return 'outline' as const
  }
}

export default function ClientDetail() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()

  const { data: client, isLoading, isError } = useClient(id!)
  const { data: apiKeys = [], isLoading: keysLoading } = useApiKeys(id!)
  const createKeyMutation = useCreateApiKey()
  const deleteKeyMutation = useDeleteApiKey()
  const deleteClientMutation = useDeleteClient()

  const [keyModalOpen, setKeyModalOpen] = useState(false)
  const [keyName, setKeyName] = useState('')
  const [keyPermissions, setKeyPermissions] = useState<Set<Permission>>(new Set(['read']))
  const [createdKey, setCreatedKey] = useState<CreateApiKeyResponse | null>(null)
  const [keyCopied, setKeyCopied] = useState(false)
  const [revokingKey, setRevokingKey] = useState<{ id: string; name: string } | null>(null)
  const [deleteOpen, setDeleteOpen] = useState(false)
  const [deleteConfirmation, setDeleteConfirmation] = useState('')

  function togglePermission(p: Permission) {
    setKeyPermissions((prev) => {
      const next = new Set(prev)
      if (next.has(p)) {
        next.delete(p)
      } else {
        next.add(p)
      }
      return next
    })
  }

  async function handleCreateKey() {
    const name = keyName.trim()
    if (!name || keyPermissions.size === 0) return
    try {
      const result = await createKeyMutation.mutateAsync({
        clientId: id!,
        request: { name, permissions: Array.from(keyPermissions) },
      })
      setCreatedKey(result)
      setKeyCopied(false)
      toastSuccess('api-key', 'created', name)
    } catch (err) {
      toastError('api-key', 'Create', err, name)
    }
  }

  function handleCopyKey() {
    if (createdKey) {
      navigator.clipboard.writeText(createdKey.key)
      setKeyCopied(true)
    }
  }

  function handleKeyModalClose() {
    if (createdKey && !keyCopied) return
    setKeyModalOpen(false)
    setKeyName('')
    setKeyPermissions(new Set(['read']))
    setCreatedKey(null)
    setKeyCopied(false)
  }

  async function handleRevokeKey() {
    if (!revokingKey) return
    try {
      await deleteKeyMutation.mutateAsync({ clientId: id!, keyId: revokingKey.id })
      toastSuccess('api-key', 'revoked', revokingKey.name)
      setRevokingKey(null)
    } catch (err) {
      toastError('api-key', 'Revoke', err, revokingKey.name)
      setRevokingKey(null)
    }
  }

  async function handleDeleteClient() {
    if (!client || deleteConfirmation !== client.name) return
    try {
      await deleteClientMutation.mutateAsync(id!)
      toastSuccess('client', 'deleted', client.name)
      navigate('/clients')
    } catch (err) {
      toastError('client', 'Delete', err, client.name)
      setDeleteOpen(false)
      setDeleteConfirmation('')
    }
  }

  if (isLoading) {
    return (
      <div className="space-y-6">
        <Skeleton className="h-8 w-48" />
        <Card>
          <CardContent className="p-6 space-y-4">
            <Skeleton className="h-6 w-64" />
            <Skeleton className="h-4 w-32" />
            <Skeleton className="h-32 w-full" />
          </CardContent>
        </Card>
      </div>
    )
  }

  if (isError || !client) {
    return (
      <Card>
        <CardContent className="p-6">
          <p className="text-destructive">Failed to load client.</p>
          <Button variant="outline" className="mt-4" onClick={() => navigate('/clients')}>
            <ArrowLeft className="mr-2 h-4 w-4" />
            Back to clients
          </Button>
        </CardContent>
      </Card>
    )
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-4">
        <Button variant="ghost" size="icon" onClick={() => navigate('/clients')} aria-label="Back to clients">
          <ArrowLeft className="h-5 w-5" />
        </Button>
        <div>
          <h1 className="text-2xl font-bold">{client.name}</h1>
          <p className="text-sm text-muted-foreground">
            Created {formatDate(client.createdAt)}
          </p>
        </div>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Key className="h-5 w-5" />
            API Keys
          </CardTitle>
          <CardDescription>
            Manage API keys for this client. Keys grant access to the Arche API with specific permissions.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex justify-end">
            <Button onClick={() => setKeyModalOpen(true)}>
              <Plus className="mr-2 h-4 w-4" />
              Create Key
            </Button>
          </div>

          {keysLoading ? (
            <div className="space-y-3">
              {Array.from({ length: 3 }).map((_, i) => (
                <Skeleton key={i} className="h-12 w-full" />
              ))}
            </div>
          ) : apiKeys.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              <Key className="mx-auto h-8 w-8 mb-2 opacity-50" />
              <p>No API keys yet. Create one to get started.</p>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>Permissions</TableHead>
                  <TableHead>ID</TableHead>
                  <TableHead className="w-[100px]">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {apiKeys.map((key) => (
                  <TableRow key={key.id}>
                    <TableCell className="font-medium">{key.name}</TableCell>
                    <TableCell>
                      <div className="flex flex-wrap gap-1">
                        {key.permissions.map((p) => (
                          <Badge key={p} variant={permissionBadgeVariant(p)}>
                            {p}
                          </Badge>
                        ))}
                      </div>
                    </TableCell>
                    <TableCell className="text-muted-foreground">
                      {key.id}
                    </TableCell>
                    <TableCell>
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => setRevokingKey({ id: key.id, name: key.name })}
                        title="Revoke key"
                      >
                        <Trash2 className="h-4 w-4 text-destructive" />
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      <Card className="border-destructive/50">
        <CardHeader>
          <CardTitle className="text-destructive flex items-center gap-2">
            <ShieldAlert className="h-5 w-5" />
            Danger Zone
          </CardTitle>
          <CardDescription>
            Deleting this client will cascade delete all associated blueprints, affixes,
            global meta attributes, and API keys. This action cannot be undone.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Button variant="destructive" onClick={() => setDeleteOpen(true)}>
            <Trash2 className="mr-2 h-4 w-4" />
            Delete Client
          </Button>
        </CardContent>
      </Card>

      <Dialog open={keyModalOpen} onOpenChange={(open) => { if (!open) handleKeyModalClose() }}>
        <DialogContent className="max-w-md">
          {createdKey ? (
            <>
              <DialogHeader>
                <DialogTitle>API Key Created</DialogTitle>
                <DialogDescription>
                  Copy this key now. You won&apos;t be able to see it again.
                </DialogDescription>
              </DialogHeader>
              <div className="space-y-4">
                <div className="relative">
                  <Input
                    readOnly
                    value={createdKey.key}
                    className="font-mono text-sm pr-20 bg-muted"
                    onClick={(e) => (e.target as HTMLInputElement).select()}
                  />
                  <Button
                    size="sm"
                    variant={keyCopied ? 'default' : 'outline'}
                    className="absolute right-1 top-1 h-8"
                    onClick={handleCopyKey}
                  >
                    {keyCopied ? (
                      <>
                        <Check className="mr-1 h-3 w-3" />
                        Copied
                      </>
                    ) : (
                      <>
                        <Copy className="mr-1 h-3 w-3" />
                        Copy
                      </>
                    )}
                  </Button>
                </div>
                <p className="text-sm text-muted-foreground">
                  <strong>{createdKey.name}</strong> with permissions:{' '}
                  {createdKey.permissions.map((p) => (
                    <Badge key={p} variant={permissionBadgeVariant(p)} className="ml-1">
                      {p}
                    </Badge>
                  ))}
                </p>
                <div className="bg-amber-50 dark:bg-amber-950 border border-amber-200 dark:border-amber-800 rounded-md p-3 text-sm">
                  Store this key securely. It cannot be retrieved later. If lost, you must revoke it and create a new one.
                </div>
              </div>
              <DialogFooter>
                <Button onClick={handleKeyModalClose} disabled={!keyCopied}>
                  {keyCopied ? 'Done' : 'Copy key to continue'}
                </Button>
              </DialogFooter>
            </>
          ) : (
            <>
              <DialogHeader>
                <DialogTitle>Create API Key</DialogTitle>
                <DialogDescription>
                  Create an API key for client <strong>{client.name}</strong>.
                </DialogDescription>
              </DialogHeader>
              <div className="space-y-4 py-4">
                <div className="space-y-2">
                  <Label htmlFor="key-name">Key Name</Label>
                  <Input
                    id="key-name"
                    placeholder="e.g. game-server-key"
                    value={keyName}
                    onChange={(e) => setKeyName(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter') handleCreateKey()
                    }}
                  />
                </div>
                <div className="space-y-2">
                  <Label>Permissions</Label>
                  <div className="grid grid-cols-2 gap-2">
                    {ALL_PERMISSIONS.map((p) => (
                      <label
                        key={p.value}
                        className={`flex items-center gap-2 p-2 rounded-md border cursor-pointer transition-colors ${
                          keyPermissions.has(p.value)
                            ? 'border-primary bg-primary/5'
                            : 'border-border hover:border-primary/50'
                        }`}
                      >
                        <input
                          type="checkbox"
                          checked={keyPermissions.has(p.value)}
                          onChange={() => togglePermission(p.value)}
                          className="sr-only"
                        />
                        <div
                          className={`w-4 h-4 rounded border-2 flex items-center justify-center ${
                            keyPermissions.has(p.value)
                              ? 'bg-primary border-primary'
                              : 'border-muted-foreground/30'
                          }`}
                        >
                          {keyPermissions.has(p.value) && <Check className="h-3 w-3 text-primary-foreground" />}
                        </div>
                        <span className="text-sm">{p.label}</span>
                      </label>
                    ))}
                  </div>
                </div>
              </div>
              <DialogFooter>
                <Button variant="outline" onClick={handleKeyModalClose} disabled={createKeyMutation.isPending}>
                  Cancel
                </Button>
                <Button
                  onClick={handleCreateKey}
                  disabled={createKeyMutation.isPending || !keyName.trim() || keyPermissions.size === 0}
                >
                  {createKeyMutation.isPending ? 'Creating...' : 'Create Key'}
                </Button>
              </DialogFooter>
            </>
          )}
        </DialogContent>
      </Dialog>

      <ConfirmDialog
        open={revokingKey !== null}
        onOpenChange={(open) => { if (!open) setRevokingKey(null) }}
        title="Revoke API Key"
        description={
          revokingKey
            ? `Are you sure you want to revoke "${revokingKey.name}"? This will immediately invalidate the key. Any services using this key will lose access.`
            : ''
        }
        confirmLabel="Revoke"
        variant="destructive"
        loading={deleteKeyMutation.isPending}
        onConfirm={handleRevokeKey}
      />

      <Dialog open={deleteOpen} onOpenChange={setDeleteOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="text-destructive">Delete Client</DialogTitle>
            <DialogDescription>
              This will permanently delete the client <strong>{client.name}</strong> and cascade delete
              all associated blueprints, affixes, global meta attributes, and API keys.
              <br /><br />
              This action cannot be undone.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <p className="text-sm text-muted-foreground">
              Type <strong>{client.name}</strong> to confirm:
            </p>
            <Input
              value={deleteConfirmation}
              onChange={(e) => setDeleteConfirmation(e.target.value)}
              placeholder={client.name}
            />
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => { setDeleteOpen(false); setDeleteConfirmation('') }} disabled={deleteClientMutation.isPending}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={handleDeleteClient}
              disabled={deleteClientMutation.isPending || deleteConfirmation !== client.name}
            >
              {deleteClientMutation.isPending ? 'Deleting...' : 'Delete Client'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
