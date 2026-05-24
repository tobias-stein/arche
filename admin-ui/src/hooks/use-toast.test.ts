import { act, renderHook } from '@testing-library/react'
import { beforeEach, describe, expect, it } from 'vitest'

const setup = async () => {
  const mod = await import('@/hooks/use-toast')
  const { useToast, toast } = mod
  const { result } = renderHook(() => useToast())
  return { result, toast }
}

describe('useToast', () => {
  beforeEach(() => {
    vi.resetModules()
  })

  it('adds a toast to the state', async () => {
    const { result, toast } = await setup()

    act(() => {
      toast({ title: 'Test', description: 'Description' })
    })

    expect(result.current.toasts).toHaveLength(1)
    expect(result.current.toasts[0].title).toBe('Test')
    expect(result.current.toasts[0].description).toBe('Description')
  })

  it('supports multiple stacked toasts', async () => {
    const { result, toast } = await setup()

    act(() => {
      toast({ title: 'Toast 1' })
      toast({ title: 'Toast 2' })
      toast({ title: 'Toast 3' })
    })

    expect(result.current.toasts).toHaveLength(3)
    expect(result.current.toasts[0].title).toBe('Toast 3')
    expect(result.current.toasts[1].title).toBe('Toast 2')
    expect(result.current.toasts[2].title).toBe('Toast 1')
  })

  it('supports success, destructive, and warning variants', async () => {
    const { result, toast } = await setup()

    act(() => {
      toast({ title: 'Destructive', variant: 'destructive' })
      toast({ title: 'Success', variant: 'success' })
      toast({ title: 'Warning', variant: 'warning' })
    })

    expect(result.current.toasts).toHaveLength(3)
    expect(result.current.toasts[0].variant).toBe('warning')
    expect(result.current.toasts[1].variant).toBe('success')
    expect(result.current.toasts[2].variant).toBe('destructive')
  })

  it('dismisses a specific toast by id', async () => {
    const { result, toast } = await setup()

    let toastId: string
    act(() => {
      const t = toast({ title: 'Keep' })
      toastId = t.id
      toast({ title: 'Dismiss Me' })
    })

    expect(result.current.toasts).toHaveLength(2)

    act(() => {
      result.current.dismiss(toastId!)
    })

    expect(result.current.toasts).toHaveLength(2)
    expect(result.current.toasts[0].title).toBe('Dismiss Me')
    expect(result.current.toasts[1].title).toBe('Keep')
  })

  it('limits toasts to TOAST_LIMIT of 10', async () => {
    const { result, toast } = await setup()

    act(() => {
      for (let i = 0; i < 15; i++) {
        toast({ title: `Toast ${i}` })
      }
    })

    expect(result.current.toasts).toHaveLength(10)
  })
})
