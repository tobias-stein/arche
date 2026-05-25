import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { describe, expect, it, vi, beforeEach } from 'vitest'

const mockBlueprintsList = vi.fn()
const mockAffixesList = vi.fn()
const mockGenerateMutation = {
  mutateAsync: vi.fn(),
  isPending: false,
}

vi.mock('@/api/generated', () => ({
  useBlueprintsList: (query?: unknown) => mockBlueprintsList(query),
  useAffixesList: (query?: unknown) => mockAffixesList(query),
  useGenerate: () => mockGenerateMutation,
}))

const mockToast = vi.fn()

vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: mockToast, dismiss: vi.fn(), toasts: [] }),
}))

import Dashboard from '@/pages/Dashboard'

function makeBlueprint(overrides: Record<string, unknown> = {}) {
  return {
    id: `bp-${overrides.id || '1'}`,
    clientId: 'client-1',
    name: `Blueprint ${overrides.id || '1'}`,
    archetype: 'weapon',
    weight: 1,
    description: null,
    attributes: {
      damage: {
        valueType: 'single',
        value: 0,
        distribution: { type: 'uniform' },
      },
      element: {
        valueType: 'enum',
        values: ['fire', 'ice', 'lightning'],
      },
      name: {
        valueType: 'string',
        minLength: 3,
        maxLength: 16,
      },
      legendary: {
        valueType: 'boolean',
        value: false,
      },
      price_range: {
        valueType: 'range',
        min: 1,
        max: 100,
      },
    },
    attributeOrder: ['damage', 'element', 'name', 'legendary', 'price_range'],
    minPrefixes: 0,
    maxPrefixes: 0,
    minSuffixes: 0,
    maxSuffixes: 0,
    createdAt: '2025-01-01T00:00:00Z',
    updatedAt: '2025-01-15T12:00:00Z',
    ...overrides,
  }
}

function makeAffix(overrides: Record<string, unknown> = {}) {
  return {
    id: `aff-${overrides.id || '1'}`,
    clientId: 'client-1',
    name: `Affix ${overrides.id || '1'}`,
    location: 'prefix',
    description: null,
    attribute: {
      valueType: 'single',
      value: 0,
    },
    createdAt: '2025-01-01T00:00:00Z',
    updatedAt: '2025-01-15T12:00:00Z',
    ...overrides,
  }
}

function renderDashboard() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={['/dashboard']}>
        <Dashboard />
      </MemoryRouter>
    </QueryClientProvider>,
  )
}

