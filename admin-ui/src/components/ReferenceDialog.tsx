import { Link } from 'react-router-dom'
import { Blocks, ExternalLink, Tags } from 'lucide-react'

import { Badge } from '@/components/ui/badge'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'

export interface ReferencingBlueprint {
  id: string
  name: string
  key?: string
}

export interface ReferencingAffix {
  id: string
  name: string
}

interface ReferenceDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  title: string
  blueprints: ReferencingBlueprint[]
  affixes: ReferencingAffix[]
}

export function ReferenceDialog({
  open,
  onOpenChange,
  title,
  blueprints,
  affixes,
}: ReferenceDialogProps) {
  const totalRefs = blueprints.length + affixes.length

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
        </DialogHeader>
        {totalRefs === 0 ? (
          <p className="text-sm text-muted-foreground text-center py-8">
            No references found
          </p>
        ) : (
          <div className="space-y-4 max-h-96 overflow-y-auto">
            {blueprints.length > 0 && (
              <div>
                <p className="text-sm font-medium mb-2">
                  Blueprints ({blueprints.length})
                </p>
                <div className="space-y-1">
                  {blueprints.map((bp) => (
                    <Link
                      key={`bp-${bp.id}`}
                      to={`/blueprints/${bp.id}`}
                      onClick={() => onOpenChange(false)}
                      className="flex items-center gap-2 rounded-md px-3 py-2 text-sm hover:bg-muted transition-colors"
                    >
                      <Blocks className="h-4 w-4 text-muted-foreground shrink-0" />
                      <span className="font-medium">{bp.name}</span>
                      {bp.key && (
                        <Badge variant="secondary" className="text-xs font-mono ml-auto">
                          {bp.key}
                        </Badge>
                      )}
                      <ExternalLink className="h-3 w-3 text-muted-foreground shrink-0" />
                    </Link>
                  ))}
                </div>
              </div>
            )}
            {affixes.length > 0 && (
              <div>
                <p className="text-sm font-medium mb-2">
                  Affixes ({affixes.length})
                </p>
                <div className="space-y-1">
                  {affixes.map((aff) => (
                    <Link
                      key={`aff-${aff.id}`}
                      to={`/affixes/${aff.id}`}
                      onClick={() => onOpenChange(false)}
                      className="flex items-center gap-2 rounded-md px-3 py-2 text-sm hover:bg-muted transition-colors"
                    >
                      <Tags className="h-4 w-4 text-muted-foreground shrink-0" />
                      <span className="font-medium">{aff.name}</span>
                      <ExternalLink className="h-3 w-3 text-muted-foreground ml-auto shrink-0" />
                    </Link>
                  ))}
                </div>
              </div>
            )}
          </div>
        )}
      </DialogContent>
    </Dialog>
  )
}
