import { create } from 'zustand'

const STORAGE_KEY = 'arche-theme'

type Mode = 'light' | 'dark'

function getInitialMode(): Mode {
  const stored = localStorage.getItem(STORAGE_KEY)
  if (stored === 'dark' || stored === 'light') return stored
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

function applyMode(mode: Mode) {
  document.documentElement.classList.toggle('dark', mode === 'dark')
  localStorage.setItem(STORAGE_KEY, mode)
}

const initial = getInitialMode()
applyMode(initial)

interface ThemeState {
  mode: Mode
  toggle: () => void
  setMode: (mode: Mode) => void
}

export const useTheme = create<ThemeState>((set) => ({
  mode: initial,
  toggle: () =>
    set((state) => {
      const next = state.mode === 'light' ? 'dark' : 'light'
      applyMode(next)
      return { mode: next }
    }),
  setMode: (mode: Mode) => {
    applyMode(mode)
    set({ mode })
  },
}))
