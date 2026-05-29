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
  client_id: string | null
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
  client_id: null,
  permissions: [],
  login: async (key: string) => {
    const client = getClient()
    client.setApiKey(key)
    const me = await client.getMe()
    if (!me.is_super && !me.client_id) {
      throw new Error('Invalid API key response: missing clientId and is_super')
    }
    const selectedClientId = me.is_super ? null : me.client_id
    useUi.getState().setSelectedClientId(selectedClientId)
    set({
      apiKey: key,
      isAuthenticated: true,
      isSuperAdmin: me.is_super,
      keyId: me.id,
      keyName: me.name,
      client_id: selectedClientId,
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
      client_id: null,
      permissions: [],
    })
    window.location.hash = '#/login'
  },
}))
