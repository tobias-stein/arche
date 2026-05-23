import { create } from 'zustand'

type Theme = 'light' | 'dark'

interface ThemeState {
  theme: Theme
  toggle: () => void
  setTheme: (theme: Theme) => void
}

export const useTheme = create<ThemeState>((set) => ({
  theme: 'dark',
  toggle: () =>
    set((state) => {
      const next = state.theme === 'light' ? 'dark' : 'light'
      document.documentElement.classList.toggle('dark', next === 'dark')
      return { theme: next }
    }),
  setTheme: (theme: Theme) => {
    document.documentElement.classList.toggle('dark', theme === 'dark')
    set({ theme })
  },
}))
