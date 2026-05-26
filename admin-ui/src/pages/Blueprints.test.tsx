import { fireEvent, render, screen, waitFor, within } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest'

const mockBlueprintsList = vi.fn()
const mockDeleteBlueprint = vi.fn()
const mockBatchDeleteBlueprints = vi.fn()
const mockBatchAssignBlueprints = vi.fn()
const mockBatchEditBlueprints = vi.fn()
const mockCreateBlueprint = vi.fn()
const mockUpdateBlueprint = vi.fn()
const mockAffixesList = vi.fn()
const mockGlobalMetaAttributesList = vi.fn()

vi.mock('@/api/generated', () => ({
  useBlueprintsList: (query?: unknown) => mockBlueprintsList(query),
  useDeleteBlueprint: () => mockDeleteBlueprint(),
  useBatchDeleteBlueprints: () => mockBatchDeleteBlueprints(),
  useBatchAssignBlueprints: () => mockBatchAssignBlueprints(),
  useBatchEditBlueprints: () => mockBatchEditBlueprints(),
  useCreateBlueprint: () => mockCreateBlueprint(),
  useUpdateBlueprint: () => mockUpdateBlueprint(),
  useAffixesList: (query?: unknown) => mockAffixesList(query),
  useGlobalMetaAttributesList: (query?: unknown) => mockGlobalMetaAttributesList(query),
}))

import Blueprints from '@/pages/Blueprints'

function makeBlueprint(overrides: Record<string, unknown> = {}) {
  return {
    clientId: 'client-1',
    name: `Blueprint ${overrides.id || '1'}`,
    archetype: 'weapon',
    weight: 1,
    description: null,
    attributes: {},
    attributeOrder: [],
    minPrefixes: 0,
    maxPrefixes: 0,
    minSuffixes: 0,
    maxSuffixes: 0,
    createdAt: '2025-01-01T00:00:00Z',
    updatedAt: '2025-01-15T12:00:00Z',
    ...overrides,
    id: `bp-${overrides.id || '1'}`,
  }
}

function createMockMutation() {
  return {
    mutateAsync: vi.fn(),
    isPending: false,
  }
}

function renderBlueprints() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={['/blueprints']}>
        <Blueprints />
      </MemoryRouter>
    </QueryClientProvider>,
  )
}

function getDesktopContainer(): HTMLElement {
  const el = document.querySelector('.hidden.md\\:block')
  if (!el) throw new Error('Desktop table container not found')
  return el as HTMLElement
}

