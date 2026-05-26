import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest'

const mockToast = vi.fn()
const mockResolveMutation = {
  mutateAsync: vi.fn(),
  isPending: false,
}

vi.mock('@/api/generated', () => ({
  useResolveImport: () => mockResolveMutation,
}))

vi.mock('@/stores/auth', () => ({
  useAuth: () => ({ apiKey: 'test-key', isAuthenticated: true }),
}))

vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: mockToast, dismiss: vi.fn(), toasts: [] }),
}))

import ImportPage from '@/pages/Import'

function renderImport() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  })
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={['/import']}>
        <ImportPage />
      </MemoryRouter>
    </QueryClientProvider>,
  )
}

function makeZipFile(name = 'export.zip'): File {
  return new File(['zip-content'], name, { type: 'application/zip' })
}

function makeNonZipFile(name = 'data.json'): File {
  return new File(['{}'], name, { type: 'application/json' })
}

function makeSuccessResponse(): Record<string, unknown> {
  return {
    status: 'ok',
    clientsCreated: 2,
    resourcesImported: 15,
  }
}

function makeConflictResponse(overrides?: Record<string, unknown>): Record<string, unknown> {
  return {
    type: '/errors/import-conflict',
    title: 'Import Conflict',
    status: 409,
    detail: '3 resources have conflicts with existing data.',
    importToken: 'import-token-abc123',
    conflicts: [
      {
        resourceType: 'blueprint',
        resourceId: 'bp-1',
        resourceName: 'Sword',
        attributes: [
          {
            key: 'damage',
            oldValue: 50,
            newValue: 75,
            valueType: 'single',
          },
          {
            key: 'rarity',
            oldValue: 'common',
            newValue: 'rare',
            valueType: 'enum',
          },
        ],
      },
      {
        resourceType: 'affix',
        resourceId: 'aff-2',
        resourceName: 'Flaming',
        attributes: [
          {
            key: 'element',
            oldValue: 'fire',
            newValue: 'ice',
            valueType: 'string',
          },
        ],
      },
    ],
    ...overrides,
  }
}

