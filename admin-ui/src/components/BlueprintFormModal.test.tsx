import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { MemoryRouter } from 'react-router-dom'
import { ArcheClient } from '@/api/generated/client'
import { setClient } from '@/api/generated/hooks'
import { BlueprintFormModal } from '@/components/BlueprintFormModal'
import type { Blueprint, AffixPoolEntry } from '@/api/generated'

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
      damage: { value_type: 'single', value: 10 },
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

  describe('Dialog accessibility', () => {
    it('renders DialogDescription in create mode', () => {
      renderModal()
      expect(
        screen.getByText('Fill in the details below to define a new blueprint.'),
      ).toBeInTheDocument()
    })

    it('renders DialogDescription in edit mode', () => {
      renderModal(createMockBlueprint())
      expect(
        screen.getByText('Modify the blueprint configuration below.'),
      ).toBeInTheDocument()
    })
  })

  describe('Weight validation', () => {
    it('weight input has min=1 attribute', () => {
      renderModal()
      const weightInput = screen.getByLabelText('Weight *')
      expect(weightInput).toHaveAttribute('min', '1')
    })
  })

  describe('Edit mode pool entries', () => {
    function createBlueprintWithPools(): Blueprint {
      const bp = createMockBlueprint()
      const extended = bp as Blueprint & {
        prefixes?: AffixPoolEntry[]
        suffixes?: AffixPoolEntry[]
      }
      extended.prefixes = [{ affixId: 'aff-a', weight: 7 }]
      extended.suffixes = [{ affixId: 'aff-b', weight: 2 }]
      return extended
    }

    it('populates prefix pool from extended blueprint data', async () => {
      const user = userEvent.setup()
      renderModal(createBlueprintWithPools())
      await user.click(screen.getByRole('tab', { name: 'Affixes' }))
      expect(screen.getByDisplayValue('7')).toBeInTheDocument()
    })

    it('shows empty pools when blueprint has no pool data', async () => {
      const user = userEvent.setup()
      renderModal(createMockBlueprint())
      await user.click(screen.getByRole('tab', { name: 'Affixes' }))
      expect(screen.getByText('No prefixes in the pool')).toBeInTheDocument()
      expect(screen.getByText('No suffixes in the pool')).toBeInTheDocument()
    })
  })

  describe('Last attribute deletion prevention', () => {
    it('prevents deleting the last attribute', async () => {
      const user = userEvent.setup()
      renderModal(createMockBlueprint())
      await user.click(screen.getByRole('tab', { name: 'Attributes' }))
      await user.click(screen.getByLabelText('Delete attribute damage'))
      expect(screen.getByText('Cannot Delete')).toBeInTheDocument()
    })
  })

  describe('Inline attribute form', () => {
    it('shows inline attribute form when "Inline Attribute" is clicked', async () => {
      const user = userEvent.setup()
      renderModal()
      await user.click(screen.getByRole('tab', { name: 'Attributes' }))
      await user.click(screen.getByRole('button', { name: /Add Attribute/i }))
      await user.click(screen.getByText('Inline Attribute'))
      expect(screen.getByLabelText('Key')).toBeInTheDocument()
      expect(screen.getByLabelText('Value Type')).toBeInTheDocument()
      expect(screen.getByRole('button', { name: 'Add' })).toBeInTheDocument()
    })

    it('shows value input when type is single', async () => {
      const user = userEvent.setup()
      renderModal()
      await user.click(screen.getByRole('tab', { name: 'Attributes' }))
      await user.click(screen.getByRole('button', { name: /Add Attribute/i }))
      await user.click(screen.getByText('Inline Attribute'))
      expect(screen.getByLabelText('Value')).toBeInTheDocument()
    })
  })

  describe('Affixes tab pools', () => {
    it('shows Add Prefix and Add Suffix buttons', async () => {
      const user = userEvent.setup()
      renderModal()
      await user.click(screen.getByRole('tab', { name: 'Affixes' }))
      expect(
        screen.getByRole('button', { name: 'Add Prefix' }),
      ).toBeInTheDocument()
      expect(
        screen.getByRole('button', { name: 'Add Suffix' }),
      ).toBeInTheDocument()
    })

    it('shows min/max prefix and suffix inputs', async () => {
      const user = userEvent.setup()
      renderModal()
      await user.click(screen.getByRole('tab', { name: 'Affixes' }))
      expect(screen.getByLabelText('Min Prefixes')).toBeInTheDocument()
      expect(screen.getByLabelText('Max Prefixes')).toBeInTheDocument()
      expect(screen.getByLabelText('Min Suffixes')).toBeInTheDocument()
      expect(screen.getByLabelText('Max Suffixes')).toBeInTheDocument()
    })
  })

  describe('Form validation', () => {
    it('shows inline validation errors when submitting empty form', async () => {
      const user = userEvent.setup()
      renderModal()
      await user.click(screen.getByRole('button', { name: 'Create Blueprint' }))
      await waitFor(() => {
        expect(screen.getByText('Name is required')).toBeInTheDocument()
      })
    })
  })
})
