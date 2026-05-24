import { create } from 'zustand'
import { getClient } from '@/api/generated/hooks'

interface AuthState {
  apiKey: string | null
  isAuthenticated: boolean
  isSuperAdmin: boolean
  login: (key: string) => Promise<void>
  logout: () => void
}

export const useAuth = create<AuthState>((set) => ({
  apiKey: null,
  isAuthenticated: false,
  isSuperAdmin: false,
  login: async (key: string) => {
    const client = getClient()
    client.setApiKey(key)
    await client.listClients()
    set({ apiKey: key, isAuthenticated: true, isSuperAdmin: true })
  },
  logout: () => {
    getClient().setApiKey('')
    set({ apiKey: null, isAuthenticated: false, isSuperAdmin: false })
    window.location.hash = '#/login'
  },
}))
