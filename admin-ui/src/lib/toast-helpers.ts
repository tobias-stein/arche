import { isApiError } from '@/api/generated/errors'
import { toast as toastFn } from '@/hooks/use-toast'

const RESOURCE_LABELS: Record<string, string> = {
  blueprint: 'Blueprint',
  affix: 'Affix',
  ['global-meta-attribute']: 'Global meta attribute',
  client: 'Client',
  ['api-key']: 'API key',
}

function resourceLabel(type: string): string {
  return RESOURCE_LABELS[type] ?? type
}

export function toastSuccess(resourceType: string, action: string, name?: string) {
  const label = resourceLabel(resourceType)
  const title = `${label} ${action}`
  toastFn({
    title,
    description: name ? `"${name}" was ${action.toLowerCase()} successfully.` : undefined,
    variant: 'success',
  })
}

export function toastDeleteSuccess(resourceType: string, name?: string) {
  toastSuccess(resourceType, 'deleted', name)
}

export function toastForceDelete(
  resourceType: string,
  removedFromCount: number,
  removedFromType: string,
  name?: string,
) {
  const label = resourceLabel(resourceType)
  const refLabel = resourceLabel(removedFromType).toLowerCase() + (removedFromCount !== 1 ? 's' : '')
  toastFn({
    title: `${label} deleted`,
    description: `"${name ?? 'Resource'}" deleted. Removed from ${removedFromCount} ${refLabel}.`,
    variant: 'warning',
  })
}

export function toastError(
  resourceType: string,
  action: string,
  error: unknown,
  name?: string,
) {
  const label = resourceLabel(resourceType)
  const fallback = `Failed to ${action.toLowerCase()} ${label.toLowerCase()}${name ? ` "${name}"` : ''}`

  let detail = fallback
  if (isApiError(error)) {
    detail = error.detail || fallback
  } else if (error instanceof Error) {
    detail = `${fallback} — ${error.message}`
  }

  toastFn({
    title: `Failed to ${action.toLowerCase()} ${label.toLowerCase()}`,
    description: detail,
    variant: 'destructive',
  })
}

export function toastDeleteError(resourceType: string, error: unknown, name?: string) {
  toastError(resourceType, 'Delete', error, name)
}
