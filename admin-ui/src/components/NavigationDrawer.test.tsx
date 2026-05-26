import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { describe, expect, it } from 'vitest'
import { ArcheClient } from '@/api/generated/client'
import { setClient } from '@/api/generated/hooks'
import { NavigationDrawer } from '@/components/NavigationDrawer'

setClient(new ArcheClient({ baseUrl: 'http://localhost:3000' }))

function renderDrawer(route = '/dashboard') {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={[route]}>
        <NavigationDrawer />
      </MemoryRouter>
    </QueryClientProvider>,
  )
}

describe('NavigationDrawer', () => {
  beforeEach(async () => {
    const { useUi } = await import('@/stores/ui')
    useUi.setState({ drawerOpen: true })
  })

  it('renders all nav links when drawer is open', () => {
    renderDrawer()

    expect(screen.getByText('Dashboard')).toBeInTheDocument()
    expect(screen.getByText('Blueprints')).toBeInTheDocument()
    expect(screen.getByText('Affixes')).toBeInTheDocument()
    expect(screen.getByText('Global Meta Attributes')).toBeInTheDocument()
    expect(screen.getByText('Audit Log')).toBeInTheDocument()
    expect(screen.getByText('Import')).toBeInTheDocument()
    expect(screen.getByText('Export')).toBeInTheDocument()
    expect(screen.getByText('API Keys')).toBeInTheDocument()
  })

  it('shows Login link when not authenticated', () => {
    renderDrawer()
    expect(screen.getByText('Login')).toBeInTheDocument()
    expect(screen.queryByText('Logout')).not.toBeInTheDocument()
  })

  it('shows Logout link when authenticated', async () => {
    const { useAuth } = await import('@/stores/auth')
    useAuth.setState({ apiKey: 'test-key', isAuthenticated: true })

    renderDrawer()
    expect(screen.getByText('Logout')).toBeInTheDocument()
    expect(screen.queryByText('Login')).not.toBeInTheDocument()

    useAuth.setState({ apiKey: null, isAuthenticated: false })
  })

  it('highlights active nav link', () => {
    renderDrawer('/blueprints')

    const links = screen.getAllByRole('link')
    const blueprintLink = links.find((l) => l.textContent?.includes('Blueprints'))
    expect(blueprintLink?.className).toContain('bg-accent')
  })

  it('shows dark mode toggle in bottom section', () => {
    renderDrawer()
    expect(screen.getByText('Dark mode')).toBeInTheDocument()
  })

  it('closes drawer on backdrop click (mobile)', async () => {
    const { useUi } = await import('@/stores/ui')
    useUi.setState({ drawerOpen: true, panelOpen: false, panelDocked: false })

    renderDrawer()

    const backdrop = document.querySelector('.fixed.inset-0.z-30')
    expect(backdrop).toBeInTheDocument()
    fireEvent.click(backdrop!)

    expect(useUi.getState().drawerOpen).toBe(false)
  })

  it('has correct nav links with href attributes', () => {
    renderDrawer()

    const links = {
      dashboard: screen.getByText('Dashboard').closest('a'),
      blueprints: screen.getByText('Blueprints').closest('a'),
      affixes: screen.getByText('Affixes').closest('a'),
      gma: screen.getByText('Global Meta Attributes').closest('a'),
      audit: screen.getByText('Audit Log').closest('a'),
      import_: screen.getByText('Import').closest('a'),
      export_: screen.getByText('Export').closest('a'),
      apikeys: screen.getByText('API Keys').closest('a'),
    }

    expect(links.dashboard?.getAttribute('href')).toBe('/dashboard')
    expect(links.blueprints?.getAttribute('href')).toBe('/blueprints')
    expect(links.affixes?.getAttribute('href')).toBe('/affixes')
    expect(links.gma?.getAttribute('href')).toBe('/global-meta-attributes')
    expect(links.audit?.getAttribute('href')).toBe('/audit-log')
    expect(links.import_?.getAttribute('href')).toBe('/import')
    expect(links.export_?.getAttribute('href')).toBe('/export')
    expect(links.apikeys?.getAttribute('href')).toBe('/clients')
  })

  it('does not render client switcher when no clients available', async () => {
    renderDrawer()

    await waitFor(() => {
      expect(screen.queryByText('Loading clients...')).not.toBeInTheDocument()
    })

    expect(screen.queryByText('All clients')).not.toBeInTheDocument()
  })

  it('renders aside with desktop stacking context', () => {
    renderDrawer()

    const aside = screen.getByRole('complementary')
    expect(aside.className).toContain('md:relative')
    expect(aside.className).toMatch(/md:z-\d+/)
  })
})
