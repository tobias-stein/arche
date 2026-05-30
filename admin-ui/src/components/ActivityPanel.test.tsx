import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { describe, expect, it, vi } from 'vitest'
import { ActivityPanel } from '@/components/ActivityPanel'

const mockNavigate = vi.fn()

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom')
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  }
})

vi.mock('@/api/generated/hooks', async () => {
  const actual = await vi.importActual('@/api/generated/hooks')
  return {
    ...actual,
    useAuditLog: vi.fn(),
  }
})

function renderPanel() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={['/dashboard']}>
        <ActivityPanel />
      </MemoryRouter>
    </QueryClientProvider>,
  )
}

async function setStoreState(overrides: Record<string, unknown>) {
  const { useUi } = await import('@/stores/ui')
  useUi.setState({ ...useUi.getState(), ...overrides })
}

const mockEntry = {
  id: 'entry-1',
  timestamp: new Date(Date.now() - 120000).toISOString(),
  actor_key_id: 'key-1',
  actor_key_name: 'admin-key',
  client_id: 'client-1',
  resource_type: 'blueprint',
  resource_id: 'bp-abc',
  action: 'created' as const,
  before: null,
  after: null,
}

const mockEntries = [
  mockEntry,
  {
    id: 'entry-2',
    timestamp: new Date(Date.now() - 300000).toISOString(),
    actor_key_id: 'key-2',
    actor_key_name: 'editor-key',
    client_id: 'client-1',
    resource_type: 'affix',
    resource_id: 'aff-xyz',
    action: 'updated' as const,
    before: null,
    after: null,
  },
  {
    id: 'entry-3',
    timestamp: new Date(Date.now() - 3600000).toISOString(),
    actor_key_id: 'key-1',
    actor_key_name: 'admin-key',
    client_id: 'client-1',
    resource_type: 'blueprint',
    resource_id: 'bp-def',
    action: 'deleted' as const,
    before: null,
    after: null,
  },
  {
    id: 'entry-4',
    timestamp: new Date(Date.now() - 86400000 * 2).toISOString(),
    actor_key_id: 'key-1',
    actor_key_name: 'admin-key',
    client_id: null,
    resource_type: 'global_meta_attribute',
    resource_id: 'gma-1',
    action: 'adjusted' as const,
    before: null,
    after: null,
  },
]

function makeQueryResult(data: typeof mockEntries) {
  return {
    data: { data, total: data.length },
    isLoading: false,
    isError: false,
    error: null,
    dataUpdatedAt: Date.now(),
    isPending: false,
    isSuccess: true,
    status: 'success' as const,
    isLoadingError: false,
    isRefetchError: false,
    isFetching: false,
    isPaused: false,
    isRefetching: false,
    isStale: true,
    isFetched: true,
    isFetchedAfterMount: true,
    isPlaceholderData: false,
    isInitialLoading: false,
    isEnabled: true,
    fetchStatus: 'idle' as const,
    refetch: vi.fn(),
    promise: Promise.resolve({ data, total: data.length }),
    failureCount: 0,
    failureReason: null,
    errorUpdateCount: 0,
    errorUpdatedAt: 0,
  } as ReturnType<typeof import('@/api/generated/hooks').useAuditLog>
}

