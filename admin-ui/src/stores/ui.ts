import { create } from 'zustand'

interface UiState {
  drawerOpen: boolean
  panelOpen: boolean
  panelDocked: boolean
  toggleDrawer: () => void
  closeDrawer: () => void
  togglePanel: () => void
  closePanel: () => void
  setPanelDocked: (docked: boolean) => void
}

export const useUi = create<UiState>((set) => ({
  drawerOpen: false,
  panelOpen: false,
  panelDocked: false,
  toggleDrawer: () => set((state) => ({ drawerOpen: !state.drawerOpen })),
  closeDrawer: () => set({ drawerOpen: false }),
  togglePanel: () => set((state) => ({ panelOpen: !state.panelOpen })),
  closePanel: () => set({ panelOpen: false }),
  setPanelDocked: (docked: boolean) => set({ panelDocked: docked }),
}))
