import { describe, expect, it, vi, beforeEach } from 'vitest'
import { ApiError } from '@/api/generated/errors'

const mockSetApiKey = vi.fn()
const mockGetMe = vi.fn()
const mockSetSelectedClientId = vi.fn()

vi.mock('@/api/generated/hooks', () => ({
  getClient: () => ({
    setApiKey: mockSetApiKey,
    getMe: mockGetMe,
  }),
  setClient: vi.fn(),
}))

vi.mock('@/stores/ui', () => ({
  useUi: {
    getState: () => ({
      setSelectedClientId: mockSetSelectedClientId,
    }),
  },
}))

function superKeyResponse() {
  return {
    id: 'key-super-1',
    name: 'Super Admin Key',
    clientId: null,
    permissions: ['admin', 'read', 'write', 'delete', 'generate'],
    isSuper: true,
  }
}

function clientKeyResponse(clientId: string) {
  return {
    id: 'key-client-1',
    name: 'Client Key',
    clientId,
    permissions: ['read', 'write', 'generate'],
    isSuper: false,
  }
}

describe('useAuth store', () => {
  async function getState() {
    const { useAuth } = await import('@/stores/auth')
    return useAuth.getState
  }

  beforeEach(async () => {
    vi.clearAllMocks()
    const get = await getState()
    get().logout()
  })

  it('starts with no api key and not authenticated', async () => {
    const get = await getState()
    expect(get().apiKey).toBeNull()
    expect(get().isAuthenticated).toBe(false)
    expect(get().isSuperAdmin).toBe(false)
    expect(get().keyId).toBeNull()
    expect(get().keyName).toBeNull()
    expect(get().clientId).toBeNull()
    expect(get().permissions).toEqual([])
  })

  it('login with super key sets isSuperAdmin and clears selectedClientId', async () => {
    mockGetMe.mockResolvedValue(superKeyResponse())

    const get = await getState()
    await get().login('super-key')

    expect(get().apiKey).toBe('super-key')
    expect(get().isAuthenticated).toBe(true)
    expect(get().isSuperAdmin).toBe(true)
    expect(get().keyId).toBe('key-super-1')
    expect(get().keyName).toBe('Super Admin Key')
    expect(get().clientId).toBeNull()
    expect(get().permissions).toEqual(['admin', 'read', 'write', 'delete', 'generate'])
    expect(mockSetApiKey).toHaveBeenCalledWith('super-key')
    expect(mockSetSelectedClientId).toHaveBeenCalledWith(null)
  })

  it('login with client-scoped key sets isSuperAdmin false and selects client', async () => {
    mockGetMe.mockResolvedValue(clientKeyResponse('client-abc'))

    const get = await getState()
    await get().login('client-key')

    expect(get().apiKey).toBe('client-key')
    expect(get().isAuthenticated).toBe(true)
    expect(get().isSuperAdmin).toBe(false)
    expect(get().keyId).toBe('key-client-1')
    expect(get().keyName).toBe('Client Key')
    expect(get().clientId).toBe('client-abc')
    expect(get().permissions).toEqual(['read', 'write', 'generate'])
    expect(mockSetApiKey).toHaveBeenCalledWith('client-key')
    expect(mockSetSelectedClientId).toHaveBeenCalledWith('client-abc')
  })

  it('login throws and does not set state on 401', async () => {
    const error = new ApiError({
      type: '/errors/unauthorized',
      title: 'Unauthorized',
      status: 401,
      detail: 'Invalid API key',
    })
    mockGetMe.mockRejectedValue(error)

    const get = await getState()
    await expect(get().login('bad-key')).rejects.toThrow()

    expect(get().apiKey).toBeNull()
    expect(get().isAuthenticated).toBe(false)
    expect(get().isSuperAdmin).toBe(false)
    expect(get().keyId).toBeNull()
    expect(get().keyName).toBeNull()
  })

  it('login throws and does not set state on 403', async () => {
    const error = new ApiError({
      type: '/errors/forbidden',
      title: 'Forbidden',
      status: 403,
      detail: 'Insufficient permissions',
    })
    mockGetMe.mockRejectedValue(error)

    const get = await getState()
    await expect(get().login('forbidden-key')).rejects.toThrow()

    expect(get().apiKey).toBeNull()
    expect(get().isAuthenticated).toBe(false)
    expect(get().isSuperAdmin).toBe(false)
  })

  it('login throws on response lacking both isSuper and clientId', async () => {
    mockGetMe.mockResolvedValue({
      id: 'key-bad',
      name: 'Bad Key',
      clientId: null,
      permissions: [],
      isSuper: false,
    })

    const get = await getState()
    await expect(get().login('bad-key')).rejects.toThrow('missing clientId and isSuper')

    expect(get().apiKey).toBeNull()
    expect(get().isAuthenticated).toBe(false)
    expect(get().isSuperAdmin).toBe(false)
  })

  it('logout clears all auth state', async () => {
    mockGetMe.mockResolvedValue(superKeyResponse())

    const get = await getState()
    await get().login('super-key')

    expect(get().apiKey).toBe('super-key')
    expect(get().isAuthenticated).toBe(true)
    expect(get().isSuperAdmin).toBe(true)

    get().logout()

    expect(get().apiKey).toBeNull()
    expect(get().isAuthenticated).toBe(false)
    expect(get().isSuperAdmin).toBe(false)
    expect(get().keyId).toBeNull()
    expect(get().keyName).toBeNull()
    expect(get().clientId).toBeNull()
    expect(get().permissions).toEqual([])
    expect(mockSetApiKey).toHaveBeenCalledWith('')
  })
})