describe('Import page', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockResolveMutation.mutateAsync.mockReset()
    mockResolveMutation.isPending = false
    mockToast.mockReset()
    globalThis.fetch = vi.fn() as unknown as typeof globalThis.fetch
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  describe('initial render', () => {
    it('renders the page title', () => {
      renderImport()
      expect(screen.getByText('Import')).toBeInTheDocument()
    })

    it('renders the stepper with three steps', () => {
      renderImport()
      expect(screen.getByText('Select File')).toBeInTheDocument()
      expect(screen.getByText('Upload')).toBeInTheDocument()
      expect(screen.getByText('Result')).toBeInTheDocument()
    })

    it('renders the drop zone', () => {
      renderImport()
      expect(
        screen.getByText('Drag and drop a ZIP file here'),
      ).toBeInTheDocument()
      expect(screen.getByText('or click to browse')).toBeInTheDocument()
    })

    it('renders browse files button', () => {
      renderImport()
      expect(
        screen.getByRole('button', { name: 'Browse files' }),
      ).toBeInTheDocument()
    })

    it('renders upload button disabled when no file selected', () => {
      renderImport()
      const uploadBtn = screen.getByRole('button', { name: /Upload and Import/ })
      expect(uploadBtn).toBeDisabled()
    })
  })

  describe('file selection', () => {
    it('shows file info after selecting a valid ZIP file', () => {
      renderImport()
      const input = screen.getByLabelText('Browse files')
      fireEvent.change(input, { target: { files: [makeZipFile('backup.zip')] } })

      expect(screen.getByText('backup.zip')).toBeInTheDocument()
      expect(screen.getByRole('button', { name: 'Remove file' })).toBeInTheDocument()
    })

    it('enables upload button after selecting a file', () => {
      renderImport()
      const input = screen.getByLabelText('Browse files')
      fireEvent.change(input, { target: { files: [makeZipFile()] } })

      const uploadBtn = screen.getByRole('button', { name: /Upload and Import/ })
      expect(uploadBtn).not.toBeDisabled()
    })

    it('shows error toast for non-ZIP files', () => {
      renderImport()
      const input = screen.getByLabelText('Browse files')
      fireEvent.change(input, { target: { files: [makeNonZipFile('data.json')] } })

      expect(mockToast).toHaveBeenCalledWith(
        expect.objectContaining({
          title: 'Invalid file type',
          variant: 'destructive',
        }),
      )
    })

    it('does not show file info for non-ZIP files', () => {
      renderImport()
      const input = screen.getByLabelText('Browse files')
      fireEvent.change(input, { target: { files: [makeNonZipFile('data.json')] } })

      expect(screen.queryByText('data.json')).not.toBeInTheDocument()
    })

    it('removes file when remove button is clicked', () => {
      renderImport()
      const input = screen.getByLabelText('Browse files')
      fireEvent.change(input, { target: { files: [makeZipFile()] } })

      expect(screen.getByText('export.zip')).toBeInTheDocument()

      fireEvent.click(screen.getByRole('button', { name: 'Remove file' }))
      expect(screen.queryByText('export.zip')).not.toBeInTheDocument()
    })

    it('handles drag-and-drop of valid ZIP file', () => {
      renderImport()
      const dropZone = screen.getByRole('button', {
        name: 'Drop zone for ZIP file upload',
      })

      const file = makeZipFile('drag.zip')
      const dataTransfer = { files: [file], types: ['Files'] }

      fireEvent.dragOver(dropZone, { dataTransfer })
      fireEvent.drop(dropZone, { dataTransfer })

      expect(screen.getByText('drag.zip')).toBeInTheDocument()
    })

    it('validates file type on drop', () => {
      renderImport()
      const dropZone = screen.getByRole('button', {
        name: 'Drop zone for ZIP file upload',
      })

      const file = makeNonZipFile('bad.exe')
      const dataTransfer = { files: [file], types: ['Files'] }

      fireEvent.drop(dropZone, { dataTransfer })

      expect(mockToast).toHaveBeenCalledWith(
        expect.objectContaining({ title: 'Invalid file type' }),
      )
      expect(screen.queryByText('bad.exe')).not.toBeInTheDocument()
    })

    it('clicking drop zone opens file browser', () => {
      renderImport()
      const dropZone = screen.getByRole('button', {
        name: 'Drop zone for ZIP file upload',
      })
      const input = screen.getByLabelText('Browse files')

      const clickSpy = vi.fn()
      input.click = clickSpy

      fireEvent.click(dropZone)
      expect(clickSpy).toHaveBeenCalled()
    })
  })

  describe('upload and import', () => {
    it('shows loading state during upload', async () => {
      const fetchMock = vi.fn(
        () => new Promise(() => {}),
      )
      globalThis.fetch = fetchMock as unknown as typeof fetch

      renderImport()
      const input = screen.getByLabelText('Browse files')
      fireEvent.change(input, { target: { files: [makeZipFile()] } })

      fireEvent.click(screen.getByRole('button', { name: /Upload and Import/ }))
      await waitFor(() => {
        expect(
          screen.getByText(/Uploading and parsing archive/),
        ).toBeInTheDocument()
      })
    })

    it('sends file to import API', async () => {
      const fetchMock = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(makeSuccessResponse()),
      })
      globalThis.fetch = fetchMock as unknown as typeof fetch

      renderImport()
      const input = screen.getByLabelText('Browse files')
      fireEvent.change(input, { target: { files: [makeZipFile('data.zip')] } })

      fireEvent.click(screen.getByRole('button', { name: /Upload and Import/ }))

      await waitFor(() => {
        expect(fetchMock).toHaveBeenCalledWith(
          expect.stringContaining('/api/import'),
          expect.objectContaining({
            method: 'POST',
            headers: expect.objectContaining({
              'X-API-Key': 'test-key',
            }),
            body: expect.any(FormData),
          }),
        )
      })

      const callBody = fetchMock.mock.calls[0][1].body as FormData
      expect(callBody.get('file')).toBeInstanceOf(File)
    })

    it('shows success state on import success', async () => {
      const fetchMock = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(makeSuccessResponse()),
      })
      globalThis.fetch = fetchMock as unknown as typeof fetch

      renderImport()
      const input = screen.getByLabelText('Browse files')
      fireEvent.change(input, { target: { files: [makeZipFile()] } })

      fireEvent.click(screen.getByRole('button', { name: /Upload and Import/ }))

      await waitFor(() => {
        expect(screen.getByText('Import Successful')).toBeInTheDocument()
      })
      expect(screen.getByText(/2 clients created/)).toBeInTheDocument()
      expect(screen.getByText(/15 resources imported/)).toBeInTheDocument()
    })

    it('shows conflict resolution on 409', async () => {
      const fetchMock = vi.fn().mockResolvedValue({
        ok: false,
        status: 409,
        json: () => Promise.resolve(makeConflictResponse()),
      })
      globalThis.fetch = fetchMock as unknown as typeof fetch

      renderImport()
      const input = screen.getByLabelText('Browse files')
      fireEvent.change(input, { target: { files: [makeZipFile()] } })

      fireEvent.click(screen.getByRole('button', { name: /Upload and Import/ }))

      await waitFor(() => {
        expect(
          screen.getByText('3 resources have conflicts with existing data.'),
        ).toBeInTheDocument()
      })
      expect(screen.getByText('Sword')).toBeInTheDocument()
      expect(screen.getByText('Flaming')).toBeInTheDocument()
    })

    it('shows error toast on unexpected error', async () => {
      const fetchMock = vi.fn().mockResolvedValue({
        ok: false,
        status: 500,
        json: () =>
          Promise.resolve({
            type: '/errors/unprocessable-entity',
            title: 'Server Error',
            status: 500,
            detail: 'Internal server error',
          }),
      })
      globalThis.fetch = fetchMock as unknown as typeof fetch

      renderImport()
      const input = screen.getByLabelText('Browse files')
      fireEvent.change(input, { target: { files: [makeZipFile()] } })

      fireEvent.click(screen.getByRole('button', { name: /Upload and Import/ }))

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith(
          expect.objectContaining({
            title: 'Import failed',
            variant: 'destructive',
          }),
        )
      })
    })

    it('shows error toast on network failure', async () => {
      const fetchMock = vi.fn().mockRejectedValue(new Error('Network error'))
      globalThis.fetch = fetchMock as unknown as typeof fetch

      renderImport()
      const input = screen.getByLabelText('Browse files')
      fireEvent.change(input, { target: { files: [makeZipFile()] } })

      fireEvent.click(screen.getByRole('button', { name: /Upload and Import/ }))

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith(
          expect.objectContaining({
            title: 'Import failed',
            variant: 'destructive',
          }),
        )
      })
    })

    it('returns to select step after upload error', async () => {
      const fetchMock = vi.fn().mockRejectedValue(new Error('Network error'))
      globalThis.fetch = fetchMock as unknown as typeof fetch

      renderImport()
      const input = screen.getByLabelText('Browse files')
      fireEvent.change(input, { target: { files: [makeZipFile()] } })

      fireEvent.click(screen.getByRole('button', { name: /Upload and Import/ }))

      await waitFor(() => {
        expect(
          screen.getByText('Drag and drop a ZIP file here'),
        ).toBeInTheDocument()
      })
    })
  })

  describe('success state', () => {
    it('shows clients created and resources imported counts', async () => {
      const fetchMock = vi.fn().mockResolvedValue({
        ok: true,
        json: () =>
          Promise.resolve({
            status: 'ok',
            clientsCreated: 3,
            resourcesImported: 42,
          }),
      })
      globalThis.fetch = fetchMock as unknown as typeof fetch

      renderImport()
      const input = screen.getByLabelText('Browse files')
      fireEvent.change(input, { target: { files: [makeZipFile()] } })

      fireEvent.click(screen.getByRole('button', { name: /Upload and Import/ }))

      await waitFor(() => {
        expect(screen.getByText(/3 clients created/)).toBeInTheDocument()
      })
      expect(screen.getByText(/42 resources imported/)).toBeInTheDocument()
    })

    it('Start New Import button resets the form', async () => {
      const fetchMock = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(makeSuccessResponse()),
      })
      globalThis.fetch = fetchMock as unknown as typeof fetch

      renderImport()
      const input = screen.getByLabelText('Browse files')
      fireEvent.change(input, { target: { files: [makeZipFile()] } })

      fireEvent.click(screen.getByRole('button', { name: /Upload and Import/ }))

      await waitFor(() => {
        expect(screen.getByText('Import Successful')).toBeInTheDocument()
      })

      fireEvent.click(screen.getByRole('button', { name: 'Start New Import' }))

      expect(
        screen.getByText('Drag and drop a ZIP file here'),
      ).toBeInTheDocument()
    })
  })

  describe('conflict resolution', () => {
    async function renderWithConflicts() {
      const fetchMock = vi.fn().mockResolvedValue({
        ok: false,
        status: 409,
        json: () => Promise.resolve(makeConflictResponse()),
      })
      globalThis.fetch = fetchMock as unknown as typeof fetch

      renderImport()
      const input = screen.getByLabelText('Browse files')
      fireEvent.change(input, { target: { files: [makeZipFile()] } })

      fireEvent.click(screen.getByRole('button', { name: /Upload and Import/ }))

      await waitFor(() => {
        expect(screen.getByText('Sword')).toBeInTheDocument()
      })
    }

    it('shows conflict resources with attribute diffs', async () => {
      await renderWithConflicts()

      expect(screen.getByText('blueprint')).toBeInTheDocument()
      expect(screen.getByText('Sword')).toBeInTheDocument()
      expect(screen.getByText('affix')).toBeInTheDocument()
      expect(screen.getByText('Flaming')).toBeInTheDocument()
      expect(screen.getByText('damage')).toBeInTheDocument()
      expect(screen.getByText('rarity')).toBeInTheDocument()
    })

    it('shows global apply to all remaining buttons', async () => {
      await renderWithConflicts()

      expect(
        screen.getByRole('button', { name: 'Keep Existing' }),
      ).toBeInTheDocument()
      expect(
        screen.getByRole('button', { name: 'Use Imported' }),
      ).toBeInTheDocument()
    })

    it('shows per-resource apply to all dropdowns', async () => {
      await renderWithConflicts()

      const applyToAllTriggers = screen.getAllByText('Choose…')
      expect(applyToAllTriggers.length).toBeGreaterThanOrEqual(2)
    })

    it('shows per-attribute resolution dropdowns', async () => {
      await renderWithConflicts()

      const selectTriggers = screen.getAllByRole('combobox')
      expect(selectTriggers.length).toBeGreaterThanOrEqual(4)
    })

    it('Apply Resolutions button is disabled until all resolved', async () => {
      await renderWithConflicts()

      const applyBtn = screen.getByRole('button', { name: /Apply Resolutions/ })
      expect(applyBtn).toBeDisabled()
    })

    it('enables Apply Resolutions after global apply to all remaining', async () => {
      await renderWithConflicts()

      const applyBtn = screen.getByRole('button', { name: /Apply Resolutions/ })
      expect(applyBtn).toBeDisabled()

      fireEvent.click(screen.getByRole('button', { name: 'Keep Existing' }))

      await waitFor(() => {
        expect(applyBtn).not.toBeDisabled()
      })
    })

    it('Apply Resolutions calls resolve import', async () => {
      mockResolveMutation.mutateAsync.mockResolvedValue({
        status: 'ok',
        clientsCreated: 2,
        resourcesImported: 15,
      })

      await renderWithConflicts()

      // Use global "apply to all remaining" to quickly set resolutions
      const keepButtons = screen.getAllByText('Keep Existing')
      fireEvent.click(keepButtons[0])

      await waitFor(() => {
        const applyBtn = screen.getByRole('button', { name: /Apply Resolutions/ })
        expect(applyBtn).not.toBeDisabled()
      })

      fireEvent.click(screen.getByRole('button', { name: /Apply Resolutions/ }))

      await waitFor(() => {
        expect(mockResolveMutation.mutateAsync).toHaveBeenCalledWith({
          importToken: 'import-token-abc123',
          resolutions: expect.objectContaining({
            'bp-1': expect.any(Object),
            'aff-2': expect.any(Object),
          }),
        })
      })
    })

    it('shows success toast after resolution', async () => {
      mockResolveMutation.mutateAsync.mockResolvedValue({
        status: 'ok',
        clientsCreated: 2,
        resourcesImported: 15,
      })

      await renderWithConflicts()

      const keepButtons = screen.getAllByText('Keep Existing')
      fireEvent.click(keepButtons[0])

      await waitFor(() => {
        const applyBtn = screen.getByRole('button', { name: /Apply Resolutions/ })
        expect(applyBtn).not.toBeDisabled()
      })

      fireEvent.click(screen.getByRole('button', { name: /Apply Resolutions/ }))

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith(
          expect.objectContaining({
            title: 'Import complete',
            variant: 'success',
          }),
        )
      })
    })

    it('shows error toast on resolution failure', async () => {
      mockResolveMutation.mutateAsync.mockRejectedValue(
        new Error('Resolution error'),
      )

      await renderWithConflicts()

      const keepButtons = screen.getAllByText('Keep Existing')
      fireEvent.click(keepButtons[0])

      await waitFor(() => {
        const applyBtn = screen.getByRole('button', { name: /Apply Resolutions/ })
        expect(applyBtn).not.toBeDisabled()
      })

      fireEvent.click(screen.getByRole('button', { name: /Apply Resolutions/ }))

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith(
          expect.objectContaining({
            title: 'Resolution failed',
            variant: 'destructive',
          }),
        )
      })
    })

    it('Cancel button resets to select step', async () => {
      await renderWithConflicts()

      fireEvent.click(screen.getByRole('button', { name: 'Cancel' }))

      expect(
        screen.getByText('Drag and drop a ZIP file here'),
      ).toBeInTheDocument()
    })

    it('global apply to all remaining sets all resolutions', async () => {
      await renderWithConflicts()

      const useImportedButtons = screen.getAllByText('Use Imported')
      fireEvent.click(useImportedButtons[0])

      await waitFor(() => {
        const applyBtn = screen.getByRole('button', { name: /Apply Resolutions/ })
        expect(applyBtn).not.toBeDisabled()
      })
    })

    it('handles empty conflicts gracefully', async () => {
      const fetchMock = vi.fn().mockResolvedValue({
        ok: false,
        status: 409,
        json: () =>
          Promise.resolve({
            type: '/errors/import-conflict',
            title: 'Import Conflict',
            status: 409,
            detail: 'No conflicts',
            importToken: 'tok',
            conflicts: [],
          }),
      })
      globalThis.fetch = fetchMock as unknown as typeof fetch

      renderImport()
      const input = screen.getByLabelText('Browse files')
      fireEvent.change(input, { target: { files: [makeZipFile()] } })

      fireEvent.click(screen.getByRole('button', { name: /Upload and Import/ }))

      await waitFor(() => {
        expect(screen.getByText('No conflicts')).toBeInTheDocument()
      })
    })
  })
})
