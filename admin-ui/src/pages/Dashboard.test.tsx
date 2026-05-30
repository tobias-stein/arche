import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { describe, expect, it, vi, beforeEach } from 'vitest'

const mockBlueprintsList = vi.fn()
const mockAffixesList = vi.fn()
const mockGlobalMetaAttributesList = vi.fn()
const mockGenerateMutation = {
  mutateAsync: vi.fn(),
  isPending: false,
}

vi.mock('@/api/generated', () => ({
  useBlueprintsList: (query?: unknown) => mockBlueprintsList(query),
  useAffixesList: (query?: unknown) => mockAffixesList(query),
  useGlobalMetaAttributesList: (query?: unknown) =>
    mockGlobalMetaAttributesList(query),
  useGenerate: () => mockGenerateMutation,
}))

const mockToast = vi.fn()

vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: mockToast, dismiss: vi.fn(), toasts: [] }),
}))

vi.mock('@/stores/ui', () => ({
  useUi: () => ({ selectedClientId: 'client-1' }),
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
        value_type: 'single',
        value: 0,
        distribution: { type: 'uniform' },
      },
      element: {
        value_type: 'enum',
        values: ['fire', 'ice', 'lightning'],
      },
      name: {
        value_type: 'string',
        minLength: 3,
        maxLength: 16,
      },
      legendary: {
        value_type: 'boolean',
        value: false,
      },
      price_range: {
        value_type: 'range',
        min: 1,
        max: 100,
      },
    },
    attribute_order: ['damage', 'element', 'name', 'legendary', 'price_range'],
    min_prefixes: 0,
    max_prefixes: 0,
    min_suffixes: 0,
    max_suffixes: 0,
    created_at: '2025-01-01T00:00:00Z',
    updated_at: '2025-01-15T12:00:00Z',
    ...overrides,
  }
}

