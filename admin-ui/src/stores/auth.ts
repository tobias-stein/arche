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
    if (!me.isSuper && !me.clientId) {
      throw new Error('Invalid API key response: missing clientId and isSuper')
    }
    const selectedClientId = me.isSuper ? null : me.clientId
    useUi.getState().setSelectedClientId(selectedClientId)
    set({
      apiKey: key,
      isAuthenticated: true,
      isSuperAdmin: me.isSuper,
      keyId: me.id,
      keyName: me.name,
      clientId: selectedClientId,
      permissions: me.permissions,
    })
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
