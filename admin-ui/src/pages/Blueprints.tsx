import { useState } from 'react'
import { Plus } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { BlueprintFormModal } from '@/components/BlueprintFormModal'

export default function Blueprints() {
  const [showCreateModal, setShowCreateModal] = useState(false)

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold">Blueprints</h2>
        <Button onClick={() => setShowCreateModal(true)}>
          <Plus className="h-4 w-4" />
          Create Blueprint
        </Button>
      </div>

      <div className="rounded-lg border bg-card text-card-foreground shadow-sm">
        <div className="p-8 text-center text-muted-foreground">
          <p>Blueprints list will appear here.</p>
          <p className="text-sm mt-1">
            (Blueprint list page is a separate issue — see blueprints-list-page)
          </p>
        </div>
      </div>

      <BlueprintFormModal
        open={showCreateModal}
        onOpenChange={setShowCreateModal}
      />
    </div>
  )
}
