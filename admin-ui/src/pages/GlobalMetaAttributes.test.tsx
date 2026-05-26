import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { describe, expect, it, vi } from 'vitest'
import type { GlobalMetaAttribute } from '@/api/generated'
import { setClient } from '@/api/generated/hooks'
import { ArcheClient } from '@/api/generated/client'
import GlobalMetaAttributes from '@/pages/GlobalMetaAttributes'

setClient(new ArcheClient({ baseUrl: 'http://localhost:3000' }))

const mockGMAs: GlobalMetaAttribute[] = [
  {
    id: 'gma-1',
    clientId: 'client-1',
    name: 'Damage',
    description: 'Weapon damage',
    valueType: 'single',
    payload: { value: 10 },
    createdAt: '2026-01-15T00:00:00Z',
    updatedAt: '2026-02-20T00:00:00Z',
  },
  {
    id: 'gma-2',
    clientId: 'client-1',
    name: 'Element',
    description: 'Element type',
    valueType: 'enum',
    payload: { values: ['Fire', 'Ice', 'Lightning'] },
    createdAt: '2026-01-16T00:00:00Z',
    updatedAt: '2026-02-21T00:00:00Z',
  },
  {
    id: 'gma-3',
    clientId: 'client-1',
    name: 'Durability',
    description: 'Max durability range',
    valueType: 'range',
    payload: { min: 50, max: 200 },
    createdAt: '2026-01-17T00:00:00Z',
    updatedAt: '2026-02-22T00:00:00Z',
  },
  {
    id: 'gma-4',
    clientId: 'client-1',
    name: 'IsMagical',
    description: 'Whether item is magical',
    valueType: 'boolean',
    payload: { value: false },
    createdAt: '2026-01-18T00:00:00Z',
    updatedAt: '2026-02-23T00:00:00Z',
  },
  {
    id: 'gma-5',
    clientId: 'client-1',
    name: 'Lore',
    description: 'Item lore text',
    valueType: 'string',
    payload: { minLength: 10, maxLength: 200 },
    createdAt: '2026-01-19T00:00:00Z',
    updatedAt: '2026-02-24T00:00:00Z',
  },
]

const mockListHook = {
  data: { data: mockGMAs, total: 5 },
  isLoading: false,
  isError: false,
  error: null,
}

vi.mock('@/api/generated', async () => {
  const actual = await vi.importActual('@/api/generated')
  return {
    ...actual,
    useGlobalMetaAttributesList: () => mockListHook,
    useDeleteGlobalMetaAttribute: () => ({
      mutateAsync: vi.fn().mockResolvedValue(undefined),
      isPending: false,
    }),
    useUpdateGlobalMetaAttribute: () => ({
      mutateAsync: vi.fn().mockResolvedValue(undefined),
      isPending: false,
    }),
    useCreateGlobalMetaAttribute: () => ({
      mutateAsync: vi.fn().mockResolvedValue(undefined),
      isPending: false,
    }),
    useBlueprintsList: () => ({
      data: { data: [], total: 0 },
      isLoading: false,
    }),
    useAffixesList: () => ({
      data: { data: [], total: 0 },
      isLoading: false,
    }),
  }
})

function renderPage() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={['/global-meta-attributes']}>
        <GlobalMetaAttributes />
      </MemoryRouter>
    </QueryClientProvider>,
  )
}

// Helper to get the desktop table (hidden on mobile via CSS class)
function getDesktopTable(): HTMLElement {
  const desktopDiv = document.querySelector('.hidden.md\\:block .relative.w-full.overflow-auto')
  if (!desktopDiv) throw new Error('Desktop table not found')
  return desktopDiv as HTMLElement
}

