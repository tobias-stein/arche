import { create } from 'zustand'
import { getClient } from '@/api/generated/hooks'
import type { Permission } from '@/api/generated/types'
import { useUi } from '@/stores/ui'

interface AuthState {
  apiKey: string | null
  isAuthenticated: boolean
  isSuperAdmin: boolean
  keyId: string | null
  keyName: string | null
  clientId: string | null
  permissions: Permission[]
  login: (key: string) => Promise<void>
  logout: () => void
}

export const useAuth = create<AuthState>((set) => ({
  apiKey: null,
  isAuthenticated: false,
  isSuperAdmin: false,
  keyId: null,
  keyName: null,
  clientId: null,
  permissions: [],
  login: async (key: string) => {
    const client = getClient()
    client.setApiKey(key)
    const me = await client.getMe()
    if (me.isSuper) {
      useUi.getState().setSelectedClientId(null)
      set({
        apiKey: key,
        isAuthenticated: true,
        isSuperAdmin: true,
        keyId: me.id,
        keyName: me.name,
        clientId: null,
        permissions: me.permissions,
      })
    } else if (me.clientId) {
      useUi.getState().setSelectedClientId(me.clientId)
      set({
        apiKey: key,
        isAuthenticated: true,
        isSuperAdmin: false,
        keyId: me.id,
        keyName: me.name,
        clientId: me.clientId,
        permissions: me.permissions,
      })
    } else {
      throw new Error('Invalid API key response: missing clientId and isSuper')
    }
  },
  logout: () => {
    getClient().setApiKey('')
    set({
      apiKey: null,
      isAuthenticated: false,
      isSuperAdmin: false,
      keyId: null,
      keyName: null,
      clientId: null,
      permissions: [],
    })
    window.location.hash = '#/login'
  },
}))
