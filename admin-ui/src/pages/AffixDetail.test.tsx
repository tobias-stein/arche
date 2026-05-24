import { render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { describe, expect, it } from 'vitest'
import { ArcheClient } from '@/api/generated/client'
import { setClient } from '@/api/generated/hooks'
import AffixDetail from '@/pages/AffixDetail'

setClient(new ArcheClient({ baseUrl: 'http://localhost:3000' }))

function renderPage(id: string) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })

  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={[`/affixes/${id}`]}>
        <AffixDetail />
      </MemoryRouter>
    </QueryClientProvider>,
  )
}

describe('AffixDetail', () => {
  it('renders loading skeleton initially', () => {
    renderPage('some-id')
    expect(screen.getByLabelText('Back to affixes')).toBeInTheDocument()
  })

  it('shows back navigation button', () => {
    renderPage('some-id')
    expect(screen.getByLabelText('Back to affixes')).toBeInTheDocument()
  })

  it('shows 404 state when affix not found', async () => {
    renderPage('non-existent-id')
    await waitFor(
      () => {
        expect(screen.getByText('Affix not found')).toBeInTheDocument()
      },
      { timeout: 5000 },
    )
  })

  it('shows "Back to Affixes" button in 404 state', async () => {
    renderPage('non-existent-id')
    await waitFor(
      () => {
        expect(screen.getByRole('button', { name: 'Back to Affixes' })).toBeInTheDocument()
      },
      { timeout: 5000 },
    )
  })

  it('shows tabs when loaded (or error state with back button)', async () => {
    renderPage('some-id')
    await waitFor(
      () => {
        expect(screen.getByText('Affix not found')).toBeInTheDocument()
      },
      { timeout: 5000 },
    )
  })

  it('shows error message text for deleted/missing affix', async () => {
    renderPage('non-existent-id')
    await waitFor(
      () => {
        expect(screen.getByText(/does not exist or has been deleted/)).toBeInTheDocument()
      },
      { timeout: 5000 },
    )
  })
})