describe('GlobalMetaAttributes list page', () => {
  it('renders the page title', () => {
    renderPage()
    expect(screen.getByText('Global Meta Attributes')).toBeInTheDocument()
  })

  it('renders all GMA names in the table', () => {
    renderPage()
    // Both desktop and mobile render names, so getAllByText returns 2 per name
    expect(screen.getAllByText('Damage').length).toBeGreaterThanOrEqual(1)
    expect(screen.getAllByText('Element').length).toBeGreaterThanOrEqual(1)
    expect(screen.getAllByText('Durability').length).toBeGreaterThanOrEqual(1)
    expect(screen.getAllByText('IsMagical').length).toBeGreaterThanOrEqual(1)
    expect(screen.getAllByText('Lore').length).toBeGreaterThanOrEqual(1)
  })

  it('renders value type badges in the desktop table', () => {
    renderPage()
    const table = within(getDesktopTable())
    expect(table.getByText('single')).toBeInTheDocument()
    expect(table.getByText('enum')).toBeInTheDocument()
    expect(table.getByText('range')).toBeInTheDocument()
    expect(table.getByText('boolean')).toBeInTheDocument()
    expect(table.getByText('string')).toBeInTheDocument()
  })

  it('renders preview values in the desktop table', () => {
    renderPage()
    const table = within(getDesktopTable())
    expect(table.getByText('10')).toBeInTheDocument()
    expect(table.getByText('Fire, Ice, Lightning')).toBeInTheDocument()
    expect(table.getByText('50 \u2013 200')).toBeInTheDocument()
    expect(table.getByText('false')).toBeInTheDocument()
    expect(table.getByText('10 \u2013 200 characters')).toBeInTheDocument()
  })

  it('shows the value type filter dropdown with all options', () => {
    renderPage()
    const select = screen.getByLabelText('Value Type')
    expect(select).toBeInTheDocument()
    const options = Array.from(select.querySelectorAll('option')).map(o => o.textContent)
    expect(options).toContain('All')
    expect(options).toContain('Single')
    expect(options).toContain('Enum')
    expect(options).toContain('Range')
    expect(options).toContain('String')
    expect(options).toContain('Boolean')
  })

  it('filters by value type', async () => {
    renderPage()
    const user = userEvent.setup()

    expect(screen.getAllByText('Damage').length).toBeGreaterThanOrEqual(1)

    const select = screen.getByLabelText('Value Type')
    await user.selectOptions(select, 'enum')

    expect(screen.getAllByText('Element').length).toBeGreaterThanOrEqual(1)
    expect(screen.queryAllByText('Damage').length).toBe(0)
  })

  it('shows search input', () => {
    renderPage()
    const input = screen.getByPlaceholderText('Search by name...')
    expect(input).toBeInTheDocument()
  })

  it('shows pagination when data is loaded', () => {
    renderPage()
    expect(screen.getByText(/Page 1/)).toBeInTheDocument()
    expect(screen.getByText(/5 total/)).toBeInTheDocument()
  })

  it('renders edit and delete buttons for each row', () => {
    renderPage()
    const editButtons = screen.getAllByLabelText(/^Edit /)
    const deleteButtons = screen.getAllByLabelText(/^Delete /)
    // 5 items, each appears in both desktop and mobile = 10
    expect(editButtons.length).toBe(10)
    expect(deleteButtons.length).toBe(10)
  })

  it('does not render checkboxes (no multi-select)', () => {
    renderPage()
    const checkboxes = screen.queryAllByRole('checkbox')
    expect(checkboxes.length).toBe(0)
  })

  it('opens edit dialog when a desktop edit button is clicked', async () => {
    renderPage()
    const user = userEvent.setup()

    // Click the first desktop edit button (h-9 w-9 -> desktop sizing)
    const desktopEdit = screen.getAllByLabelText('Edit Damage')[0]!
    await user.click(desktopEdit)

    expect(screen.getByRole('dialog')).toBeInTheDocument()
    expect(screen.getByText('Edit Global Meta Attribute')).toBeInTheDocument()
  })

  it('opens delete confirmation when a desktop delete button is clicked', async () => {
    renderPage()
    const user = userEvent.setup()

    const desktopDelete = screen.getAllByLabelText('Delete Damage')[0]!
    await user.click(desktopDelete)

    expect(screen.getByRole('dialog')).toBeInTheDocument()
    expect(screen.getByText('Delete Global Meta Attribute')).toBeInTheDocument()
    expect(screen.getByText(/Are you sure you want to delete/)).toBeInTheDocument()
  })
})
