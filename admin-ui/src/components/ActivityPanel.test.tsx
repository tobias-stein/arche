import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { ActivityPanel } from '@/components/ActivityPanel'

function renderPanel() {
  return render(<ActivityPanel />)
}

describe('ActivityPanel', () => {
  beforeEach(async () => {
    const { useUi } = await import('@/stores/ui')
    useUi.setState({ panelOpen: true })
  })

  it('renders the activity panel', () => {
    renderPanel()

    expect(screen.getByText('Activity')).toBeInTheDocument()
    expect(screen.getByText('Activity log will appear here.')).toBeInTheDocument()
  })

  it('has a close button', () => {
    renderPanel()

    const closeButton = screen.getByLabelText('Close activity panel')
    expect(closeButton).toBeInTheDocument()
  })

  it('closes panel on close button click', async () => {
    const { useUi } = await import('@/stores/ui')
    useUi.setState({ panelOpen: true })

    renderPanel()

    const closeButton = screen.getByLabelText('Close activity panel')
    fireEvent.click(closeButton)

    expect(useUi.getState().panelOpen).toBe(false)
  })

  it('closes panel on outside click', async () => {
    const { useUi } = await import('@/stores/ui')
    useUi.setState({ panelOpen: true })

    renderPanel()

    fireEvent.mouseDown(document.body)

    expect(useUi.getState().panelOpen).toBe(false)
  })

  it('does not close when clicking inside the panel', async () => {
    const { useUi } = await import('@/stores/ui')
    useUi.setState({ panelOpen: true })

    renderPanel()

    const panelContent = screen.getByText('Activity')
    fireEvent.mouseDown(panelContent)

    expect(useUi.getState().panelOpen).toBe(true)
  })
})
