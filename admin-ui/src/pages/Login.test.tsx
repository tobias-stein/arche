import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { describe, expect, it, vi } from 'vitest'
import { ApiError } from '@/api/generated/errors'
import Login from '@/pages/Login'

const mockSetApiKey = vi.fn()
const mockListClients = vi.fn()

vi.mock('@/api/generated/hooks', () => ({
  getClient: () => ({
    setApiKey: mockSetApiKey,
    listClients: mockListClients,
  }),
  setClient: vi.fn(),
}))

function renderLogin() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={['/login']}>
        <Routes>
          <Route path="/login" element={<Login />} />
          <Route path="/dashboard" element={<div>Dashboard</div>} />
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>,
  )
}

describe('Login page', () => {
  beforeEach(async () => {
    vi.clearAllMocks()
    const { useAuth } = await import('@/stores/auth')
    useAuth.setState({ apiKey: null, isAuthenticated: false, isSuperAdmin: false })
  })

  it('renders the login form', () => {
    renderLogin()

    expect(screen.getByText('Arche Admin')).toBeInTheDocument()
    expect(screen.getByPlaceholderText('API Key')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /sign in/i })).toBeInTheDocument()
  })

  it('has a disabled submit button when input is empty', () => {
    renderLogin()

    const button = screen.getByRole('button')
    expect(button).toBeDisabled()
  })

  it('enables submit button when API key is entered', async () => {
    renderLogin()

    const input = screen.getByPlaceholderText('API Key')
    await userEvent.type(input, 'test-key')

    const button = screen.getByRole('button')
    expect(button).not.toBeDisabled()
  })

  it('shows error message on invalid API key', async () => {
    const error = new ApiError({
      type: '/errors/unauthorized',
      title: 'Unauthorized',
      status: 401,
      detail: 'Invalid API key',
    })
    mockListClients.mockRejectedValue(error)

    renderLogin()

    const input = screen.getByPlaceholderText('API Key')
    await userEvent.type(input, 'bad-key')

    const button = screen.getByRole('button')
    await userEvent.click(button)

    await waitFor(() => {
      expect(screen.getByRole('alert')).toHaveTextContent('Invalid API key')
    })
  })

  it('shows connection error on network failure', async () => {
    mockListClients.mockRejectedValue(new Error('Failed to fetch'))

    renderLogin()

    const input = screen.getByPlaceholderText('API Key')
    await userEvent.type(input, 'test-key')

    const button = screen.getByRole('button')
    await userEvent.click(button)

    await waitFor(() => {
      expect(screen.getByRole('alert')).toHaveTextContent('Failed to fetch')
    })
  })

  it('redirects to dashboard on successful login', async () => {
    mockListClients.mockResolvedValue({ data: [], nextCursor: undefined })

    renderLogin()

    const input = screen.getByPlaceholderText('API Key')
    await userEvent.type(input, 'valid-key')

    const button = screen.getByRole('button')
    await userEvent.click(button)

    await waitFor(() => {
      expect(screen.getByText('Dashboard')).toBeInTheDocument()
    })
  })
})
