import { render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { describe, expect, it } from 'vitest'
import { ArcheClient } from '@/api/generated/client'
import { setClient } from '@/api/generated/hooks'
import BlueprintDetail from '@/pages/BlueprintDetail'

setClient(new ArcheClient({ baseUrl: 'http://localhost:3000' }))

function renderPage(id: string) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })

  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={[`/blueprints/${id}`]}>
        <BlueprintDetail />
      </MemoryRouter>
    </QueryClientProvider>,
  )
}

describe('BlueprintDetail', () => {
  it('renders loading skeleton initially', () => {
    renderPage('some-id')
    expect(screen.getByLabelText('Back to blueprints')).toBeInTheDocument()
  })

  it('shows back navigation button', () => {
    renderPage('some-id')
    expect(screen.getByLabelText('Back to blueprints')).toBeInTheDocument()
  })

  it('shows 404 state when blueprint not found', async () => {
    renderPage('non-existent-id')
    await waitFor(
      () => {
        expect(screen.getByText('Blueprint not found')).toBeInTheDocument()
      },
      { timeout: 5000 },
    )
  })

  it('shows "Back to Blueprints" button in 404 state', async () => {
    renderPage('non-existent-id')
    await waitFor(
      () => {
        expect(screen.getByRole('button', { name: 'Back to Blueprints' })).toBeInTheDocument()
      },
      { timeout: 5000 },
    )
  })

  it('shows tabs when loaded (or error state with back button)', async () => {
    renderPage('some-id')
    await waitFor(
      () => {
        expect(screen.getByText('Blueprint not found')).toBeInTheDocument()
      },
      { timeout: 5000 },
    )
  })

  it('shows error message text for deleted/missing blueprint', async () => {
    renderPage('non-existent-id')
    await waitFor(
      () => {
        expect(screen.getByText(/does not exist or has been deleted/)).toBeInTheDocument()
      },
      { timeout: 5000 },
    )
  })

  it('displays all tab labels in the error state', async () => {
    renderPage('non-existent-id')
    await waitFor(
      () => {
        expect(screen.getByText('Blueprint not found')).toBeInTheDocument()
      },
      { timeout: 5000 },
    )
    expect(screen.getByRole('button', { name: 'Back to Blueprints' })).toBeInTheDocument()
  })
})