describe('Blueprints list page', () => {
  beforeEach(() => {
    vi.useRealTimers()
    vi.clearAllMocks()

    mockDeleteBlueprint.mockReturnValue(createMockMutation())
    mockBatchDeleteBlueprints.mockReturnValue(createMockMutation())
    mockBatchAssignBlueprints.mockReturnValue(createMockMutation())
    mockBatchEditBlueprints.mockReturnValue(createMockMutation())
    mockCreateBlueprint.mockReturnValue(createMockMutation())
    mockUpdateBlueprint.mockReturnValue(createMockMutation())
    mockAffixesList.mockReturnValue({
      data: { data: [], total: 0 },
      isLoading: false,
      isError: false,
      error: null,
    })

    mockGlobalMetaAttributesList.mockReturnValue({
      data: { data: [], total: 0 },
      isLoading: false,
      isError: false,
      error: null,
    })

    mockBlueprintsList.mockReturnValue({
      data: { data: [], total: 0 },
      isLoading: false,
      isError: false,
      error: null,
    })
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  function setBlueprints(data: ReturnType<typeof makeBlueprint>[], total?: number) {
    mockBlueprintsList.mockImplementation((query?: Record<string, unknown>) => {
      const q = query as Record<string, unknown> | undefined
      const perPage = q?.perPage ? Number(q?.perPage) : 25
      const slice = data.slice(0, perPage)
      return {
        data: { data: slice, total: total ?? data.length },
        isLoading: false,
        isError: false,
        error: null,
      }
    })
  }

  function setLoading() {
    mockBlueprintsList.mockReturnValue({
      data: undefined,
      isLoading: true,
      isError: false,
      error: null,
    })
  }

  function setError(message = 'Network error') {
    mockBlueprintsList.mockReturnValue({
      data: undefined,
      isLoading: false,
      isError: true,
      error: new Error(message),
    })
  }

  // --- Table Rendering ---

  it('renders blueprint data in the table', () => {
    setBlueprints([
      makeBlueprint({ id: '1', name: 'Sword', archetype: 'weapon', weight: 10 }),
      makeBlueprint({ id: '2', name: 'Shield', archetype: 'armor', weight: 5 }),
    ])

    renderBlueprints()

    const desktop = getDesktopContainer()
    expect(within(desktop).getByText('Sword')).toBeInTheDocument()
    expect(within(desktop).getByText('Shield')).toBeInTheDocument()
    expect(within(desktop).getByText('weapon')).toBeInTheDocument()
    expect(within(desktop).getByText('armor')).toBeInTheDocument()
    expect(within(desktop).getByText('10')).toBeInTheDocument()
    expect(within(desktop).getByText('5')).toBeInTheDocument()
  })

  it('renders blueprint names as links to detail page', () => {
    setBlueprints([makeBlueprint({ id: '1', name: 'Sword' })])

    renderBlueprints()

    const desktop = getDesktopContainer()
    const link = within(desktop).getByText('Sword')
    expect(link.tagName).toBe('A')
    expect(link.getAttribute('href')).toBe('/blueprints/bp-1')
  })

  // --- Loading State ---

  it('shows loading skeletons while data fetches', () => {
    setLoading()

    renderBlueprints()

    const skeletons = document.querySelectorAll('.animate-pulse')
    expect(skeletons.length).toBeGreaterThan(0)
  })

  // --- Empty State ---

  it('shows empty state when no blueprints exist', () => {
    setBlueprints([])

    renderBlueprints()

    expect(screen.getByText('No blueprints found')).toBeInTheDocument()
    expect(screen.getByText('Create your first blueprint to get started')).toBeInTheDocument()
  })

  // --- Error State ---

  it('shows error state when fetch fails', () => {
    setError('Failed to connect')

    renderBlueprints()

    expect(screen.getByText('Failed to load blueprints')).toBeInTheDocument()
    expect(screen.getByText('Failed to connect')).toBeInTheDocument()
  })

  // --- Search ---

  it('renders search input with placeholder', () => {
    setBlueprints([makeBlueprint({ id: '1', name: 'Sword' })])

    renderBlueprints()

    const searchInput = screen.getByPlaceholderText('Search by name...')
    expect(searchInput).toBeInTheDocument()
    expect(searchInput.tagName).toBe('INPUT')
  })

  // --- Archetype Filter ---

  it('populates archetype dropdown from distinct archetypes', () => {
    setBlueprints([
      makeBlueprint({ id: '1', archetype: 'weapon' }),
      makeBlueprint({ id: '2', archetype: 'armor' }),
      makeBlueprint({ id: '3', archetype: 'weapon' }),
    ])

    renderBlueprints()

    const select = screen.getByLabelText('Archetype') as HTMLSelectElement
    expect(select).toBeInTheDocument()
    const optionValues = Array.from(select.options).map((o) => o.value)
    expect(optionValues).toContain('armor')
    expect(optionValues).toContain('weapon')
    expect(optionValues).toContain('all')
  })

  // --- Multi-select ---

  it('allows selecting individual rows via checkbox', () => {
    setBlueprints([
      makeBlueprint({ id: '1', name: 'Sword' }),
      makeBlueprint({ id: '2', name: 'Shield' }),
    ])

    renderBlueprints()

    const desktop = getDesktopContainer()
    const swordCheckbox = within(desktop).getByLabelText('Select Sword')
    fireEvent.click(swordCheckbox)

    expect(screen.getByText('1 selected')).toBeInTheDocument()
  })

  it('header checkbox selects/deselects all visible rows', () => {
    setBlueprints([
      makeBlueprint({ id: '1', name: 'Sword' }),
      makeBlueprint({ id: '2', name: 'Shield' }),
    ])

    renderBlueprints()

    const desktop = getDesktopContainer()
    const headerCheckbox = within(desktop).getByLabelText('Select all')
    fireEvent.click(headerCheckbox)

    expect(screen.getByText('2 selected')).toBeInTheDocument()

    fireEvent.click(headerCheckbox)

    expect(screen.queryByText(/selected/)).not.toBeInTheDocument()
  })

  // --- Batch Toolbar ---

  it('shows batch toolbar when one or more rows selected', () => {
    setBlueprints([
      makeBlueprint({ id: '1', name: 'Sword' }),
      makeBlueprint({ id: '2', name: 'Shield' }),
    ])

    renderBlueprints()

    expect(screen.queryByText('Batch Delete')).not.toBeInTheDocument()

    const desktop = getDesktopContainer()
    fireEvent.click(within(desktop).getByLabelText('Select Sword'))
    expect(screen.getByText('Batch Delete')).toBeInTheDocument()
    expect(screen.getByText('Batch Assign')).toBeInTheDocument()
    expect(screen.getByText('Batch Edit')).toBeInTheDocument()
  })

  it('clear button deselects all rows', () => {
    setBlueprints([makeBlueprint({ id: '1', name: 'Sword' })])

    renderBlueprints()

    const desktop = getDesktopContainer()
    fireEvent.click(within(desktop).getByLabelText('Select Sword'))
    expect(screen.getByText('1 selected')).toBeInTheDocument()

    fireEvent.click(screen.getByText('Clear'))
    expect(screen.queryByText(/selected/)).not.toBeInTheDocument()
  })

  it('shows batch delete confirmation dialog', async () => {
    setBlueprints([makeBlueprint({ id: '1', name: 'Sword' })])

    renderBlueprints()

    const desktop = getDesktopContainer()
    fireEvent.click(within(desktop).getByLabelText('Select Sword'))
    fireEvent.click(screen.getByText('Batch Delete'))

    await waitFor(() => {
      expect(screen.getByText('Batch Delete Blueprints')).toBeInTheDocument()
    })
  })

  // --- Row Actions ---

  it('shows Edit, Duplicate, and Delete button for each row', () => {
    setBlueprints([makeBlueprint({ id: '1', name: 'Sword' })])

    renderBlueprints()

    const desktop = getDesktopContainer()
    expect(within(desktop).getByLabelText('Edit Sword')).toBeInTheDocument()
    expect(within(desktop).getByLabelText('Duplicate Sword')).toBeInTheDocument()
    expect(within(desktop).getByLabelText('Delete Sword')).toBeInTheDocument()
  })

  it('opens edit dialog when Edit is clicked', async () => {
    setBlueprints([makeBlueprint({ id: '1', name: 'Sword', archetype: 'weapon' })])

    renderBlueprints()

    const desktop = getDesktopContainer()
    fireEvent.click(within(desktop).getByLabelText('Edit Sword'))

    await waitFor(() => {
      expect(screen.getByText('Edit Blueprint')).toBeInTheDocument()
    })
  })

  it('shows delete confirmation when Delete is clicked', async () => {
    setBlueprints([makeBlueprint({ id: '1', name: 'Sword' })])

    renderBlueprints()

    const desktop = getDesktopContainer()
    fireEvent.click(within(desktop).getByLabelText('Delete Sword'))

    await waitFor(() => {
      expect(screen.getByText('Delete Blueprint')).toBeInTheDocument()
    })
  })

  it('opens duplicate modal when Duplicate is clicked', async () => {
    setBlueprints([makeBlueprint({ id: '1', name: 'Sword', archetype: 'weapon', weight: 10 })])
    mockCreateBlueprint.mockReturnValue(createMockMutation())

    renderBlueprints()

    const desktop = getDesktopContainer()
    fireEvent.click(within(desktop).getByLabelText('Duplicate Sword'))

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Duplicate Blueprint' })).toBeInTheDocument()
    })
    expect(screen.getByDisplayValue('Sword (Copy)')).toBeInTheDocument()
  })

  // --- Pagination ---

  it('shows pagination with page numbers and prev/next', () => {
    const many = Array.from({ length: 50 }, (_, i) =>
      makeBlueprint({ id: String(i + 1), name: `Item ${i + 1}` }),
    )
    setBlueprints(many, 50)

    renderBlueprints()

    expect(screen.getByText('Prev')).toBeInTheDocument()
    expect(screen.getByText('Next')).toBeInTheDocument()
  })

  it('shows page size selector', () => {
    setBlueprints([makeBlueprint({ id: '1' })])

    renderBlueprints()

    const perPageSelect = screen.getByLabelText('Per page')
    expect(perPageSelect).toBeInTheDocument()
    expect((perPageSelect as HTMLSelectElement).value).toBe('25')
  })

  // --- Sort ---

  it('renders sortable column headers', () => {
    setBlueprints([
      makeBlueprint({ id: '2', name: 'Bow', weight: 5, updatedAt: '2025-01-01T00:00:00Z' }),
      makeBlueprint({ id: '1', name: 'Sword', weight: 10, updatedAt: '2025-01-02T00:00:00Z' }),
    ])

    renderBlueprints()

    const desktop = getDesktopContainer()
    const nameHeader = within(desktop).getByText('Name').closest('button')
    expect(nameHeader).toBeInTheDocument()

    const weightHeader = within(desktop).getByText('Weight').closest('button')
    expect(weightHeader).toBeInTheDocument()

    const updatedHeader = within(desktop).getByText('Updated').closest('button')
    expect(updatedHeader).toBeInTheDocument()
  })

  // --- Responsive ---

  it('opens batch edit dialog when Batch Edit is clicked', async () => {
    const bps = [
      makeBlueprint({
        id: '1',
        name: 'Sword',
        attributes: { damage: { valueType: 'single', value: 10 } },
        attributeOrder: ['damage'],
      }),
      makeBlueprint({
        id: '2',
        name: 'Shield',
        attributes: { damage: { valueType: 'single', value: 20 } },
        attributeOrder: ['damage'],
      }),
    ]
    setBlueprints(bps)

    renderBlueprints()

    const desktop = getDesktopContainer()
    fireEvent.click(within(desktop).getByLabelText('Select Sword'))
    fireEvent.click(within(desktop).getByLabelText('Select Shield'))

    fireEvent.click(screen.getByText('Batch Edit'))

    await waitFor(() => {
      expect(screen.getByText('Batch Edit Blueprints')).toBeInTheDocument()
    })
  })

  it('shows selected blueprint names in batch edit dialog', async () => {
    const bps = [
      makeBlueprint({
        id: '1',
        name: 'Sword',
        attributes: { damage: { valueType: 'single', value: 10 } },
        attributeOrder: ['damage'],
      }),
      makeBlueprint({
        id: '2',
        name: 'Shield',
        attributes: { damage: { valueType: 'single', value: 20 } },
        attributeOrder: ['damage'],
      }),
    ]
    setBlueprints(bps)

    renderBlueprints()

    const desktop = getDesktopContainer()
    fireEvent.click(within(desktop).getByLabelText('Select Sword'))
    fireEvent.click(within(desktop).getByLabelText('Select Shield'))
    fireEvent.click(screen.getByText('Batch Edit'))

    await waitFor(() => {
      const dialog = screen.getByRole('dialog')
      expect(within(dialog).getByText(/Editing 2 blueprints/)).toBeInTheDocument()
    })
  })

  it('shows common attributes in batch edit dialog', async () => {
    const bps = [
      makeBlueprint({
        id: '1',
        name: 'Sword',
        attributes: { damage: { valueType: 'single', value: 10 } },
        attributeOrder: ['damage'],
      }),
      makeBlueprint({
        id: '2',
        name: 'Shield',
        attributes: { damage: { valueType: 'single', value: 10 } },
        attributeOrder: ['damage'],
      }),
    ]
    setBlueprints(bps)

    renderBlueprints()

    const desktop = getDesktopContainer()
    fireEvent.click(within(desktop).getByLabelText('Select Sword'))
    fireEvent.click(within(desktop).getByLabelText('Select Shield'))
    fireEvent.click(screen.getByText('Batch Edit'))

    await waitFor(() => {
      expect(screen.getByText('damage')).toBeInTheDocument()
      expect(screen.getByText('single')).toBeInTheDocument()
    })
  })

  it('shows no common attributes message when none exist', async () => {
    const bps = [
      makeBlueprint({
        id: '1',
        name: 'Sword',
        attributes: { damage: { valueType: 'single', value: 10 } },
        attributeOrder: ['damage'],
      }),
      makeBlueprint({
        id: '2',
        name: 'Shield',
        attributes: { defense: { valueType: 'range', min: 1, max: 10 } },
        attributeOrder: ['defense'],
      }),
    ]
    setBlueprints(bps)

    renderBlueprints()

    const desktop = getDesktopContainer()
    fireEvent.click(within(desktop).getByLabelText('Select Sword'))
    fireEvent.click(within(desktop).getByLabelText('Select Shield'))
    fireEvent.click(screen.getByText('Batch Edit'))

    await waitFor(() => {
      expect(
        screen.getByText('No common attributes found across selected blueprints'),
      ).toBeInTheDocument()
    })
  })

  it('shows blank fields when attribute values differ', async () => {
    const bps = [
      makeBlueprint({
        id: '1',
        name: 'Sword',
        attributes: { damage: { valueType: 'single', value: 10 } },
        attributeOrder: ['damage'],
      }),
      makeBlueprint({
        id: '2',
        name: 'Shield',
        attributes: { damage: { valueType: 'single', value: 20 } },
        attributeOrder: ['damage'],
      }),
    ]
    setBlueprints(bps)

    renderBlueprints()

    const desktop = getDesktopContainer()
    fireEvent.click(within(desktop).getByLabelText('Select Sword'))
    fireEvent.click(within(desktop).getByLabelText('Select Shield'))
    fireEvent.click(screen.getByText('Batch Edit'))

    await waitFor(() => {
      expect(screen.getByText(/values differ/)).toBeInTheDocument()
    })
  })

  it('excludes attributes with different value types', async () => {
    const bps = [
      makeBlueprint({
        id: '1',
        name: 'Sword',
        attributes: { damage: { valueType: 'single', value: 10 } },
        attributeOrder: ['damage'],
      }),
      makeBlueprint({
        id: '2',
        name: 'Shield',
        attributes: { damage: { valueType: 'range', min: 1, max: 10 } },
        attributeOrder: ['damage'],
      }),
    ]
    setBlueprints(bps)

    renderBlueprints()

    const desktop = getDesktopContainer()
    fireEvent.click(within(desktop).getByLabelText('Select Sword'))
    fireEvent.click(within(desktop).getByLabelText('Select Shield'))
    fireEvent.click(screen.getByText('Batch Edit'))

    await waitFor(() => {
      expect(
        screen.getByText('No common attributes found across selected blueprints'),
      ).toBeInTheDocument()
    })
  })

  it('submits batch edit with edited values', async () => {
    const mutateAsync = vi.fn().mockResolvedValue(undefined)
    mockBatchEditBlueprints.mockReturnValue({ mutateAsync, isPending: false })

    const bps = [
      makeBlueprint({
        id: '1',
        name: 'Sword',
        attributes: { damage: { valueType: 'single', value: 10 } },
        attributeOrder: ['damage'],
      }),
      makeBlueprint({
        id: '2',
        name: 'Shield',
        attributes: { damage: { valueType: 'single', value: 10 } },
        attributeOrder: ['damage'],
      }),
    ]
    setBlueprints(bps)

    renderBlueprints()

    const desktop = getDesktopContainer()
    fireEvent.click(within(desktop).getByLabelText('Select Sword'))
    fireEvent.click(within(desktop).getByLabelText('Select Shield'))
    fireEvent.click(screen.getByText('Batch Edit'))

    await waitFor(() => {
      expect(screen.getByText('Batch Edit Blueprints')).toBeInTheDocument()
    })

    const valueInput = screen.getByPlaceholderText('Enter a number')
    fireEvent.change(valueInput, { target: { value: '50' } })

    const dialog = screen.getByRole('dialog')
    fireEvent.click(within(dialog).getByRole('button', { name: /Update/ }))

    await waitFor(() => {
      expect(mutateAsync).toHaveBeenCalledWith({
        blueprintIds: ['bp-1', 'bp-2'],
        attributes: expect.objectContaining({
          damage: expect.objectContaining({ valueType: 'single', value: 50 }),
        }),
      })
    })
  })

  it('clears selection after successful batch edit', async () => {
    const mutateAsync = vi.fn().mockResolvedValue(undefined)
    mockBatchEditBlueprints.mockReturnValue({ mutateAsync, isPending: false })

    const bps = [
      makeBlueprint({
        id: '1',
        name: 'Sword',
        attributes: { damage: { valueType: 'single', value: 10 } },
        attributeOrder: ['damage'],
      }),
    ]
    setBlueprints(bps)

    renderBlueprints()

    const desktop = getDesktopContainer()
    fireEvent.click(within(desktop).getByLabelText('Select Sword'))
    fireEvent.click(screen.getByText('Batch Edit'))

    await waitFor(() => {
      expect(screen.getByText('Batch Edit Blueprints')).toBeInTheDocument()
    })

    const valueInput = screen.getByPlaceholderText('Enter a number')
    fireEvent.change(valueInput, { target: { value: '999' } })

    const dialog = screen.getByRole('dialog')
    fireEvent.click(within(dialog).getByRole('button', { name: /Update/ }))

    await waitFor(() => {
      expect(screen.queryByText(/selected/)).not.toBeInTheDocument()
    })
  })

  it('shows error toast on batch edit failure', async () => {
    const mutateAsync = vi.fn().mockRejectedValue(new Error('Server error'))
    mockBatchEditBlueprints.mockReturnValue({ mutateAsync, isPending: false })

    const bps = [
      makeBlueprint({
        id: '1',
        name: 'Sword',
        attributes: { damage: { valueType: 'single', value: 10 } },
        attributeOrder: ['damage'],
      }),
    ]
    setBlueprints(bps)

    renderBlueprints()

    const desktop = getDesktopContainer()
    fireEvent.click(within(desktop).getByLabelText('Select Sword'))
    fireEvent.click(screen.getByText('Batch Edit'))

    await waitFor(() => {
      expect(screen.getByText('Batch Edit Blueprints')).toBeInTheDocument()
    })

    const valueInput = screen.getByPlaceholderText('Enter a number')
    fireEvent.change(valueInput, { target: { value: '999' } })

    const dialog = screen.getByRole('dialog')
    fireEvent.click(within(dialog).getByRole('button', { name: /Update/ }))

    await waitFor(() => {
      expect(mutateAsync).toHaveBeenCalled()
    })
  })

  it('dialog submit button shows update count when not pending', async () => {
    mockBatchEditBlueprints.mockReturnValue({ mutateAsync: vi.fn(), isPending: false })

    const bps = [
      makeBlueprint({
        id: '1',
        name: 'Sword',
        attributes: { damage: { valueType: 'single', value: 10 } },
        attributeOrder: ['damage'],
      }),
    ]
    setBlueprints(bps)

    renderBlueprints()

    const desktop = getDesktopContainer()
    fireEvent.click(within(desktop).getByLabelText('Select Sword'))
    fireEvent.click(screen.getByText('Batch Edit'))

    await waitFor(() => {
      expect(screen.getByText('Batch Edit Blueprints')).toBeInTheDocument()
    })

    const dialog = screen.getByRole('dialog')
    expect(
      within(dialog).getByRole('button', { name: /Update \(1 blueprint\)/ }),
    ).toBeInTheDocument()
  })

  it('disables batch edit toolbar button when mutation is pending', async () => {
    mockBatchEditBlueprints.mockReturnValue({ mutateAsync: vi.fn(), isPending: true })

    setBlueprints([makeBlueprint({ id: '1', name: 'Sword' })])

    renderBlueprints()

    const desktop = getDesktopContainer()
    fireEvent.click(within(desktop).getByLabelText('Select Sword'))

    const batchEditBtn = screen.getByText('Batch Edit')
    expect(batchEditBtn).toBeDisabled()
  })

  it('handles boolean attributes in batch edit', async () => {
    const bps = [
      makeBlueprint({
        id: '1',
        name: 'Sword',
        attributes: { magic: { valueType: 'boolean', value: true } },
        attributeOrder: ['magic'],
      }),
      makeBlueprint({
        id: '2',
        name: 'Shield',
        attributes: { magic: { valueType: 'boolean', value: false } },
        attributeOrder: ['magic'],
      }),
    ]
    setBlueprints(bps)

    renderBlueprints()

    const desktop = getDesktopContainer()
    fireEvent.click(within(desktop).getByLabelText('Select Sword'))
    fireEvent.click(within(desktop).getByLabelText('Select Shield'))
    fireEvent.click(screen.getByText('Batch Edit'))

    await waitFor(() => {
      expect(screen.getByText('magic')).toBeInTheDocument()
      expect(screen.getByText(/values differ/)).toBeInTheDocument()
    })
  })

  it('handles enum attributes in batch edit', async () => {
    const bps = [
      makeBlueprint({
        id: '1',
        name: 'Sword',
        attributes: { element: { valueType: 'enum', values: ['fire', 'ice'] } },
        attributeOrder: ['element'],
      }),
      makeBlueprint({
        id: '2',
        name: 'Shield',
        attributes: { element: { valueType: 'enum', values: ['fire', 'ice'] } },
        attributeOrder: ['element'],
      }),
    ]
    setBlueprints(bps)

    renderBlueprints()

    const desktop = getDesktopContainer()
    fireEvent.click(within(desktop).getByLabelText('Select Sword'))
    fireEvent.click(within(desktop).getByLabelText('Select Shield'))
    fireEvent.click(screen.getByText('Batch Edit'))

    await waitFor(() => {
      expect(screen.getByText('element')).toBeInTheDocument()
      const enumInput = screen.getByPlaceholderText('e.g. fire, ice, lightning')
      expect(enumInput).toBeInTheDocument()
      expect((enumInput as HTMLInputElement).value).toBe('fire, ice')
    })
  })

  it('handles range attributes in batch edit', async () => {
    const bps = [
      makeBlueprint({
        id: '1',
        name: 'Sword',
        attributes: { level: { valueType: 'range', min: 1, max: 50 } },
        attributeOrder: ['level'],
      }),
      makeBlueprint({
        id: '2',
        name: 'Shield',
        attributes: { level: { valueType: 'range', min: 1, max: 50 } },
        attributeOrder: ['level'],
      }),
    ]
    setBlueprints(bps)

    renderBlueprints()

    const desktop = getDesktopContainer()
    fireEvent.click(within(desktop).getByLabelText('Select Sword'))
    fireEvent.click(within(desktop).getByLabelText('Select Shield'))
    fireEvent.click(screen.getByText('Batch Edit'))

    await waitFor(() => {
      expect(screen.getByText('level')).toBeInTheDocument()
      const minInputs = screen.getAllByPlaceholderText('Min')
      const maxInputs = screen.getAllByPlaceholderText('Max')
      expect(minInputs.length).toBeGreaterThan(0)
      expect(maxInputs.length).toBeGreaterThan(0)
    })
  })

  it('excludes ref_id attributes from batch edit', async () => {
    const bps = [
      makeBlueprint({
        id: '1',
        name: 'Sword',
        attributes: { style: { $ref_id: 'gma-1' }, damage: { valueType: 'single', value: 10 } },
        attributeOrder: ['style', 'damage'],
      }),
      makeBlueprint({
        id: '2',
        name: 'Shield',
        attributes: { style: { $ref_id: 'gma-1' }, damage: { valueType: 'single', value: 20 } },
        attributeOrder: ['style', 'damage'],
      }),
    ]
    setBlueprints(bps)

    renderBlueprints()

    const desktop = getDesktopContainer()
    fireEvent.click(within(desktop).getByLabelText('Select Sword'))
    fireEvent.click(within(desktop).getByLabelText('Select Shield'))
    fireEvent.click(screen.getByText('Batch Edit'))

    await waitFor(() => {
      expect(screen.getByText('damage')).toBeInTheDocument()
      expect(screen.queryByText('style')).not.toBeInTheDocument()
    })
  })

  it('renders mobile card view on small screens', () => {
    setBlueprints([makeBlueprint({ id: '1', name: 'Sword' })])

    renderBlueprints()

    const mobileContainer = document.querySelector('.md\\:hidden.divide-y')
    expect(mobileContainer).toBeInTheDocument()
  })
})
