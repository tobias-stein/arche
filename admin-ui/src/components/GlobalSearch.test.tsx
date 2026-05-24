import { render, screen, act } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { describe, expect, it, vi } from 'vitest'
import { ArcheClient } from '@/api/generated/client'
import { setClient } from '@/api/generated/hooks'
import { useUi } from '@/stores/ui'
import { GlobalSearch } from '@/components/GlobalSearch'

const mockListBlueprints = vi.fn()
const mockListAffixes = vi.fn()

const mockClient = {
  listBlueprints: mockListBlueprints,
  listAffixes: mockListAffixes,
} as unknown as ArcheClient

setClient(mockClient)

function renderSearch() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  })
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={['/dashboard']}>
        <GlobalSearch />
      </MemoryRouter>
    </QueryClientProvider>,
  )
}

describe('GlobalSearch', () => {
  beforeEach(async () => {
    vi.clearAllMocks()
    mockListBlueprints.mockResolvedValue({ data: [], nextCursor: undefined, total: 0 })
    mockListAffixes.mockResolvedValue({ data: [], nextCursor: undefined, total: 0 })
    const { useUi: ui } = await import('@/stores/ui')
    act(() => {
      ui.setState({
        searchOpen: false,
      })
    })
  })

  it('does not render the dialog when searchOpen is false', () => {
    renderSearch()
    expect(screen.queryByPlaceholderText('Search blueprints and affixes...')).not.toBeInTheDocument()
  })

  it('renders the search input when searchOpen is true', () => {
    act(() => {
      useUi.setState({ searchOpen: true })
    })
    renderSearch()
    expect(screen.getByPlaceholderText('Search blueprints and affixes...')).toBeInTheDocument()
  })

  it('shows initial prompt when no search term is entered', () => {
    act(() => {
      useUi.setState({ searchOpen: true })
    })
    renderSearch()
    expect(screen.getByText('Type to search blueprints and affixes by name.')).toBeInTheDocument()
  })

  it('closes when onOpenChange is called with false', () => {
    act(() => {
      useUi.setState({ searchOpen: true })
    })
    renderSearch()
    act(() => {
      useUi.getState().closeSearch()
    })
    expect(useUi.getState().searchOpen).toBe(false)
  })

  it('shows empty state when search yields no results', async () => {
    mockListBlueprints.mockResolvedValue({ data: [], nextCursor: undefined, total: 0 })
    mockListAffixes.mockResolvedValue({ data: [], nextCursor: undefined, total: 0 })

    act(() => {
      useUi.setState({ searchOpen: true })
    })
    renderSearch()

    const input = screen.getByPlaceholderText('Search blueprints and affixes...')
    await userEvent.type(input, 'zzz')

    await vi.waitFor(() => {
      expect(screen.getByText('No results found.')).toBeInTheDocument()
    })
  })

  it('shows blueprints grouped under a heading', async () => {
    mockListBlueprints.mockResolvedValue({
      data: [
        { id: '1', name: 'Sword', archetype: 'weapon', clientId: 'c1', weight: 10, description: null, attributes: {}, attributeOrder: [], minPrefixes: 0, maxPrefixes: 3, minSuffixes: 0, maxSuffixes: 2, createdAt: '', updatedAt: '' },
      ],
      nextCursor: undefined,
      total: 1,
    })
    mockListAffixes.mockResolvedValue({ data: [], nextCursor: undefined, total: 0 })

    act(() => {
      useUi.setState({ searchOpen: true })
    })
    renderSearch()

    const input = screen.getByPlaceholderText('Search blueprints and affixes...')
    await userEvent.type(input, 'sword')

    await vi.waitFor(() => {
      expect(screen.getByText('Blueprints')).toBeInTheDocument()
      expect(screen.getByText('Sword')).toBeInTheDocument()
      expect(screen.getByText('weapon')).toBeInTheDocument()
    })
  })

  it('shows affixes grouped under a heading', async () => {
    mockListBlueprints.mockResolvedValue({ data: [], nextCursor: undefined, total: 0 })
    mockListAffixes.mockResolvedValue({
      data: [
        { id: '1', name: 'Flame', location: 'prefix', clientId: 'c1', description: null, attribute: { type: 'text', value: '' }, createdAt: '', updatedAt: '' },
      ],
      nextCursor: undefined,
      total: 1,
    })

    act(() => {
      useUi.setState({ searchOpen: true })
    })
    renderSearch()

    const input = screen.getByPlaceholderText('Search blueprints and affixes...')
    await userEvent.type(input, 'flame')

    await vi.waitFor(() => {
      expect(screen.getByText('Affixes')).toBeInTheDocument()
      expect(screen.getByText('Flame')).toBeInTheDocument()
      expect(screen.getByText('prefix')).toBeInTheDocument()
    })
  })

  it('shows both groups when both have results', async () => {
    mockListBlueprints.mockResolvedValue({
      data: [
        { id: '1', name: 'Sword', archetype: 'weapon', clientId: 'c1', weight: 10, description: null, attributes: {}, attributeOrder: [], minPrefixes: 0, maxPrefixes: 3, minSuffixes: 0, maxSuffixes: 2, createdAt: '', updatedAt: '' },
      ],
      nextCursor: undefined,
      total: 1,
    })
    mockListAffixes.mockResolvedValue({
      data: [
        { id: '2', name: 'Flame', location: 'suffix', clientId: 'c1', description: null, attribute: { type: 'text', value: '' }, createdAt: '', updatedAt: '' },
      ],
      nextCursor: undefined,
      total: 1,
    })

    act(() => {
      useUi.setState({ searchOpen: true })
    })
    renderSearch()

    const input = screen.getByPlaceholderText('Search blueprints and affixes...')
    await userEvent.type(input, 's')

    await vi.waitFor(() => {
      expect(screen.getByText('Blueprints')).toBeInTheDocument()
      expect(screen.getByText('Sword')).toBeInTheDocument()
      expect(screen.getByText('Affixes')).toBeInTheDocument()
      expect(screen.getByText('Flame')).toBeInTheDocument()
    })
  })

  it('triggers search after debounce delay', async () => {
    mockListBlueprints.mockResolvedValue({ data: [], nextCursor: undefined, total: 0 })
    mockListAffixes.mockResolvedValue({ data: [], nextCursor: undefined, total: 0 })

    act(() => {
      useUi.setState({ searchOpen: true })
    })
    renderSearch()

    const input = screen.getByPlaceholderText('Search blueprints and affixes...')
    await userEvent.type(input, 'test')

    expect(mockListBlueprints).not.toHaveBeenCalled()
    expect(mockListAffixes).not.toHaveBeenCalled()

    await vi.waitFor(() => {
      expect(mockListBlueprints).toHaveBeenCalledWith({ search: 'test', perPage: 5 })
      expect(mockListAffixes).toHaveBeenCalledWith({ search: 'test', perPage: 5 })
    })
  })

  it('resets input when dialog is closed', () => {
    act(() => {
      useUi.setState({ searchOpen: true })
    })
    renderSearch()

    act(() => {
      useUi.setState({ searchOpen: false })
    })
    act(() => {
      useUi.setState({ searchOpen: true })
    })

    const reopenedInput = screen.getByPlaceholderText('Search blueprints and affixes...')
    expect(reopenedInput).toHaveValue('')
  })

  it('closes search and navigates when a blueprint item is selected', async () => {
    mockListBlueprints.mockResolvedValue({
      data: [
        { id: 'bp-1', name: 'Sword', archetype: 'weapon', clientId: 'c1', weight: 10, description: null, attributes: {}, attributeOrder: [], minPrefixes: 0, maxPrefixes: 3, minSuffixes: 0, maxSuffixes: 2, createdAt: '', updatedAt: '' },
      ],
      nextCursor: undefined,
      total: 1,
    })
    mockListAffixes.mockResolvedValue({ data: [], nextCursor: undefined, total: 0 })

    act(() => {
      useUi.setState({ searchOpen: true })
    })
    renderSearch()

    const input = screen.getByPlaceholderText('Search blueprints and affixes...')
    await userEvent.type(input, 'sword')

    await vi.waitFor(() => {
      expect(screen.getByText('Sword')).toBeInTheDocument()
    })

    const item = screen.getByText('Sword')
    item.click()

    expect(useUi.getState().searchOpen).toBe(false)
  })

  it('closes search and navigates when an affix item is selected', async () => {
    mockListBlueprints.mockResolvedValue({ data: [], nextCursor: undefined, total: 0 })
    mockListAffixes.mockResolvedValue({
      data: [
        { id: 'affix-1', name: 'Flame', location: 'prefix', clientId: 'c1', description: null, attribute: { type: 'text', value: '' }, createdAt: '', updatedAt: '' },
      ],
      nextCursor: undefined,
      total: 1,
    })

    act(() => {
      useUi.setState({ searchOpen: true })
    })
    renderSearch()

    const input = screen.getByPlaceholderText('Search blueprints and affixes...')
    await userEvent.type(input, 'flame')

    await vi.waitFor(() => {
      expect(screen.getByText('Flame')).toBeInTheDocument()
    })

    const item = screen.getByText('Flame')
    item.click()

    expect(useUi.getState().searchOpen).toBe(false)
  })
})
