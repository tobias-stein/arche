import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { describe, expect, it, vi } from 'vitest'
import { ArcheClient } from '@/api/generated/client'
import { setClient } from '@/api/generated/hooks'
import {
  AffixCreateEditDialog,
} from '@/pages/AffixCreateEditDialog'
import type { Affix } from '@/api/generated/types'

setClient(new ArcheClient({ baseUrl: 'http://localhost:3000' }))

const mockAffix: Affix = {
  id: 'aff-1',
  clientId: 'client-1',
  name: 'Flame',
  location: 'prefix',
  description: 'Adds fire damage',
  attribute: {
    name: 'damage',
    value_type: 'single',
    value: 10,
  },
  createdAt: '2024-01-01T00:00:00Z',
  updatedAt: '2024-01-01T00:00:00Z',
}

const mockAffixWithGlobal: Affix = {
  id: 'aff-2',
  clientId: 'client-1',
  name: 'Ice',
  location: 'suffix',
  description: null,
  attribute: { $ref_id: 'gma-1' },
  createdAt: '2024-01-01T00:00:00Z',
  updatedAt: '2024-01-01T00:00:00Z',
}

function renderDialog(
  props: Partial<{
    affix: Affix | null
    open: boolean
    onSubmit: () => Promise<void>
    isPending: boolean
  }>,
) {
  const onOpenChange = vi.fn()
  const onSubmit = props.onSubmit ?? vi.fn().mockResolvedValue(undefined)
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })

  render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter>
        <AffixCreateEditDialog
          affix={props.affix === undefined ? null : props.affix}
          open={props.open ?? true}
          onOpenChange={onOpenChange}
          onSubmit={onSubmit}
          isPending={props.isPending ?? false}
        />
      </MemoryRouter>
    </QueryClientProvider>,
  )

  return { onOpenChange, onSubmit }
}

