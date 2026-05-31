import { describe, it, expect, vi, afterEach } from 'vitest'
import { renderHook } from '@testing-library/react'
import { useBreakpoint } from '../hooks/useBreakpoint'

afterEach(() => {
  vi.restoreAllMocks()
})

function setViewport(width: number) {
  const queries: [string, boolean][] = [
    ['(max-width: 639px)', width <= 639],
    ['(min-width: 640px) and (max-width: 1024px)', width >= 640 && width <= 1024],
    ['(min-width: 1025px)', width >= 1025],
  ]
  const matchMediaMock = (query: string): MediaQueryList => {
    const match = queries.find(([q]) => q === query)
    return {
      matches: match ? match[1] : false,
      media: query,
      addEventListener: (_: string, cb: () => void) => {},
      removeEventListener: (_: string, cb: () => void) => {},
    } as MediaQueryList
  }
  vi.spyOn(window, 'matchMedia').mockImplementation(matchMediaMock)
}

describe('useBreakpoint', () => {
  it('returns mobile for viewport < 640px', () => {
    setViewport(375)
    const { result } = renderHook(() => useBreakpoint())
    expect(result.current).toBe('mobile')
  })

  it('returns tablet for viewport 640-1024px', () => {
    setViewport(768)
    const { result } = renderHook(() => useBreakpoint())
    expect(result.current).toBe('tablet')
  })

  it('returns desktop for viewport > 1024px', () => {
    setViewport(1440)
    const { result } = renderHook(() => useBreakpoint())
    expect(result.current).toBe('desktop')
  })

  it('returns mobile at boundary 639px', () => {
    setViewport(639)
    const { result } = renderHook(() => useBreakpoint())
    expect(result.current).toBe('mobile')
  })

  it('returns tablet at boundary 640px', () => {
    setViewport(640)
    const { result } = renderHook(() => useBreakpoint())
    expect(result.current).toBe('tablet')
  })

  it('returns tablet at boundary 1024px', () => {
    setViewport(1024)
    const { result } = renderHook(() => useBreakpoint())
    expect(result.current).toBe('tablet')
  })

  it('returns desktop at boundary 1025px', () => {
    setViewport(1025)
    const { result } = renderHook(() => useBreakpoint())
    expect(result.current).toBe('desktop')
  })
})