describe('ActivityPanel', () => {
  beforeEach(async () => {
    vi.clearAllMocks()
    await setStoreState({ panelOpen: true, panelDocked: false, selectedClientId: null })
    const { useAuditLog } = await import('@/api/generated/hooks')
    vi.mocked(useAuditLog).mockReturnValue(makeQueryResult(mockEntries))
  })

  it('renders the activity panel header', () => {
    renderPanel()
    expect(screen.getByText('Activity')).toBeInTheDocument()
  })

  it('renders audit log entries', () => {
    renderPanel()
    expect(screen.getAllByText('admin-key')).toHaveLength(3)
    expect(screen.getByText('editor-key')).toBeInTheDocument()
  })

  it('renders relative timestamps', () => {
    renderPanel()
    expect(screen.getByText('2 min ago')).toBeInTheDocument()
    expect(screen.getByText('5 min ago')).toBeInTheDocument()
    expect(screen.getByText('2 days ago')).toBeInTheDocument()
  })

  it('renders action badges with correct text', () => {
    renderPanel()
    expect(screen.getByText('created')).toBeInTheDocument()
    expect(screen.getByText('updated')).toBeInTheDocument()
    expect(screen.getByText('deleted')).toBeInTheDocument()
    expect(screen.getByText('adjusted')).toBeInTheDocument()
  })

  it('renders resource type and id for each entry', () => {
    renderPanel()
    expect(screen.getAllByText('blueprint')).toHaveLength(2)
    expect(screen.getByText('affix')).toBeInTheDocument()
    expect(screen.getByText('global_meta_attribute')).toBeInTheDocument()
    expect(screen.getByText('bp-abc')).toBeInTheDocument()
    expect(screen.getByText('aff-xyz')).toBeInTheDocument()
  })

  it('has a close button', () => {
    renderPanel()
    expect(screen.getByLabelText('Close activity panel')).toBeInTheDocument()
  })

  it('closes panel on close button click', async () => {
    const { useUi } = await import('@/stores/ui')
    await setStoreState({ panelOpen: true })

    renderPanel()

    fireEvent.click(screen.getByLabelText('Close activity panel'))
    expect(useUi.getState().panelOpen).toBe(false)
  })

  it('closes panel on outside click when floating', async () => {
    const { useUi } = await import('@/stores/ui')
    await setStoreState({ panelOpen: true, panelDocked: false })

    renderPanel()

    fireEvent.mouseDown(document.body)
    expect(useUi.getState().panelOpen).toBe(false)
  })

  it('does not close when clicking inside the panel', async () => {
    const { useUi } = await import('@/stores/ui')
    await setStoreState({ panelOpen: true, panelDocked: false })

    renderPanel()

    fireEvent.mouseDown(screen.getByText('Activity'))
    expect(useUi.getState().panelOpen).toBe(true)
  })

  it('does not close on outside click when docked', async () => {
    const { useUi } = await import('@/stores/ui')
    await setStoreState({ panelOpen: true, panelDocked: true })

    renderPanel()

    fireEvent.mouseDown(document.body)
    expect(useUi.getState().panelOpen).toBe(true)
  })

  it('does not render pin button when floating', () => {
    renderPanel()
    expect(screen.queryByLabelText('Pin panel')).not.toBeInTheDocument()
  })

  it('has a "View all" link', () => {
    renderPanel()
    expect(screen.getByText('View all')).toBeInTheDocument()
  })

  it('navigates to audit log on "View all" click', () => {
    renderPanel()
    fireEvent.click(screen.getByText('View all'))
    expect(mockNavigate).toHaveBeenCalledWith('/audit-log')
  })

  it('navigates to blueprint detail on blueprint entry click', () => {
    renderPanel()

    const entries = screen.getAllByText('blueprint')
    fireEvent.click(entries[0].closest('button')!)
    expect(mockNavigate).toHaveBeenCalledWith('/blueprints/bp-abc')
  })

  it('navigates to affix detail on affix entry click', () => {
    renderPanel()

    fireEvent.click(screen.getByText('aff-xyz').closest('button')!)
    expect(mockNavigate).toHaveBeenCalledWith('/affixes/aff-xyz')
  })

  it('navigates to global meta attribute detail on GMA entry click', () => {
    renderPanel()

    fireEvent.click(screen.getByText('gma-1').closest('button')!)
    expect(mockNavigate).toHaveBeenCalledWith('/global-meta-attributes/gma-1')
  })

  it('shows empty state when no entries', async () => {
    const { useAuditLog } = await import('@/api/generated/hooks')
    vi.mocked(useAuditLog).mockReturnValue(makeQueryResult([]))

    renderPanel()
    expect(screen.getByText('No recent activity.')).toBeInTheDocument()
  })

  it('filters by selectedClientId when set', async () => {
    const { useAuditLog } = await import('@/api/generated/hooks')
    await setStoreState({ selectedClientId: 'client-1' })

    renderPanel()

    await waitFor(() => {
      expect(useAuditLog).toHaveBeenCalledWith({
        client_id: 'client-1',
        limit: 50,
      })
    })
  })
})
