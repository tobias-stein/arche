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
      panelOpen: false,
      panelDocked: false,
      searchOpen: false,
    })
    const { useTheme } = await import('@/stores/theme')
    useTheme.setState({ mode: 'light' })
  })

  it('renders sidebar with nav links', () => {
    renderShell()

    expect(screen.getByText('Dashboard')).toBeInTheDocument()
    expect(screen.getByText('Blueprints')).toBeInTheDocument()
  })

  it('renders the search button', () => {
    renderShell()

    expect(screen.getByText('Search')).toBeInTheDocument()
  })

  it('opens search when search button is clicked', async () => {
    const { useUi } = await import('@/stores/ui')
    useUi.setState({ searchOpen: false })

    renderShell()

    const searchButton = screen.getByText('Search')
    searchButton.click()

    expect(useUi.getState().searchOpen).toBe(true)
  })

  it('renders the sidebar toggle button', () => {
    renderShell()

    expect(screen.getByLabelText('Toggle sidebar')).toBeInTheDocument()
  })

  it('renders the activity panel toggle button', () => {
    renderShell()

    expect(screen.getByLabelText('Open activity panel')).toBeInTheDocument()
  })

  it('renders the dark mode toggle button in light mode', () => {
    renderShell()

    expect(screen.getByLabelText('Switch to dark mode')).toBeInTheDocument()
  })

  it('renders sidebar nav links', () => {
    renderShell()

    expect(screen.getByText('Dashboard')).toBeInTheDocument()
    expect(screen.getByText('Blueprints')).toBeInTheDocument()
    expect(screen.getByText('Affixes')).toBeInTheDocument()
    expect(screen.getByText('Global Meta Attributes')).toBeInTheDocument()
    expect(screen.getByText('Audit Log')).toBeInTheDocument()
    expect(screen.getByText('API Keys')).toBeInTheDocument()
  })

  it('does not render activity panel when closed', () => {
    renderShell()

    expect(screen.queryByText('Activity')).not.toBeInTheDocument()
  })

  it('renders activity panel when open', async () => {
    const { useUi } = await import('@/stores/ui')
    useUi.setState({ panelOpen: true })

    renderShell()

    expect(screen.getByText('Activity')).toBeInTheDocument()
    expect(screen.getByText('No recent activity.')).toBeInTheDocument()
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
