import { render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { describe, expect, it, vi } from 'vitest'
import { ArcheClient } from '@/api/generated/client'
import { setClient } from '@/api/generated/hooks'
import { AppShell } from '@/components/AppShell'

setClient(new ArcheClient({ baseUrl: 'http://localhost:3000' }))

vi.mock('@/components/GlobalSearch', () => ({
  GlobalSearch: () => null,
}))

function renderShell(route = '/dashboard') {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={[route]}>
        <AppShell />
      </MemoryRouter>
    </QueryClientProvider>,
  )
}

describe('AppShell', () => {
  beforeEach(async () => {
    const { useUi } = await import('@/stores/ui')
    useUi.setState({
      drawerOpen: true,
      panelOpen: false,
      panelDocked: false,
      searchOpen: false,
    })
    const { useTheme } = await import('@/stores/theme')
    useTheme.setState({ mode: 'light' })
  })

  it('renders the header with app name', () => {
    renderShell()

    expect(screen.getByText('Arche Admin')).toBeInTheDocument()
  })

  it('renders the search bar placeholder', () => {
    renderShell()

    expect(screen.getByText('Search...')).toBeInTheDocument()
  })

  it('opens search when search button is clicked', async () => {
    const { useUi } = await import('@/stores/ui')
    useUi.setState({ searchOpen: false })

    renderShell()

    const searchButton = screen.getByText('Search...')
    searchButton.click()

    expect(useUi.getState().searchOpen).toBe(true)
  })

  it('renders the drawer toggle button', () => {
    renderShell()

    expect(screen.getByLabelText('Close navigation')).toBeInTheDocument()
  })

  it('renders the activity panel toggle button', () => {
    renderShell()

    expect(screen.getByLabelText('Open activity panel')).toBeInTheDocument()
  })

  it('renders the dark mode toggle button in light mode', () => {
    renderShell()

    expect(screen.getByLabelText('Switch to dark mode')).toBeInTheDocument()
  })

  it('renders the navigation drawer', () => {
    renderShell()

    expect(screen.getByText('Dashboard')).toBeInTheDocument()
  })

  it('does not render activity panel when closed', () => {
    renderShell()

    expect(screen.queryByText('Activity log will appear here.')).not.toBeInTheDocument()
  })

  it('renders activity panel when open', async () => {
    const { useUi } = await import('@/stores/ui')
    useUi.setState({ panelOpen: true })

    renderShell()

    expect(screen.getByText('Activity log will appear here.')).toBeInTheDocument()
  })

  it('toggles drawer when hamburger is clicked', async () => {
    const { useUi } = await import('@/stores/ui')
    useUi.setState({ drawerOpen: true })

    renderShell()

    const toggleButton = screen.getByLabelText('Close navigation')
    toggleButton.click()

    expect(useUi.getState().drawerOpen).toBe(false)

    toggleButton.click()
    expect(useUi.getState().drawerOpen).toBe(true)
  })

  it('toggles panel when panel button is clicked', async () => {
    const { useUi } = await import('@/stores/ui')
    useUi.setState({ panelOpen: false })

    renderShell()

    const toggleButton = screen.getByLabelText('Open activity panel')
    toggleButton.click()

    expect(useUi.getState().panelOpen).toBe(true)
  })

  it('toggles theme when theme button is clicked', async () => {
    const { useTheme } = await import('@/stores/theme')
    useTheme.setState({ mode: 'light' })

    renderShell()

    expect(screen.getByLabelText('Switch to dark mode')).toBeInTheDocument()

    const themeButton = screen.getByLabelText('Switch to dark mode')
    themeButton.click()

    expect(useTheme.getState().mode).toBe('dark')
  })
})