function makeAffix(overrides: Record<string, unknown> = {}) {
  return {
    id: `aff-${overrides.id || '1'}`,
    client_id: 'client-1',
    name: `Affix ${overrides.id || '1'}`,
    location: 'prefix',
    description: null,
    attribute: {
      value_type: 'single',
      value: 0,
    },
    created_at: '2025-01-01T00:00:00Z',
    updated_at: '2025-01-15T12:00:00Z',
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

    mockGlobalMetaAttributesList.mockReturnValue({
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
      name_parts: { base: 'Sword', prefixes: ['Flaming'], suffixes: ['of Doom'] },
      blueprint_id: 'bp-1',
      blueprint_attributes: { damage: 50, element: 'fire' },
      affix_attributes: [],
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

    fireEvent.click(screen.getByRole('button', { name: /Remove constraint/i }))
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
      name_parts: { base: 'Item', prefixes: [], suffixes: [] },
      blueprint_id: 'bp-1',
      blueprint_attributes: {},
      affix_attributes: [],
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
      name_parts: { base: 'Item', prefixes: [], suffixes: [] },
      blueprint_id: 'bp-1',
      blueprint_attributes: {},
      affix_attributes: [],
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
      name_parts: { base: 'Blade', prefixes: ['Flaming'], suffixes: ['of Destiny'] },
      blueprint_id: 'bp-1',
      blueprint_attributes: { damage: 85, element: 'fire', durability: 100 },
      affix_attributes: [
        {
          affix_id: 'aff-1',
          affix_name: 'Flaming',
          fire_damage: 25,
        },
        {
          affix_id: 'aff-2',
          affix_name: 'Destiny',
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
      name_parts: { base: 'Test', prefixes: [], suffixes: [] },
      blueprint_id: 'bp-1',
      blueprint_attributes: {},
      affix_attributes: [],
    })

    renderDashboard()

    const trigger = screen.getByRole('combobox', { name: /Archetype/i })
    fireEvent.click(trigger)
    fireEvent.click(screen.getByText('weapon'))

    fireEvent.click(screen.getByText('Add constraint'))

    const seedInput = screen.getByPlaceholderText('Leave empty for random')
    fireEvent.change(seedInput, { target: { value: '42' } })

    const min_prefixes = screen.getByLabelText('Min Prefixes')
    fireEvent.change(min_prefixes, { target: { value: '2' } })

    fireEvent.click(screen.getByRole('button', { name: /Generate/i }))

    await waitFor(() => {
      expect(screen.getByText('Test')).toBeInTheDocument()
    })

    fireEvent.click(screen.getByRole('button', { name: /Clear/i }))

    await waitFor(() => {
      expect(screen.queryByText('Test')).not.toBeInTheDocument()
    })

    expect(seedInput).toHaveValue('')
    expect(min_prefixes).toHaveValue(0)
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
      name_parts: { base: 'Test', prefixes: [], suffixes: [] },
      blueprint_id: 'bp-1',
      blueprint_attributes: {},
      affix_attributes: [],
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
      name_parts: { base: 'Test', prefixes: [], suffixes: [] },
      blueprint_id: 'bp-1',
      blueprint_attributes: {},
      affix_attributes: [],
    })

    renderDashboard()

    const min_prefixes = screen.getByLabelText('Min Prefixes')
    fireEvent.change(min_prefixes, { target: { value: '1' } })

    const max_suffixes = screen.getByLabelText('Max Suffixes')
    fireEvent.change(max_suffixes, { target: { value: '2' } })

    const flamingCheckboxes = screen.getAllByRole('checkbox', { name: 'Flaming' })
    fireEvent.click(flamingCheckboxes[0])

    fireEvent.click(screen.getByRole('button', { name: /Generate/i }))

    await waitFor(() => {
      const call = mockGenerateMutation.mutateAsync.mock.calls[0][0]
      expect(call.affixes.min_prefixes).toBe(1)
      expect(call.affixes.max_suffixes).toBe(2)
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
      name_parts: { base: 'Test', prefixes: [], suffixes: [] },
      blueprint_id: 'bp-1',
      blueprint_attributes: {},
      affix_attributes: [],
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

describe('Dashboard Stat Cards', () => {
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

    mockGlobalMetaAttributesList.mockReturnValue({
      data: { data: [], total: 0 },
      isLoading: false,
      isError: false,
      error: null,
    })
  })

  it('renders three stat cards with correct counts', () => {
    mockBlueprintsList.mockReturnValue({
      data: { data: [], total: 15 },
      isLoading: false,
      isError: false,
      error: null,
    })
    mockAffixesList.mockReturnValue({
      data: { data: [], total: 8 },
      isLoading: false,
      isError: false,
      error: null,
    })
    mockGlobalMetaAttributesList.mockReturnValue({
      data: { data: [], total: 4 },
      isLoading: false,
      isError: false,
      error: null,
    })

    const { container } = renderDashboard()

    expect(screen.getByText('Total Blueprints')).toBeInTheDocument()
    expect(screen.getByText('Total Affixes')).toBeInTheDocument()
    expect(screen.getByText('Global Meta Attributes')).toBeInTheDocument()

    const statLinks = container.querySelectorAll('a.text-2xl')
    const statValues = Array.from(statLinks).map((el) => el.textContent?.trim())
    expect(statValues).toContain('15')
    expect(statValues).toContain('8')
    expect(statValues).toContain('4')
    expect(statLinks[0]).toHaveAttribute('href', '/blueprints')
    expect(statLinks[1]).toHaveAttribute('href', '/affixes')
    expect(statLinks[2]).toHaveAttribute('href', '/global-meta-attributes')
  })

  it('shows skeleton stat cards while loading', () => {
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

  it('shows global meta attributes count', () => {
    mockGlobalMetaAttributesList.mockReturnValue({
      data: { data: [], total: 7 },
      isLoading: false,
      isError: false,
      error: null,
    })

    renderDashboard()

    const gmaLink = screen.getByText('7')
    expect(gmaLink).toBeInTheDocument()
    expect(gmaLink.closest('a')).toHaveAttribute('href', '/global-meta-attributes')
  })
})

describe('Dashboard Warnings', () => {
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

    mockGlobalMetaAttributesList.mockReturnValue({
      data: { data: [], total: 0 },
      isLoading: false,
      isError: false,
      error: null,
    })
  })

  it('shows zero-weight blueprint warning', () => {
    mockBlueprintsList.mockReturnValue({
      data: {
        data: [makeBlueprint({ id: '1', weight: 0, name: 'ZeroWeight' })],
        total: 1,
      },
      isLoading: false,
      isError: false,
      error: null,
    })

    renderDashboard()

    expect(screen.getByText('Zero-weight blueprint')).toBeInTheDocument()
    expect(
      screen.getByText(/has a weight of 0/),
    ).toBeInTheDocument()
    expect(screen.getByText('View ZeroWeight')).toBeInTheDocument()
  })

  it('shows empty prefix pool warning', () => {
    mockBlueprintsList.mockReturnValue({
      data: {
        data: [
          makeBlueprint({
            id: '1',
            name: 'NeedsPrefix',
            min_prefixes: 2,
            max_prefixes: 3,
            prefixes: [],
            suffixes: [],
          }),
        ],
        total: 1,
      },
      isLoading: false,
      isError: false,
      error: null,
    })

    renderDashboard()

    expect(screen.getByText('Empty prefix pool')).toBeInTheDocument()
    expect(
      screen.getByText('View NeedsPrefix'),
    ).toBeInTheDocument()
  })

  it('shows insufficient suffix pool warning', () => {
    mockBlueprintsList.mockReturnValue({
      data: {
        data: [
          makeBlueprint({
            id: '1',
            name: 'NeedsSuffix',
            min_prefixes: 0,
            max_prefixes: 0,
            min_suffixes: 3,
            max_suffixes: 5,
            suffixes: [{ affix_id: 'aff-1', weight: 1 }],
          }),
        ],
        total: 1,
      },
      isLoading: false,
      isError: false,
      error: null,
    })

    renderDashboard()

    expect(screen.getByText('Insufficient suffix pool')).toBeInTheDocument()
    expect(
      screen.getByText('View NeedsSuffix'),
    ).toBeInTheDocument()
  })

  it('shows zero-weight pool entry warning', () => {
    mockBlueprintsList.mockReturnValue({
      data: {
        data: [
          makeBlueprint({
            id: '1',
            name: 'ZeroWeightAffix',
            min_prefixes: 1,
            max_prefixes: 1,
            prefixes: [{ affix_id: 'aff-x', weight: 0 }],
            suffixes: [],
          }),
        ],
        total: 1,
      },
      isLoading: false,
      isError: false,
      error: null,
    })

    renderDashboard()

    expect(screen.getByText('Zero-weight affix assignment')).toBeInTheDocument()
  })

  it('shows dangling $ref_id warning', () => {
    mockBlueprintsList.mockReturnValue({
      data: {
        data: [
          makeBlueprint({
            id: '1',
            name: 'DanglingRef',
            attributes: {
              power: { $ref_id: 'non-existent-global' },
            },
            attribute_order: ['power'],
          }),
        ],
        total: 1,
      },
      isLoading: false,
      isError: false,
      error: null,
    })
    mockAffixesList.mockReturnValue({
      data: {
        data: [
          makeAffix({
            id: '1',
            name: 'DanglingAffix',
            attribute: { $ref_id: 'also-missing' },
          }),
        ],
        total: 1,
      },
      isLoading: false,
      isError: false,
      error: null,
    })

    renderDashboard()

    const danglingItems = screen.getAllByText('Dangling `$ref_id`')
    expect(danglingItems.length).toBe(2)
    expect(screen.getByText('View DanglingRef')).toBeInTheDocument()
  })

  it('shows invalid range attribute warning (min > max)', () => {
    mockBlueprintsList.mockReturnValue({
      data: {
        data: [
          makeBlueprint({
            id: '1',
            name: 'BadRange',
            attributes: {
              damage: {
                value_type: 'range',
                min: 100,
                max: 10,
              },
            },
            attribute_order: ['damage'],
          }),
        ],
        total: 1,
      },
      isLoading: false,
      isError: false,
      error: null,
    })

    renderDashboard()

    expect(screen.getByText('Invalid range attribute')).toBeInTheDocument()
    expect(
      screen.getByText('View BadRange'),
    ).toBeInTheDocument()
  })

  it('shows empty enum attribute warning', () => {
    mockBlueprintsList.mockReturnValue({
      data: {
        data: [
          makeBlueprint({
            id: '1',
            name: 'BadEnum',
            attributes: {
              element: {
                value_type: 'enum',
                values: [],
              },
            },
            attribute_order: ['element'],
          }),
        ],
        total: 1,
      },
      isLoading: false,
      isError: false,
      error: null,
    })

    renderDashboard()

    expect(screen.getByText('Empty enum attribute')).toBeInTheDocument()
    expect(
      screen.getByText('View BadEnum'),
    ).toBeInTheDocument()
  })

  it('shows invalid distribution config warning (stdDev = 0)', () => {
    mockBlueprintsList.mockReturnValue({
      data: {
        data: [
          makeBlueprint({
            id: '1',
            name: 'BadDist',
            attributes: {
              damage: {
                value_type: 'single',
                value: 0,
                distribution: { type: 'normal', stdDev: 0 },
              },
            },
            attribute_order: ['damage'],
          }),
        ],
        total: 1,
      },
      isLoading: false,
      isError: false,
      error: null,
    })

    renderDashboard()

    expect(screen.getByText('Invalid distribution config')).toBeInTheDocument()
    expect(
      screen.getByText('View BadDist'),
    ).toBeInTheDocument()
  })

  it('shows invalid distribution config warning (rate = 0)', () => {
    mockBlueprintsList.mockReturnValue({
      data: {
        data: [
          makeBlueprint({
            id: '1',
            name: 'BadRate',
            attributes: {
              damage: {
                value_type: 'single',
                value: 0,
                distribution: { type: 'exponential', rate: 0 },
              },
            },
            attribute_order: ['damage'],
          }),
        ],
        total: 1,
      },
      isLoading: false,
      isError: false,
      error: null,
    })

    renderDashboard()

    expect(screen.getByText('Invalid distribution config')).toBeInTheDocument()
    expect(
      screen.getByText('View BadRate'),
    ).toBeInTheDocument()
  })

  it('shows "No warnings" message when there are no warnings', () => {
    mockBlueprintsList.mockReturnValue({
      data: {
        data: [makeBlueprint({ id: '1', weight: 1 })],
        total: 1,
      },
      isLoading: false,
      isError: false,
      error: null,
    })

    renderDashboard()

    expect(
      screen.getByText(/No warnings found/),
    ).toBeInTheDocument()
  })

  it('allows dismissing a warning', () => {
    mockBlueprintsList.mockReturnValue({
      data: {
        data: [makeBlueprint({ id: '1', weight: 0, name: 'ZeroWeight' })],
        total: 1,
      },
      isLoading: false,
      isError: false,
      error: null,
    })

    renderDashboard()

    expect(screen.getByText('Zero-weight blueprint')).toBeInTheDocument()

    const dismissBtn = screen.getByRole('button', { name: 'Dismiss warning' })
    fireEvent.click(dismissBtn)

    expect(screen.queryByText('Zero-weight blueprint')).not.toBeInTheDocument()
  })

  it('shows warning skeleton while loading', () => {
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
    mockGlobalMetaAttributesList.mockReturnValue({
      data: undefined,
      isLoading: true,
      isError: false,
      error: null,
    })

    renderDashboard()

    const skeletons = document.querySelectorAll('.animate-pulse')
    expect(skeletons.length).toBeGreaterThan(0)
  })

  it('each warning has a link to the relevant resource', () => {
    mockBlueprintsList.mockReturnValue({
      data: {
        data: [makeBlueprint({ id: 'bp-abc', weight: 0, name: 'ZeroWeight' })],
        total: 1,
      },
      isLoading: false,
      isError: false,
      error: null,
    })

    renderDashboard()

    const viewLink = screen.getByText('View ZeroWeight')
    expect(viewLink).toBeInTheDocument()
    expect(viewLink.closest('a')).toHaveAttribute(
      'href',
      '/blueprints/bp-abc',
    )
  })
})
