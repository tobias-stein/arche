import { describe, expect, it, vi } from 'vitest'
import { ApiError } from '@/api/generated/errors'

const mockSetApiKey = vi.fn()
const mockListClients = vi.fn()

vi.mock('@/api/generated/hooks', () => ({
  getClient: () => ({
    setApiKey: mockSetApiKey,
    listClients: mockListClients,
  }),
  setClient: vi.fn(),
}))

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
  })

  it('login sets apiKey, isAuthenticated, and isSuperAdmin on success', async () => {
    mockListClients.mockResolvedValue({ data: [], nextCursor: undefined })

    const get = await getState()
    await get().login('valid-key')

    expect(get().apiKey).toBe('valid-key')
    expect(get().isAuthenticated).toBe(true)
    expect(get().isSuperAdmin).toBe(true)
    expect(mockSetApiKey).toHaveBeenCalledWith('valid-key')
  })

  it('login throws and does not set state on 401', async () => {
    const error = new ApiError({
      type: '/errors/unauthorized',
      title: 'Unauthorized',
      status: 401,
      detail: 'Invalid API key',
    })
    mockListClients.mockRejectedValue(error)

    const get = await getState()
    await expect(get().login('bad-key')).rejects.toThrow()

    expect(get().apiKey).toBeNull()
    expect(get().isAuthenticated).toBe(false)
    expect(get().isSuperAdmin).toBe(false)
  })

  it('login throws and does not set state on 403', async () => {
    const error = new ApiError({
      type: '/errors/forbidden',
      title: 'Forbidden',
      status: 403,
      detail: 'Insufficient permissions',
    })
    mockListClients.mockRejectedValue(error)

    const get = await getState()
    await expect(get().login('client-key')).rejects.toThrow()

    expect(get().apiKey).toBeNull()
    expect(get().isAuthenticated).toBe(false)
    expect(get().isSuperAdmin).toBe(false)
  })

  it('logout clears api key, auth state, and client', async () => {
    mockListClients.mockResolvedValue({ data: [], nextCursor: undefined })

    const get = await getState()
    await get().login('valid-key')

    expect(get().apiKey).toBe('valid-key')
    expect(get().isAuthenticated).toBe(true)

    get().logout()

    expect(get().apiKey).toBeNull()
    expect(get().isAuthenticated).toBe(false)
    expect(get().isSuperAdmin).toBe(false)
    expect(mockSetApiKey).toHaveBeenCalledWith('')
  })

  it('isAuthenticated reflects apiKey state', async () => {
    const get = await getState()
    expect(get().isAuthenticated).toBe(false)

    mockListClients.mockResolvedValue({ data: [], nextCursor: undefined })
    await get().login('key')

    expect(get().isAuthenticated).toBe(true)
    expect(get().apiKey).toBe('key')

    get().logout()

    expect(get().isAuthenticated).toBe(false)
    expect(get().apiKey).toBeNull()
  })
})
