import { create } from 'zustand'

const DOCKED_KEY = 'arche-panel-docked'

function getInitialDocked(): boolean {
  try {
    return localStorage.getItem(DOCKED_KEY) === 'true'
  } catch {
    return false
  }
}

interface UiState {
  panelOpen: boolean
  panelDocked: boolean
  searchOpen: boolean
  selectedClientId: string | null
  togglePanel: () => void
  closePanel: () => void
  setPanelDocked: (docked: boolean) => void
  setSelectedClientId: (id: string | null) => void
  openSearch: () => void
  closeSearch: () => void
}

export const useUi = create<UiState>((set) => ({
  panelOpen: false,
  panelDocked: getInitialDocked(),
  searchOpen: false,
  selectedClientId: null,
  togglePanel: () => set((state) => ({ panelOpen: !state.panelOpen })),
  closePanel: () => set({ panelOpen: false }),
  setPanelDocked: (docked: boolean) => {
    try {
      localStorage.setItem(DOCKED_KEY, String(docked))
    } catch { /* noop */ }
    set({ panelDocked: docked })
  },
  setSelectedClientId: (id: string | null) => set({ selectedClientId: id }),
  openSearch: () => set({ searchOpen: true }),
  closeSearch: () => set({ searchOpen: false }),
}))
