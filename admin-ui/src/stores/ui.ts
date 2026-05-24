import { create } from 'zustand'

interface UiState {
  drawerOpen: boolean
  panelOpen: boolean
  panelDocked: boolean
  searchOpen: boolean
  toggleDrawer: () => void
  closeDrawer: () => void
  togglePanel: () => void
  closePanel: () => void
  setPanelDocked: (docked: boolean) => void
  openSearch: () => void
  closeSearch: () => void
}

export const useUi = create<UiState>((set) => ({
  drawerOpen: false,
  panelOpen: false,
  panelDocked: false,
  searchOpen: false,
  toggleDrawer: () => set((state) => ({ drawerOpen: !state.drawerOpen })),
  closeDrawer: () => set({ drawerOpen: false }),
  togglePanel: () => set((state) => ({ panelOpen: !state.panelOpen })),
  closePanel: () => set({ panelOpen: false }),
  setPanelDocked: (docked: boolean) => set({ panelDocked: docked }),
  openSearch: () => set({ searchOpen: true }),
  closeSearch: () => set({ searchOpen: false }),
}))