describe('AffixCreateEditDialog', () => {
  describe('create mode', () => {
    it('shows "Create Affix" title', () => {
      renderDialog({})
      expect(screen.getByRole('heading', { name: 'Create Affix' })).toBeInTheDocument()
    })

    it('renders name field', () => {
      renderDialog({})
      expect(screen.getByLabelText('Name')).toBeInTheDocument()
    })

    it('renders type selector with Prefix default', () => {
      renderDialog({})
      const typeSelect = screen.getByLabelText('Type') as HTMLSelectElement
      expect(typeSelect.value).toBe('prefix')
    })

    it('renders description field', () => {
      renderDialog({})
      expect(screen.getByLabelText('Description')).toBeInTheDocument()
    })

    it('renders attribute source toggle (Inline / From Library)', () => {
      renderDialog({})
      expect(screen.getByText('Inline')).toBeInTheDocument()
      expect(screen.getByText('From Library')).toBeInTheDocument()
    })

    it('shows inline attribute form by default', () => {
      renderDialog({})
      expect(screen.getByLabelText('Attribute Name')).toBeInTheDocument()
      expect(screen.getByLabelText('Value Type')).toBeInTheDocument()
    })

    it('switches to global attribute picker when From Library is clicked', () => {
      renderDialog({})
      fireEvent.click(screen.getByText('From Library'))
      expect(screen.getByText('Select a global attribute...')).toBeInTheDocument()
    })

    it('switches back to inline form when Inline is clicked', () => {
      renderDialog({})
      fireEvent.click(screen.getByText('From Library'))
      fireEvent.click(screen.getByText('Inline'))
      expect(screen.getByLabelText('Attribute Name')).toBeInTheDocument()
    })

    it('has Cancel button', () => {
      renderDialog({})
      expect(screen.getByRole('button', { name: 'Cancel' })).toBeInTheDocument()
    })

    it('has Create button', () => {
      renderDialog({})
      expect(screen.getByRole('button', { name: 'Create' })).toBeInTheDocument()
    })

    it('disables Create button when name is empty', () => {
      renderDialog({})
      expect(screen.getByRole('button', { name: 'Create' })).toBeDisabled()
    })
  })

  describe('edit mode', () => {
    it('shows "Edit Affix" title', () => {
      renderDialog({ affix: mockAffix })
      expect(screen.getByRole('heading', { name: 'Edit Affix' })).toBeInTheDocument()
    })

    it('pre-fills name field', () => {
      renderDialog({ affix: mockAffix })
      const nameInput = screen.getByLabelText('Name') as HTMLInputElement
      expect(nameInput.value).toBe('Flame')
    })

    it('pre-fills type selector', () => {
      renderDialog({ affix: mockAffix })
      const typeSelect = screen.getByLabelText('Type') as HTMLSelectElement
      expect(typeSelect.value).toBe('prefix')
    })

    it('pre-fills description', () => {
      renderDialog({ affix: mockAffix })
      const descInput = screen.getByLabelText('Description') as HTMLInputElement
      expect(descInput.value).toBe('Adds fire damage')
    })

    it('shows inline attribute source for inline affix', () => {
      renderDialog({ affix: mockAffix })
      expect(screen.getByLabelText('Attribute Name')).toBeInTheDocument()
    })

    it('shows global attribute source for global ref affix', () => {
      renderDialog({ affix: mockAffixWithGlobal })
      const fromLibBtn = screen.getByText('From Library')
      expect(fromLibBtn.className).toContain('bg-primary')
    })

    it('has Save button', () => {
      renderDialog({ affix: mockAffix })
      expect(screen.getByRole('button', { name: 'Save' })).toBeInTheDocument()
    })
  })

  describe('submit', () => {
    it('calls onSubmit when Save is clicked with valid data', async () => {
      const onSubmit = vi.fn().mockResolvedValue(undefined)
      renderDialog({ affix: mockAffix, onSubmit })
      fireEvent.click(screen.getByRole('button', { name: 'Save' }))
      await waitFor(() => {
        expect(onSubmit).toHaveBeenCalledTimes(1)
      })
    })

    it('does not call onSubmit when Create is clicked with empty name', () => {
      const onSubmit = vi.fn().mockResolvedValue(undefined)
      renderDialog({ onSubmit })
      const createBtn = screen.getByRole('button', { name: 'Create' })
      expect(createBtn).toBeDisabled()
      expect(onSubmit).not.toHaveBeenCalled()
    })

    it('calls onSubmit with Create data when form filled in create mode', async () => {
      const onSubmit = vi.fn().mockResolvedValue(undefined)
      renderDialog({ onSubmit })
      const nameInput = screen.getByLabelText('Name') as HTMLInputElement
      fireEvent.change(nameInput, { target: { value: 'New Affix' } })
      const attrNameInput = screen.getByLabelText('Attribute Name') as HTMLInputElement
      fireEvent.change(attrNameInput, { target: { value: 'damage' } })
      const valueInput = screen.getByLabelText('Value') as HTMLInputElement
      fireEvent.change(valueInput, { target: { value: '10' } })
      await waitFor(() => {
        expect(screen.getByRole('button', { name: 'Create' })).not.toBeDisabled()
      })
      fireEvent.click(screen.getByRole('button', { name: 'Create' }))
      await waitFor(() => {
        expect(onSubmit).toHaveBeenCalledTimes(1)
      })
    })

    it('shows "Saving..." when isPending is true', () => {
      renderDialog({ affix: mockAffix, isPending: true })
      expect(screen.getByRole('button', { name: 'Saving...' })).toBeInTheDocument()
    })
  })

  describe('inline attribute form', () => {
    it('shows value type options', () => {
      renderDialog({})
      const vtSelect = screen.getByLabelText('Value Type') as HTMLSelectElement
      expect(vtSelect.value).toBe('single')
      const options = Array.from(vtSelect.options).map((o) => o.value)
      expect(options).toEqual(['single', 'enum', 'range', 'string', 'boolean'])
    })

    it('shows value input when type is single', () => {
      renderDialog({})
      expect(screen.getByLabelText('Value')).toBeInTheDocument()
    })

    it('shows values input when type is enum', () => {
      renderDialog({})
      const vtSelect = screen.getByLabelText('Value Type') as HTMLSelectElement
      fireEvent.change(vtSelect, { target: { value: 'enum' } })
      expect(screen.getByLabelText('Values (comma-separated)')).toBeInTheDocument()
    })

    it('shows min/max inputs when type is range', () => {
      renderDialog({})
      const vtSelect = screen.getByLabelText('Value Type') as HTMLSelectElement
      fireEvent.change(vtSelect, { target: { value: 'range' } })
      expect(screen.getByLabelText('Min')).toBeInTheDocument()
      expect(screen.getByLabelText('Max')).toBeInTheDocument()
    })

    it('shows boolean radio when type is boolean', () => {
      renderDialog({})
      const vtSelect = screen.getByLabelText('Value Type') as HTMLSelectElement
      fireEvent.change(vtSelect, { target: { value: 'boolean' } })
      expect(screen.getByText('True')).toBeInTheDocument()
      expect(screen.getByText('False')).toBeInTheDocument()
    })

    it('shows distribution toggle for single type', () => {
      renderDialog({})
      expect(screen.getByText('Enable Distribution')).toBeInTheDocument()
    })

    it('shows distribution toggle for range type', () => {
      renderDialog({})
      const vtSelect = screen.getByLabelText('Value Type') as HTMLSelectElement
      fireEvent.change(vtSelect, { target: { value: 'range' } })
      expect(screen.getByText('Enable Distribution')).toBeInTheDocument()
    })

    it('shows distribution type selector when enabled', () => {
      renderDialog({})
      fireEvent.click(screen.getByText('Enable Distribution'))
      expect(screen.getByLabelText('Distribution Type')).toBeInTheDocument()
    })
  })
})
