import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { describe, expect, it, vi, beforeEach } from 'vitest'

const mockClientsList = vi.fn()
const mockExportMutation = {
  mutateAsync: vi.fn(),
  isPending: false,
}

vi.mock('@/api/generated', () => ({
  useClientsList: (query?: unknown) => mockClientsList(query),
  useExportData: () => mockExportMutation,
}))

vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: mockToast, dismiss: vi.fn(), toasts: [] }),
}))

const mockToast = vi.fn()

import ExportPage from '@/pages/Export'

function makeClient(overrides: Record<string, unknown> = {}) {
  return {
    id: `client-${overrides.id || '1'}`,
    name: `Client ${overrides.id || '1'}`,
    createdAt: '2025-01-01T00:00:00Z',
    updatedAt: '2025-01-15T12:00:00Z',
    ...overrides,
  }
}

function renderExport() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={['/export']}>
        <ExportPage />
      </MemoryRouter>
    </QueryClientProvider>,
  )
}

describe('Export page', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockExportMutation.isPending = false
    mockExportMutation.mutateAsync.mockReset()
    mockToast.mockReset()
    URL.createObjectURL = vi.fn(() => 'blob:export')
    URL.revokeObjectURL = vi.fn()
  })

  describe('loading state', () => {
    it('renders skeleton placeholders while clients are loading', () => {
      mockClientsList.mockReturnValue({
        data: undefined,
        isLoading: true,
        isError: false,
        error: null,
      })
      renderExport()
      expect(screen.getByText('Export')).toBeInTheDocument()
      const skeletons = document.querySelectorAll('.animate-pulse')
      expect(skeletons.length).toBeGreaterThan(0)
    })
  })

  describe('error state', () => {
    it('renders error message when clients fail to load', () => {
      mockClientsList.mockReturnValue({
        data: undefined,
        isLoading: false,
        isError: true,
        error: new Error('Network error'),
      })
      renderExport()
      expect(screen.getByText('Failed to load clients')).toBeInTheDocument()
      expect(screen.getByText('Network error')).toBeInTheDocument()
    })
  })

  describe('empty state', () => {
    it('renders empty message when no clients exist', () => {
      mockClientsList.mockReturnValue({
        data: { data: [], total: 0 },
        isLoading: false,
        isError: false,
        error: null,
      })
      renderExport()
      expect(screen.getByText('No clients found')).toBeInTheDocument()
    })
  })

  describe('client table', () => {
    it('renders clients with checkboxes, all pre-selected', () => {
      mockClientsList.mockReturnValue({
        data: { data: [makeClient({ id: '1' }), makeClient({ id: '2' })], total: 2 },
        isLoading: false,
        isError: false,
        error: null,
      })
      renderExport()

      expect(screen.getByText('Client 1')).toBeInTheDocument()
      expect(screen.getByText('Client 2')).toBeInTheDocument()

      const checkboxes = screen.getAllByRole('checkbox', { name: /Select/ })
      const clientCheckboxes = checkboxes.filter(
        (cb) => cb.getAttribute('aria-label') !== 'Select all clients',
      )
      clientCheckboxes.forEach((cb) => {
        expect(cb).toBeChecked()
      })

      expect(screen.getByLabelText('Select all clients')).toBeChecked()
    })

    it('toggle all deselects all clients', () => {
      mockClientsList.mockReturnValue({
        data: { data: [makeClient({ id: '1' }), makeClient({ id: '2' })], total: 2 },
        isLoading: false,
        isError: false,
        error: null,
      })
      renderExport()

      const selectAll = screen.getByLabelText('Select all clients')
      fireEvent.click(selectAll)

      const checkboxes = screen.getAllByRole('checkbox', { name: /Select/ })
      const clientCheckboxes = checkboxes.filter(
        (cb) => cb.getAttribute('aria-label') !== 'Select all clients',
      )
      clientCheckboxes.forEach((cb) => {
        expect(cb).not.toBeChecked()
      })
    })

    it('toggle all reselects all clients', () => {
      mockClientsList.mockReturnValue({
        data: { data: [makeClient({ id: '1' }), makeClient({ id: '2' })], total: 2 },
        isLoading: false,
        isError: false,
        error: null,
      })
      renderExport()

      const selectAll = screen.getByLabelText('Select all clients')
      fireEvent.click(selectAll)
      fireEvent.click(selectAll)

      const checkboxes = screen.getAllByRole('checkbox', { name: /Select/ })
      const clientCheckboxes = checkboxes.filter(
        (cb) => cb.getAttribute('aria-label') !== 'Select all clients',
      )
      clientCheckboxes.forEach((cb) => {
        expect(cb).toBeChecked()
      })
    })

    it('deselecting an individual client toggles its checkbox', () => {
      mockClientsList.mockReturnValue({
        data: { data: [makeClient({ id: '1' }), makeClient({ id: '2' })], total: 2 },
        isLoading: false,
        isError: false,
        error: null,
      })
      renderExport()

      const client1Checkbox = screen.getByLabelText('Select Client 1')
      fireEvent.click(client1Checkbox)
      expect(client1Checkbox).not.toBeChecked()

      fireEvent.click(client1Checkbox)
      expect(client1Checkbox).toBeChecked()
    })
  })

  describe('export options', () => {
    it('renders API keys, audit log, and inline global refs toggles', () => {
      mockClientsList.mockReturnValue({
        data: { data: [makeClient({ id: '1' })], total: 1 },
        isLoading: false,
        isError: false,
        error: null,
      })
      renderExport()

      expect(screen.getByLabelText('Export API keys')).toBeInTheDocument()
      expect(screen.getByLabelText('Export audit log')).toBeInTheDocument()
      expect(screen.getByLabelText('Inline global references')).toBeInTheDocument()
    })

    it('toggles default to unchecked', () => {
      mockClientsList.mockReturnValue({
        data: { data: [makeClient({ id: '1' })], total: 1 },
        isLoading: false,
        isError: false,
        error: null,
      })
      renderExport()

      expect(screen.getByLabelText('Export API keys')).not.toBeChecked()
      expect(screen.getByLabelText('Export audit log')).not.toBeChecked()
      expect(screen.getByLabelText('Inline global references')).not.toBeChecked()
    })

    it('can toggle options on and off', () => {
      mockClientsList.mockReturnValue({
        data: { data: [makeClient({ id: '1' })], total: 1 },
        isLoading: false,
        isError: false,
        error: null,
      })
      renderExport()

      const apiKeysToggle = screen.getByLabelText('Export API keys')
      fireEvent.click(apiKeysToggle)
      expect(apiKeysToggle).toBeChecked()
      fireEvent.click(apiKeysToggle)
      expect(apiKeysToggle).not.toBeChecked()
    })
  })

  describe('export action', () => {
    it('sends correct ExportRequest and downloads ZIP', async () => {
      const mockArrayBuffer = new ArrayBuffer(8)
      mockExportMutation.mutateAsync.mockResolvedValue(mockArrayBuffer)

      const client1 = makeClient({ id: '1', name: 'Alpha' })
      const client2 = makeClient({ id: '2', name: 'Beta' })
      mockClientsList.mockReturnValue({
        data: {
          data: [client1, client2],
          total: 2,
        },
        isLoading: false,
        isError: false,
        error: null,
      })
      renderExport()

      const apiKeysToggle = screen.getByLabelText('Export API keys')
      const auditLogToggle = screen.getByLabelText('Export audit log')
      const inlineRefsToggle = screen.getByLabelText('Inline global references')
      fireEvent.click(apiKeysToggle)
      fireEvent.click(auditLogToggle)
      fireEvent.click(inlineRefsToggle)

      const exportButton = screen.getByRole('button', { name: /Export/ })
      fireEvent.click(exportButton)

      await waitFor(() => {
        expect(mockExportMutation.mutateAsync).toHaveBeenCalledWith({
          client_ids: [client1.id, client2.id],
          include_api_keys: true,
          include_audit_log: true,
          inline_global_refs: true,
        })
      })

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith(
          expect.objectContaining({ title: 'Export complete' }),
        )
      })
    })

    it('disables export button when no clients selected', () => {
      mockClientsList.mockReturnValue({
        data: { data: [makeClient({ id: '1' })], total: 1 },
        isLoading: false,
        isError: false,
        error: null,
      })
      renderExport()

      const selectAll = screen.getByLabelText('Select all clients')
      fireEvent.click(selectAll)

      const exportButton = screen.getByRole('button', { name: /Export/ })
      expect(exportButton).toBeDisabled()
    })

    it('disables export button while export is in progress', () => {
      mockExportMutation.isPending = true
      mockClientsList.mockReturnValue({
        data: { data: [makeClient({ id: '1' })], total: 1 },
        isLoading: false,
        isError: false,
        error: null,
      })
      renderExport()

      const exportButton = screen.getByRole('button', { name: 'Preparing export...' })
      expect(exportButton).toBeDisabled()
    })

    it('shows loading spinner during export', () => {
      mockExportMutation.isPending = true
      mockClientsList.mockReturnValue({
        data: { data: [makeClient({ id: '1' })], total: 1 },
        isLoading: false,
        isError: false,
        error: null,
      })
      renderExport()

      expect(screen.getByText('Preparing export...')).toBeInTheDocument()
    })

    it('shows toast on export failure', async () => {
      mockExportMutation.mutateAsync.mockRejectedValue(new Error('Server error'))
      mockClientsList.mockReturnValue({
        data: { data: [makeClient({ id: '1' })], total: 1 },
        isLoading: false,
        isError: false,
        error: null,
      })
      renderExport()

      const exportButton = screen.getByRole('button', { name: /Export/ })
      fireEvent.click(exportButton)

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith(
          expect.objectContaining({
            title: 'Export failed',
            variant: 'destructive',
          }),
        )
      })
    })
  })
})
