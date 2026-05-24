import { create } from 'zustand'

const STORAGE_KEY = 'arche-theme'

type Mode = 'light' | 'dark'

function getInitialMode(): Mode {
  const stored = localStorage.getItem(STORAGE_KEY)
  if (stored === 'dark' || stored === 'light') return stored
  if (window.matchMedia('(prefers-color-scheme: dark)').matches) return 'dark'
  return 'light'
}

function applyClass(mode: Mode) {
  document.documentElement.classList.toggle('dark', mode === 'dark')
}

function persist(mode: Mode) {
  localStorage.setItem(STORAGE_KEY, mode)
}

const initial = getInitialMode()
applyClass(initial)

interface ThemeState {
  mode: Mode
  toggle: () => void
  setMode: (mode: Mode) => void
}

export const useTheme = create<ThemeState>((set) => ({
  mode: initial,
  toggle: () =>
    set((state) => {
      const next: Mode = state.mode === 'light' ? 'dark' : 'light'
      persist(next)
      applyClass(next)
      return { mode: next }
    }),
  setMode: (mode: Mode) => {
    persist(mode)
    applyClass(mode)
    set({ mode })
  },
}))
