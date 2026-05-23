import { create } from 'zustand'

interface UiState {
  drawerOpen: boolean
  panelOpen: boolean
  panelDocked: boolean
  toggleDrawer: () => void
  togglePanel: () => void
  setPanelDocked: (docked: boolean) => void
}

export const useUi = create<UiState>((set) => ({
  drawerOpen: false,
  panelOpen: false,
  panelDocked: false,
  toggleDrawer: () => set((state) => ({ drawerOpen: !state.drawerOpen })),
  togglePanel: () => set((state) => ({ panelOpen: !state.panelOpen })),
  setPanelDocked: (docked: boolean) => set({ panelDocked: docked }),
}))
