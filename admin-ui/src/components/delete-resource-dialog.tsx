import { AlertTriangle, ExternalLink } from 'lucide-react'
import { Link } from 'react-router-dom'

import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'

export interface ReferencingResource {
  id: string
  name: string
  type: string
  detailUrl: string
}

export type ResourceTypeLabel =
  | 'blueprint'
  | 'affix'
  | 'global meta attribute'
  | 'client'

export interface DeleteResourceDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  resourceName: string
  resourceType: ResourceTypeLabel
  referencingResources: ReferencingResource[]
  onForceDelete: () => void
  isDeleting?: boolean
}

export function DeleteResourceDialog({
  open,
  onOpenChange,
  resourceName,
  resourceType,
  referencingResources,
  onForceDelete,
  isDeleting = false,
}: DeleteResourceDialogProps) {
  const hasReferences = referencingResources.length > 0

  let buttonLabel = 'Delete'
  if (isDeleting) {
    buttonLabel = 'Deleting...'
  } else if (hasReferences) {
    buttonLabel = 'Force Delete'
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>
            Delete {resourceType}
          </DialogTitle>
          <DialogDescription>
            Are you sure you want to delete{' '}
            <span className="font-semibold text-foreground">&ldquo;{resourceName}&rdquo;</span>?
          </DialogDescription>
        </DialogHeader>

        {hasReferences && (
          <div className="space-y-3">
            <div className="flex items-start gap-2 rounded-md border border-amber-200 bg-amber-50 p-3 text-amber-900 dark:border-amber-800 dark:bg-amber-950 dark:text-amber-100">
              <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0" />
              <div className="text-sm">
                This action cannot be undone.{' '}
                {referencingResources.length} resource{referencingResources.length !== 1 ? 's' : ''}{' '}
                will be modified.
              </div>
            </div>

            <div>
              <p className="mb-2 text-sm font-medium">Referenced by:</p>
              <ul className="max-h-48 space-y-1 overflow-y-auto rounded-md border bg-background p-2">
                {referencingResources.map((ref) => (
                  <li key={`${ref.type}-${ref.id}`}>
                    <Link
                      to={ref.detailUrl}
                      className="inline-flex items-center gap-1 rounded px-1 text-sm text-primary hover:underline"
                    >
                      <span className="font-medium">{ref.name}</span>
                      <span className="text-muted-foreground">({ref.type})</span>
                      <ExternalLink className="h-3 w-3" />
                    </Link>
                  </li>
                ))}
              </ul>
            </div>
          </div>
        )}

        <DialogFooter className="gap-2 sm:gap-0">
          <DialogClose asChild>
            <Button variant="outline">Cancel</Button>
          </DialogClose>
          <Button
            variant="destructive"
            onClick={onForceDelete}
            disabled={isDeleting}
          >
            {buttonLabel}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
