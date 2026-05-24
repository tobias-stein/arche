import { describe, expect, it, vi } from 'vitest'

import { ApiError } from '@/api/generated/errors'

vi.mock('@/hooks/use-toast', () => ({
  toast: vi.fn(),
}))

import { toast as mockedToast } from '@/hooks/use-toast'
import {
  toastDeleteError,
  toastDeleteSuccess,
  toastForceDelete,
} from '@/lib/toast-helpers'

describe('toast-helpers', () => {
  it('toastDeleteSuccess shows success toast', () => {
    toastDeleteSuccess('blueprint', 'SwordOfPower')

    expect(mockedToast).toHaveBeenCalledWith({
      title: 'Blueprint deleted',
      description: '"SwordOfPower" was deleted successfully.',
      variant: 'success',
    })
  })

  it('toastDeleteSuccess works without name', () => {
    toastDeleteSuccess('client')

    expect(mockedToast).toHaveBeenCalledWith({
      title: 'Client deleted',
      description: undefined,
      variant: 'success',
    })
  })

  it('toastDeleteError shows error toast with ApiError detail', () => {
    const apiError = new ApiError({
      type: '/errors/delete-referenced-resource',
      title: 'Cannot delete',
      status: 409,
      detail: 'Failed to delete affix — it is referenced by 2 blueprints',
    })

    toastDeleteError('affix', apiError, 'Flame')

    expect(mockedToast).toHaveBeenCalledWith({
      title: 'Failed to delete affix',
      description: 'Failed to delete affix — it is referenced by 2 blueprints',
      variant: 'destructive',
    })
  })

  it('toastDeleteError shows fallback message for non-ApiError', () => {
    toastDeleteError('blueprint', new Error('Network error'), 'Sword')

    expect(mockedToast).toHaveBeenCalledWith({
      title: 'Failed to delete blueprint',
      description: 'Failed to delete blueprint "Sword" — Network error',
      variant: 'destructive',
    })
  })

  it('toastDeleteError shows fallback for unknown errors', () => {
    toastDeleteError('global meta attribute', 'some string error')

    expect(mockedToast).toHaveBeenCalledWith({
      title: 'Failed to delete global meta attribute',
      description: 'Failed to delete global meta attribute',
      variant: 'destructive',
    })
  })

  it('toastForceDelete shows warning toast with cascade summary', () => {
    toastForceDelete('affix', 3, 'blueprint', 'Flame')

    expect(mockedToast).toHaveBeenCalledWith({
      title: 'Affix deleted',
      description: '"Flame" deleted. Removed from 3 blueprints.',
      variant: 'warning',
    })
  })

  it('toastForceDelete uses singular form for 1 reference', () => {
    toastForceDelete('blueprint', 1, 'affix')

    expect(mockedToast).toHaveBeenCalledWith({
      title: 'Blueprint deleted',
      description: '"Resource" deleted. Removed from 1 affix.',
      variant: 'warning',
    })
  })
})
