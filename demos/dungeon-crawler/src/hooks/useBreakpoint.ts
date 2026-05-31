import { useState, useEffect } from 'react'

export type Breakpoint = 'mobile' | 'tablet' | 'desktop'

const MOBILE_QUERY = '(max-width: 639px)'
const TABLET_QUERY = '(min-width: 640px) and (max-width: 1024px)'
const DESKTOP_QUERY = '(min-width: 1025px)'

function getBreakpoint(): Breakpoint {
  if (window.matchMedia(DESKTOP_QUERY).matches) return 'desktop'
  if (window.matchMedia(TABLET_QUERY).matches) return 'tablet'
  return 'mobile'
}

export function useBreakpoint(): Breakpoint {
  const [bp, setBp] = useState<Breakpoint>(getBreakpoint)

  useEffect(() => {
    function handleChange() {
      setBp(getBreakpoint())
    }

    const mqls = [
      window.matchMedia(MOBILE_QUERY),
      window.matchMedia(TABLET_QUERY),
      window.matchMedia(DESKTOP_QUERY),
    ]

    mqls.forEach(mql => mql.addEventListener('change', handleChange))
    return () => mqls.forEach(mql => mql.removeEventListener('change', handleChange))
  }, [])

  return bp
}
