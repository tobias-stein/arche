import { useCallback, useRef } from 'react'

interface SwipeOptions {
  threshold?: number
  direction?: 'left' | 'right'
  onSwipe: () => void
}

export function useSwipe({ threshold = 80, direction = 'left', onSwipe }: SwipeOptions) {
  const startX = useRef(0)
  const startY = useRef(0)

  const onTouchStart = useCallback((e: React.TouchEvent) => {
    startX.current = e.touches[0].clientX
    startY.current = e.touches[0].clientY
  }, [])

  const onTouchEnd = useCallback((e: React.TouchEvent) => {
    const endX = e.changedTouches[0].clientX
    const endY = e.changedTouches[0].clientY
    const dx = endX - startX.current
    const dy = endY - startY.current
    if (Math.abs(dx) > Math.abs(dy) && Math.abs(dx) >= threshold) {
      if (direction === 'left' && dx < 0) {
        onSwipe()
      } else if (direction === 'right' && dx > 0) {
        onSwipe()
      }
    }
  }, [threshold, direction, onSwipe])

  return { onTouchStart, onTouchEnd }
}
