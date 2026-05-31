import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useLongPress } from '../hooks/useLongPress'

describe('useLongPress', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('fires onLongPress after duration', () => {
    const onLongPress = vi.fn()
    const { result } = renderHook(() => useLongPress({ onLongPress, duration: 500 }))

    act(() => {
      result.current.onMouseDown({ preventDefault: vi.fn() } as unknown as React.MouseEvent)
    })

    expect(onLongPress).not.toHaveBeenCalled()

    act(() => {
      vi.advanceTimersByTime(500)
    })

    expect(onLongPress).toHaveBeenCalledTimes(1)
  })

  it('cancels on mouse up before duration', () => {
    const onLongPress = vi.fn()
    const { result } = renderHook(() => useLongPress({ onLongPress, duration: 500 }))

    act(() => {
      result.current.onMouseDown({ preventDefault: vi.fn() } as unknown as React.MouseEvent)
    })

    act(() => {
      result.current.onMouseUp()
    })

    act(() => {
      vi.advanceTimersByTime(500)
    })

    expect(onLongPress).not.toHaveBeenCalled()
  })

  it('cancels on mouse leave', () => {
    const onLongPress = vi.fn()
    const { result } = renderHook(() => useLongPress({ onLongPress, duration: 500 }))

    act(() => {
      result.current.onMouseDown({ preventDefault: vi.fn() } as unknown as React.MouseEvent)
    })

    act(() => {
      result.current.onMouseLeave()
    })

    act(() => {
      vi.advanceTimersByTime(500)
    })

    expect(onLongPress).not.toHaveBeenCalled()
  })

  it('cancels on touch end', () => {
    const onLongPress = vi.fn()
    const { result } = renderHook(() => useLongPress({ onLongPress, duration: 500 }))

    act(() => {
      result.current.onTouchStart({ preventDefault: vi.fn() } as unknown as React.TouchEvent)
    })

    act(() => {
      result.current.onTouchEnd()
    })

    act(() => {
      vi.advanceTimersByTime(500)
    })

    expect(onLongPress).not.toHaveBeenCalled()
  })

  it('cancels on touch move', () => {
    const onLongPress = vi.fn()
    const { result } = renderHook(() => useLongPress({ onLongPress, duration: 500 }))

    act(() => {
      result.current.onTouchStart({ preventDefault: vi.fn() } as unknown as React.TouchEvent)
    })

    act(() => {
      result.current.onTouchMove()
    })

    act(() => {
      vi.advanceTimersByTime(500)
    })

    expect(onLongPress).not.toHaveBeenCalled()
  })
})
