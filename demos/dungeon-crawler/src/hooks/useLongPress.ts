import { useCallback, useRef } from 'react'

interface LongPressOptions {
  duration?: number
  onLongPress: () => void
}

export function useLongPress({ duration = 500, onLongPress }: LongPressOptions) {
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const triggeredRef = useRef(false)

  const start = useCallback((e: React.TouchEvent | React.MouseEvent) => {
    e.preventDefault()
    triggeredRef.current = false
    timerRef.current = setTimeout(() => {
      triggeredRef.current = true
      onLongPress()
    }, duration)
  }, [duration, onLongPress])

  const cancel = useCallback(() => {
    if (timerRef.current) {
      clearTimeout(timerRef.current)
      timerRef.current = null
    }
  }, [])

  const isTriggered = useCallback(() => triggeredRef.current, [])

  return {
    onTouchStart: start,
    onTouchEnd: cancel,
    onTouchMove: cancel,
    onMouseDown: start,
    onMouseUp: cancel,
    onMouseLeave: cancel,
    isTriggered,
  }
}