describe('Dashboard Quick Generate', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockGenerateMutation.isPending = false
    mockGenerateMutation.mutateAsync.mockReset()
    mockToast.mockReset()

    mockBlueprintsList.mockReturnValue({
      data: { data: [], total: 0 },
      isLoading: false,
      isError: false,
      error: null,
    })

    mockAffixesList.mockReturnValue({
      data: { data: [], total: 0 },
      isLoading: false,
      isError: false,
      error: null,
    })
  })

  // --- Loading State ---
  it('shows skeletons while data is loading', () => {
    mockBlueprintsList.mockReturnValue({
      data: undefined,
      isLoading: true,
      isError: false,
      error: null,
    })
    mockAffixesList.mockReturnValue({
      data: undefined,
      isLoading: true,
      isError: false,
      error: null,
    })

    renderDashboard()

    const skeletons = document.querySelectorAll('.animate-pulse')
    expect(skeletons.length).toBeGreaterThan(0)
  })

  // --- Error State ---
  it('shows error message when blueprints fail to load', () => {
    mockBlueprintsList.mockReturnValue({
      data: undefined,
      isLoading: false,
      isError: true,
      error: new Error('Network error'),
    })

    renderDashboard()

    expect(
      screen.getByText(/Failed to load blueprints/),
    ).toBeInTheDocument()
  })

  // --- Empty State ---
  it('shows message when no blueprints exist', () => {
    renderDashboard()

    expect(
      screen.getByText(/No blueprints found/),
    ).toBeInTheDocument()
    expect(
      screen.getByText(/No attributes available/),
    ).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /Generate/i })).toBeDisabled()
  })

  // --- Archetype Dropdown Populated ---
  it('populates archetype dropdown from blueprint data', () => {
    mockBlueprintsList.mockReturnValue({
      data: {
        data: [
          makeBlueprint({ id: '1', archetype: 'weapon' }),
          makeBlueprint({ id: '2', archetype: 'armor' }),
          makeBlueprint({ id: '3', archetype: 'weapon' }),
        ],
        total: 3,
      },
      isLoading: false,
      isError: false,
      error: null,
    })

    renderDashboard()

    expect(screen.getByText('Any archetype')).toBeInTheDocument()

    const trigger = screen.getByRole('combobox', { name: /Archetype/i })
    fireEvent.click(trigger)

    expect(screen.getByText('armor')).toBeInTheDocument()
    expect(screen.getByText('weapon')).toBeInTheDocument()
  })

  // --- Archetype Selection ---
  it('selects an archetype and includes it in generate request', async () => {
    mockBlueprintsList.mockReturnValue({
      data: {
        data: [makeBlueprint({ id: '1', archetype: 'weapon' })],
        total: 1,
      },
      isLoading: false,
      isError: false,
      error: null,
    })
    mockGenerateMutation.mutateAsync.mockResolvedValue({
      seed: 42,
      name: 'Flaming Sword of Doom',
      nameParts: { base: 'Sword', prefixes: ['Flaming'], suffixes: ['of Doom'] },
      blueprintId: 'bp-1',
      blueprintAttributes: { damage: 50, element: 'fire' },
      affixAttributes: [],
    })

    renderDashboard()

    const trigger = screen.getByRole('combobox', { name: /Archetype/i })
    fireEvent.click(trigger)
    fireEvent.click(screen.getByText('weapon'))

    fireEvent.click(screen.getByRole('button', { name: /Generate/i }))

    await waitFor(() => {
      expect(mockGenerateMutation.mutateAsync).toHaveBeenCalledWith(
        expect.objectContaining({ archetype: 'weapon' }),
      )
    })
  })

  // --- Constraint Builder ---
  it('adds and removes constraint rows', () => {
    mockBlueprintsList.mockReturnValue({
      data: {
        data: [makeBlueprint()],
        total: 1,
      },
      isLoading: false,
      isError: false,
      error: null,
    })

    renderDashboard()

    expect(screen.queryByPlaceholderText('Value')).not.toBeInTheDocument()

    fireEvent.click(screen.getByText('Add constraint'))

    expect(screen.getByPlaceholderText('Value')).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: '' })) // trash button
  })

  it('populates attribute key dropdown from blueprint attributes', () => {
    mockBlueprintsList.mockReturnValue({
      data: {
        data: [makeBlueprint()],
        total: 1,
      },
      isLoading: false,
      isError: false,
      error: null,
    })

    renderDashboard()

    fireEvent.click(screen.getByText('Add constraint'))

    const triggers = screen.getAllByRole('combobox')
    const attributeTrigger = triggers.find(
      (el) => (el as HTMLElement).textContent === 'Attribute',
    )
    expect(attributeTrigger).toBeTruthy()
    if (attributeTrigger) fireEvent.click(attributeTrigger)

    expect(screen.getByText('damage')).toBeInTheDocument()
    expect(screen.getByText('element')).toBeInTheDocument()
    expect(screen.getByText('name')).toBeInTheDocument()
    expect(screen.getByText('legendary')).toBeInTheDocument()
    expect(screen.getByText('price_range')).toBeInTheDocument()
  })

  // --- Seed Input ---
  it('includes seed in generate request when provided', async () => {
    mockBlueprintsList.mockReturnValue({
      data: {
        data: [makeBlueprint()],
        total: 1,
      },
      isLoading: false,
      isError: false,
      error: null,
    })
    mockGenerateMutation.mutateAsync.mockResolvedValue({
      seed: 12345,
      name: 'Test Item',
      nameParts: { base: 'Item', prefixes: [], suffixes: [] },
      blueprintId: 'bp-1',
      blueprintAttributes: {},
      affixAttributes: [],
    })

    renderDashboard()

    const seedInput = screen.getByPlaceholderText('Leave empty for random')
    fireEvent.change(seedInput, { target: { value: '12345' } })

    fireEvent.click(screen.getByRole('button', { name: /Generate/i }))

    await waitFor(() => {
      expect(mockGenerateMutation.mutateAsync).toHaveBeenCalledWith(
        expect.objectContaining({ seed: 12345 }),
      )
    })
  })

  it('sends null seed when input is empty', async () => {
    mockBlueprintsList.mockReturnValue({
      data: {
        data: [makeBlueprint()],
        total: 1,
      },
      isLoading: false,
      isError: false,
      error: null,
    })
    mockGenerateMutation.mutateAsync.mockResolvedValue({
      seed: 9999,
      name: 'Random Item',
      nameParts: { base: 'Item', prefixes: [], suffixes: [] },
      blueprintId: 'bp-1',
      blueprintAttributes: {},
      affixAttributes: [],
    })

    renderDashboard()

    fireEvent.click(screen.getByRole('button', { name: /Generate/i }))

    await waitFor(() => {
      expect(mockGenerateMutation.mutateAsync).toHaveBeenCalledWith(
        expect.objectContaining({ seed: null }),
      )
    })
  })

  // --- Generate Result Display ---
  it('displays generated result after successful generation', async () => {
    mockBlueprintsList.mockReturnValue({
      data: {
        data: [makeBlueprint()],
        total: 1,
      },
      isLoading: false,
      isError: false,
      error: null,
    })
    mockGenerateMutation.mutateAsync.mockResolvedValue({
      seed: 42,
      name: 'Flaming Blade of Destiny',
      nameParts: { base: 'Blade', prefixes: ['Flaming'], suffixes: ['of Destiny'] },
      blueprintId: 'bp-1',
      blueprintAttributes: { damage: 85, element: 'fire', durability: 100 },
      affixAttributes: [
        {
          affixId: 'aff-1',
          affixName: 'Flaming',
          fireDamage: 25,
        },
        {
          affixId: 'aff-2',
          affixName: 'Destiny',
          luck: 10,
        },
      ],
    })

    renderDashboard()

    fireEvent.click(screen.getByRole('button', { name: /Generate/i }))

    await waitFor(() => {
      expect(screen.getByText('Flaming Blade of Destiny')).toBeInTheDocument()
    })

    expect(screen.getByText('42')).toBeInTheDocument()
    expect(screen.getByText('bp-1')).toBeInTheDocument()
    expect(screen.getByText(/damage: 85/)).toBeInTheDocument()
    expect(screen.getByText(/element: "fire"/)).toBeInTheDocument()
    expect(screen.getByText(/durability: 100/)).toBeInTheDocument()

    expect(screen.getByText('Flaming')).toBeInTheDocument()
    expect(screen.getByText('Destiny')).toBeInTheDocument()
  })

  // --- Error State (no matching blueprints) ---
  it('shows error message when generate fails', async () => {
    mockBlueprintsList.mockReturnValue({
      data: {
        data: [makeBlueprint()],
        total: 1,
      },
      isLoading: false,
      isError: false,
      error: null,
    })
    mockGenerateMutation.mutateAsync.mockRejectedValue(
      new Error('No matching blueprints found'),
    )

    renderDashboard()

    fireEvent.click(screen.getByRole('button', { name: /Generate/i }))

    await waitFor(() => {
      expect(screen.getByText('No matching blueprints found')).toBeInTheDocument()
    })

    expect(mockToast).toHaveBeenCalledWith(
      expect.objectContaining({ variant: 'destructive' }),
    )
  })

  // --- Clear Button ---
  it('clears the form when Clear button is clicked', async () => {
    mockBlueprintsList.mockReturnValue({
      data: {
        data: [makeBlueprint()],
        total: 1,
      },
      isLoading: false,
      isError: false,
      error: null,
    })
    mockGenerateMutation.mutateAsync.mockResolvedValue({
      seed: 1,
      name: 'Test',
      nameParts: { base: 'Test', prefixes: [], suffixes: [] },
      blueprintId: 'bp-1',
      blueprintAttributes: {},
      affixAttributes: [],
    })

    renderDashboard()

    const trigger = screen.getByRole('combobox', { name: /Archetype/i })
    fireEvent.click(trigger)
    fireEvent.click(screen.getByText('weapon'))

    fireEvent.click(screen.getByText('Add constraint'))

    const seedInput = screen.getByPlaceholderText('Leave empty for random')
    fireEvent.change(seedInput, { target: { value: '42' } })

    const minPrefixes = screen.getByLabelText('Min Prefixes')
    fireEvent.change(minPrefixes, { target: { value: '2' } })

    fireEvent.click(screen.getByRole('button', { name: /Generate/i }))

    await waitFor(() => {
      expect(screen.getByText('Test')).toBeInTheDocument()
    })

    fireEvent.click(screen.getByRole('button', { name: /Clear/i }))

    await waitFor(() => {
      expect(screen.queryByText('Test')).not.toBeInTheDocument()
    })

    expect(seedInput).toHaveValue('')
    expect(minPrefixes).toHaveValue(0)
  })

  // --- Loading State During Generation ---
  it('shows loading state during generation', async () => {
    mockBlueprintsList.mockReturnValue({
      data: {
        data: [makeBlueprint()],
        total: 1,
      },
      isLoading: false,
      isError: false,
      error: null,
    })
    mockGenerateMutation.isPending = true
    mockGenerateMutation.mutateAsync.mockResolvedValue({
      seed: 1,
      name: 'Test',
      nameParts: { base: 'Test', prefixes: [], suffixes: [] },
      blueprintId: 'bp-1',
      blueprintAttributes: {},
      affixAttributes: [],
    })

    renderDashboard()

    expect(screen.getByText('Generating...')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /Generating/i })).toBeDisabled()
  })

  // --- Affix Controls ---
  it('sends affix constraints in generate request', async () => {
    mockBlueprintsList.mockReturnValue({
      data: {
        data: [makeBlueprint()],
        total: 1,
      },
      isLoading: false,
      isError: false,
      error: null,
    })
    mockAffixesList.mockReturnValue({
      data: {
        data: [
          makeAffix({ id: '1', name: 'Flaming' }),
          makeAffix({ id: '2', name: 'Frozen', location: 'suffix' }),
        ],
        total: 2,
      },
      isLoading: false,
      isError: false,
      error: null,
    })
    mockGenerateMutation.mutateAsync.mockResolvedValue({
      seed: 1,
      name: 'Test',
      nameParts: { base: 'Test', prefixes: [], suffixes: [] },
      blueprintId: 'bp-1',
      blueprintAttributes: {},
      affixAttributes: [],
    })

    renderDashboard()

    const minPrefixes = screen.getByLabelText('Min Prefixes')
    fireEvent.change(minPrefixes, { target: { value: '1' } })

    const maxSuffixes = screen.getByLabelText('Max Suffixes')
    fireEvent.change(maxSuffixes, { target: { value: '2' } })

    const flamingCheckboxes = screen.getAllByRole('checkbox', { name: 'Flaming' })
    fireEvent.click(flamingCheckboxes[0])

    fireEvent.click(screen.getByRole('button', { name: /Generate/i }))

    await waitFor(() => {
      const call = mockGenerateMutation.mutateAsync.mock.calls[0][0]
      expect(call.affixes.minPrefixes).toBe(1)
      expect(call.affixes.maxSuffixes).toBe(2)
      expect(call.affixes.require).toContain('1')
    })
  })

  // --- Affix Filter Search ---
  it('filters affixes by search input', () => {
    mockBlueprintsList.mockReturnValue({
      data: {
        data: [makeBlueprint()],
        total: 1,
      },
      isLoading: false,
      isError: false,
      error: null,
    })
    mockAffixesList.mockReturnValue({
      data: {
        data: [
          makeAffix({ id: '1', name: 'Flaming' }),
          makeAffix({ id: '2', name: 'Frozen' }),
          makeAffix({ id: '3', name: 'Sharp' }),
        ],
        total: 3,
      },
      isLoading: false,
      isError: false,
      error: null,
    })

    renderDashboard()

    const flamingElements = screen.getAllByText('Flaming')
    expect(flamingElements.length).toBeGreaterThanOrEqual(2)

    const filterInputs = screen.getAllByPlaceholderText('Filter affixes...')
    fireEvent.change(filterInputs[0], { target: { value: 'Fro' } })

    expect(screen.getAllByText('Frozen').length).toBe(2)
    expect(screen.getAllByText('Flaming').length).toBe(1)
    expect(screen.getAllByText('Sharp').length).toBe(1)
  })

  // --- Constraint Values in Request ---
  it('builds constraint values correctly in generate request', async () => {
    mockBlueprintsList.mockReturnValue({
      data: {
        data: [makeBlueprint()],
        total: 1,
      },
      isLoading: false,
      isError: false,
      error: null,
    })
    mockGenerateMutation.mutateAsync.mockResolvedValue({
      seed: 1,
      name: 'Test',
      nameParts: { base: 'Test', prefixes: [], suffixes: [] },
      blueprintId: 'bp-1',
      blueprintAttributes: {},
      affixAttributes: [],
    })

    renderDashboard()

    fireEvent.click(screen.getByText('Add constraint'))

    const triggers = screen.getAllByRole('combobox')
    const attributeTrigger = triggers.find(
      (el) => (el as HTMLElement).textContent === 'Attribute',
    )
    if (attributeTrigger) fireEvent.click(attributeTrigger)
    fireEvent.click(screen.getByText('damage'))

    const valueInput = screen.getByPlaceholderText('Number')
    fireEvent.change(valueInput, { target: { value: '100' } })

    fireEvent.click(screen.getByRole('button', { name: /Generate/i }))

    await waitFor(() => {
      expect(mockGenerateMutation.mutateAsync).toHaveBeenCalledWith(
        expect.objectContaining({
          constraints: { damage: 100 },
        }),
      )
    })
  })

  // --- Empty Affixes ---
  it('handles empty affix list gracefully', () => {
    mockBlueprintsList.mockReturnValue({
      data: {
        data: [makeBlueprint()],
        total: 1,
      },
      isLoading: false,
      isError: false,
      error: null,
    })
    mockAffixesList.mockReturnValue({
      data: { data: [], total: 0 },
      isLoading: false,
      isError: false,
      error: null,
    })

    renderDashboard()

    expect(screen.getAllByText('No affixes found')).toHaveLength(2)
    expect(screen.getByText('Generate')).not.toBeDisabled()
  })
})
