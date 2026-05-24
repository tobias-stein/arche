import { fireEvent, render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { describe, expect, it, vi } from 'vitest'

import {
  DeleteResourceDialog,
  type ReferencingResource,
} from '@/components/delete-resource-dialog'

function renderDialog(props: Partial<{
  resourceName: string
  resourceType: 'blueprint' | 'affix' | 'global meta attribute' | 'client'
  referencingResources: ReferencingResource[]
  onForceDelete: () => void
  isDeleting: boolean
}>) {
  const onOpenChange = vi.fn()
  const onForceDelete = props.onForceDelete ?? vi.fn()

  render(
    <MemoryRouter>
      <DeleteResourceDialog
        open={true}
        onOpenChange={onOpenChange}
        resourceName={props.resourceName ?? 'TestResource'}
        resourceType={props.resourceType ?? 'blueprint'}
        referencingResources={props.referencingResources ?? []}
        onForceDelete={onForceDelete}
        isDeleting={props.isDeleting ?? false}
      />
    </MemoryRouter>,
  )

  return { onOpenChange, onForceDelete }
}

describe('DeleteResourceDialog', () => {
  it('shows resource name and type in the title', () => {
    renderDialog({ resourceName: 'SwordOfPower', resourceType: 'blueprint' })

    expect(screen.getByRole('heading', { name: 'Delete blueprint' })).toBeInTheDocument()
    expect(screen.getByText(/Are you sure you want to delete/)).toBeInTheDocument()
    expect(screen.getByText(/SwordOfPower/)).toBeInTheDocument()
  })

  it('shows Cancel button', () => {
    renderDialog({})

    expect(screen.getByRole('button', { name: 'Cancel' })).toBeInTheDocument()
  })

  it('shows Delete button (not Force Delete) when no references exist', () => {
    renderDialog({ referencingResources: [] })

    expect(screen.getByRole('button', { name: 'Delete' })).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Force Delete' })).not.toBeInTheDocument()
  })

  it('shows Force Delete button when references exist', () => {
    renderDialog({
      referencingResources: [
        { id: 'bp1', name: 'Iron Sword', type: 'blueprint', detailUrl: '/blueprints/bp1' },
      ],
    })

    expect(screen.getByRole('button', { name: 'Force Delete' })).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Delete' })).not.toBeInTheDocument()
  })

  it('shows warning text with reference count', () => {
    renderDialog({
      referencingResources: [
        { id: 'bp1', name: 'Iron Sword', type: 'blueprint', detailUrl: '/blueprints/bp1' },
        { id: 'bp2', name: 'Steel Axe', type: 'blueprint', detailUrl: '/blueprints/bp2' },
      ],
    })

    expect(screen.getByText(/This action cannot be undone/)).toBeInTheDocument()
    expect(screen.getByText(/2 resources will be modified/)).toBeInTheDocument()
  })

  it('uses singular form for single reference', () => {
    renderDialog({
      referencingResources: [
        { id: 'bp1', name: 'Iron Sword', type: 'blueprint', detailUrl: '/blueprints/bp1' },
      ],
    })

    expect(screen.getByText(/1 resource will be modified/)).toBeInTheDocument()
  })

  it('lists referencing resources with links', () => {
    renderDialog({
      referencingResources: [
        { id: 'bp1', name: 'Iron Sword', type: 'blueprint', detailUrl: '/blueprints/bp1' },
        { id: 'aff1', name: 'Flame', type: 'affix', detailUrl: '/affixes/aff1' },
      ],
    })

    const links = screen.getAllByRole('link')
    expect(links).toHaveLength(2)

    const ironSwordLink = links[0]!
    expect(ironSwordLink).toHaveTextContent('Iron Sword')
    expect(ironSwordLink).toHaveTextContent('blueprint')
    expect(ironSwordLink).toHaveAttribute('href', '/blueprints/bp1')

    const flameLink = links[1]!
    expect(flameLink).toHaveTextContent('Flame')
    expect(flameLink).toHaveTextContent('affix')
    expect(flameLink).toHaveAttribute('href', '/affixes/aff1')
  })

  it('does not show warning or reference list when no references exist', () => {
    renderDialog({ referencingResources: [] })

    expect(screen.queryByText(/resources will be modified/)).not.toBeInTheDocument()
    expect(screen.queryByText('Referenced by:')).not.toBeInTheDocument()
  })

  it('calls onForceDelete when Force Delete is clicked', () => {
    const onForceDelete = vi.fn()
    renderDialog({
      referencingResources: [
        { id: 'bp1', name: 'Iron Sword', type: 'blueprint', detailUrl: '/blueprints/bp1' },
      ],
      onForceDelete,
    })

    fireEvent.click(screen.getByRole('button', { name: 'Force Delete' }))
    expect(onForceDelete).toHaveBeenCalledTimes(1)
  })

  it('calls onForceDelete when Delete is clicked (no references)', () => {
    const onForceDelete = vi.fn()
    renderDialog({
      referencingResources: [],
      onForceDelete,
    })

    fireEvent.click(screen.getByRole('button', { name: 'Delete' }))
    expect(onForceDelete).toHaveBeenCalledTimes(1)
  })

  it('disables the action button when isDeleting is true', () => {
    renderDialog({
      referencingResources: [],
      isDeleting: true,
    })

    expect(screen.getByRole('button', { name: 'Deleting...' })).toBeDisabled()
  })

  it('shows "Deleting..." text when isDeleting is true with references', () => {
    renderDialog({
      referencingResources: [
        { id: 'bp1', name: 'Iron Sword', type: 'blueprint', detailUrl: '/blueprints/bp1' },
      ],
      isDeleting: true,
    })

    expect(screen.getByRole('button', { name: 'Deleting...' })).toBeInTheDocument()
  })

  it('renders with affix resource type', () => {
    renderDialog({ resourceType: 'affix', resourceName: 'Flame' })
    expect(screen.getByRole('heading', { name: 'Delete affix' })).toBeInTheDocument()
  })

  it('renders with global meta attribute resource type', () => {
    renderDialog({ resourceType: 'global meta attribute', resourceName: 'Damage' })
    expect(screen.getByRole('heading', { name: 'Delete global meta attribute' })).toBeInTheDocument()
  })

  it('renders with client resource type', () => {
    renderDialog({ resourceType: 'client', resourceName: 'MyGame' })
    expect(screen.getByRole('heading', { name: 'Delete client' })).toBeInTheDocument()
  })
})
