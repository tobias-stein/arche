import { create } from 'zustand'

interface AuthState {
  apiKey: string | null
  isAuthenticated: boolean
  login: (key: string) => void
  logout: () => void
}

export const useAuth = create<AuthState>((set) => ({
  apiKey: null,
  isAuthenticated: false,
  login: (key: string) => set({ apiKey: key, isAuthenticated: true }),
  logout: () => set({ apiKey: null, isAuthenticated: false }),
}))
