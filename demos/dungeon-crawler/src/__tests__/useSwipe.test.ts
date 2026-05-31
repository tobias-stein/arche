import { describe, it, expect, vi } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useSwipe } from '../hooks/useSwipe'

describe('useSwipe', () => {
  it('fires onSwipe on left swipe past threshold', () => {
    const onSwipe = vi.fn()
    const { result } = renderHook(() => useSwipe({ onSwipe, threshold: 80, direction: 'left' }))

    act(() => {
      result.current.onTouchStart({
        touches: [{ clientX: 200, clientY: 100 }],
      } as unknown as React.TouchEvent)
    })

    act(() => {
      result.current.onTouchEnd({
        changedTouches: [{ clientX: 100, clientY: 105 }],
      } as unknown as React.TouchEvent)
    })

    expect(onSwipe).toHaveBeenCalledTimes(1)
  })

  it('fires onSwipe on right swipe past threshold', () => {
    const onSwipe = vi.fn()
    const { result } = renderHook(() => useSwipe({ onSwipe, threshold: 80, direction: 'right' }))

    act(() => {
      result.current.onTouchStart({
        touches: [{ clientX: 100, clientY: 100 }],
      } as unknown as React.TouchEvent)
    })

    act(() => {
      result.current.onTouchEnd({
        changedTouches: [{ clientX: 200, clientY: 105 }],
      } as unknown as React.TouchEvent)
    })

    expect(onSwipe).toHaveBeenCalledTimes(1)
  })

  it('does not fire onSwipe when swipe is below threshold', () => {
    const onSwipe = vi.fn()
    const { result } = renderHook(() => useSwipe({ onSwipe, threshold: 80, direction: 'left' }))

    act(() => {
      result.current.onTouchStart({
        touches: [{ clientX: 150, clientY: 100 }],
      } as unknown as React.TouchEvent)
    })

    act(() => {
      result.current.onTouchEnd({
        changedTouches: [{ clientX: 130, clientY: 102 }],
      } as unknown as React.TouchEvent)
    })

    expect(onSwipe).not.toHaveBeenCalled()
  })

  it('does not fire onSwipe for vertical swipe (same direction threshold)', () => {
    const onSwipe = vi.fn()
    const { result } = renderHook(() => useSwipe({ onSwipe, threshold: 80, direction: 'left' }))

    act(() => {
      result.current.onTouchStart({
        touches: [{ clientX: 200, clientY: 100 }],
      } as unknown as React.TouchEvent)
    })

    act(() => {
      result.current.onTouchEnd({
        changedTouches: [{ clientX: 190, clientY: 300 }],
      } as unknown as React.TouchEvent)
    })

    expect(onSwipe).not.toHaveBeenCalled()
  })
})
