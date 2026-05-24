import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { describe, expect, it, vi } from 'vitest'
import { MemoryRouter } from 'react-router-dom'
import { ArcheClient } from '@/api/generated/client'
import { setClient } from '@/api/generated/hooks'
import { BlueprintFormModal } from '@/components/BlueprintFormModal'
import type { Blueprint } from '@/api/generated'

setClient(new ArcheClient({ baseUrl: 'http://localhost:3000' }))

function renderModal(blueprint?: Blueprint | null, duplicateFrom?: Blueprint | null) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  const onOpenChange = vi.fn()
  const result = render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter>
        <BlueprintFormModal
          open={true}
          onOpenChange={onOpenChange}
          blueprint={blueprint ?? null}
          duplicateFrom={duplicateFrom ?? null}
        />
      </MemoryRouter>
    </QueryClientProvider>,
  )
  return { ...result, onOpenChange }
}

function createMockBlueprint(): Blueprint {
  return {
    id: 'bp-1',
    clientId: 'client-1',
    name: 'Test BP',
    archetype: 'weapon',
    weight: 5,
    description: 'test',
    attributes: {
      damage: { valueType: 'single', value: 10 },
    },
    attributeOrder: ['damage'],
    minPrefixes: 1,
    maxPrefixes: 3,
    minSuffixes: 0,
    maxSuffixes: 2,
    createdAt: '2026-01-01T00:00:00Z',
    updatedAt: '2026-01-01T00:00:00Z',
  }
}

describe('BlueprintFormModal', () => {
  beforeEach(async () => {
    const { useTheme } = await import('@/stores/theme')
    useTheme.setState({ mode: 'light' })
  })

  describe('Create mode', () => {
    it('renders with "Create Blueprint" title', () => {
      renderModal()
      expect(
        screen.getByRole('heading', { name: 'Create Blueprint' }),
      ).toBeInTheDocument()
    })

    it('renders General tab fields', () => {
      renderModal()
      expect(screen.getByLabelText('Name *')).toBeInTheDocument()
      expect(screen.getByLabelText('Archetype *')).toBeInTheDocument()
      expect(screen.getByLabelText('Weight *')).toBeInTheDocument()
    })

    it('renders tab triggers for General, Attributes, Affixes', () => {
      renderModal()
      expect(
        screen.getByRole('tab', { name: 'General' }),
      ).toBeInTheDocument()
      expect(
        screen.getByRole('tab', { name: 'Attributes' }),
      ).toBeInTheDocument()
      expect(
        screen.getByRole('tab', { name: 'Affixes' }),
      ).toBeInTheDocument()
    })

    it('has Cancel and Create Blueprint buttons', () => {
      renderModal()
      expect(screen.getByText('Cancel')).toBeInTheDocument()
      expect(
        screen.getByRole('button', { name: 'Create Blueprint' }),
      ).toBeInTheDocument()
    })
  })

  describe('Edit mode', () => {
    it('renders with "Edit Blueprint" title', () => {
      renderModal(createMockBlueprint())
      expect(
        screen.getByRole('heading', { name: 'Edit Blueprint' }),
      ).toBeInTheDocument()
    })

    it('pre-fills form fields with blueprint data', () => {
      renderModal(createMockBlueprint())
      expect(screen.getByDisplayValue('Test BP')).toBeInTheDocument()
      expect(screen.getByDisplayValue('weapon')).toBeInTheDocument()
    })

    it('shows Save Changes button', () => {
      renderModal(createMockBlueprint())
      expect(
        screen.getByRole('button', { name: 'Save Changes' }),
      ).toBeInTheDocument()
    })
  })

  describe('Duplicate mode', () => {
    function createDuplicateMock() {
      const bp = createMockBlueprint()
      return { ...bp, name: 'Test BP (Copy)' }
    }

    it('renders with "Duplicate Blueprint" title', () => {
      renderModal(null, createDuplicateMock())
      expect(
        screen.getByRole('heading', { name: 'Duplicate Blueprint' }),
      ).toBeInTheDocument()
    })

    it('pre-fills form fields with source blueprint data', () => {
      renderModal(null, createDuplicateMock())
      expect(screen.getByDisplayValue('Test BP (Copy)')).toBeInTheDocument()
      expect(screen.getByDisplayValue('weapon')).toBeInTheDocument()
    })

    it('shows Create Blueprint button (not Save Changes)', () => {
      renderModal(null, createDuplicateMock())
      expect(
        screen.getByRole('button', { name: 'Create Blueprint' }),
      ).toBeInTheDocument()
      expect(
        screen.queryByRole('button', { name: 'Save Changes' }),
      ).not.toBeInTheDocument()
    })

    it('pre-fills affix counts from source', async () => {
      const user = userEvent.setup()
      renderModal(null, createDuplicateMock())
      await user.click(screen.getByRole('tab', { name: 'Affixes' }))
      expect(screen.getByDisplayValue('1')).toBeInTheDocument()
      expect(screen.getByDisplayValue('3')).toBeInTheDocument()
      expect(screen.getByDisplayValue('0')).toBeInTheDocument()
      expect(screen.getByDisplayValue('2')).toBeInTheDocument()
    })
  })

  describe('Tab navigation', () => {
    it('shows Add Attribute button when switching to Attributes tab', async () => {
      const user = userEvent.setup()
      renderModal()
      await user.click(screen.getByRole('tab', { name: 'Attributes' }))
      expect(
        screen.getByRole('button', { name: /Add Attribute/i }),
      ).toBeInTheDocument()
    })

    it('shows Affixes tab content', async () => {
      const user = userEvent.setup()
      renderModal()
      await user.click(screen.getByRole('tab', { name: 'Affixes' }))
      expect(screen.getByText('Prefix Counts')).toBeInTheDocument()
      expect(screen.getByText('Suffix Counts')).toBeInTheDocument()
    })
  })
})
