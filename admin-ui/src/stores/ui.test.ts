import { act } from 'react'
import { describe, expect, it } from 'vitest'
import { useUi } from '@/stores/ui'

function getState() {
  return useUi.getState()
}

describe('useUi store', () => {
  beforeEach(() => {
    act(() => {
      useUi.setState({
        drawerOpen: false,
        panelOpen: false,
        panelDocked: false,
      })
    })
  })

  it('starts with drawer and panel closed', () => {
    const state = getState()
    expect(state.drawerOpen).toBe(false)
    expect(state.panelOpen).toBe(false)
    expect(state.panelDocked).toBe(false)
  })

  it('toggleDrawer opens the drawer when closed', () => {
    act(() => {
      getState().toggleDrawer()
    })
    expect(getState().drawerOpen).toBe(true)
  })

  it('toggleDrawer closes the drawer when open', () => {
    act(() => {
      useUi.setState({ drawerOpen: true })
      getState().toggleDrawer()
    })
    expect(getState().drawerOpen).toBe(false)
  })

  it('closeDrawer sets drawer to false', () => {
    act(() => {
      useUi.setState({ drawerOpen: true })
      getState().closeDrawer()
    })
    expect(getState().drawerOpen).toBe(false)
  })

  it('togglePanel opens the panel when closed', () => {
    act(() => {
      getState().togglePanel()
    })
    expect(getState().panelOpen).toBe(true)
  })

  it('togglePanel closes the panel when open', () => {
    act(() => {
      useUi.setState({ panelOpen: true })
      getState().togglePanel()
    })
    expect(getState().panelOpen).toBe(false)
  })

  it('closePanel sets panel to false', () => {
    act(() => {
      useUi.setState({ panelOpen: true })
      getState().closePanel()
    })
    expect(getState().panelOpen).toBe(false)
  })

  it('setPanelDocked updates docked state', () => {
    act(() => {
      getState().setPanelDocked(true)
    })
    expect(getState().panelDocked).toBe(true)

    act(() => {
      getState().setPanelDocked(false)
    })
    expect(getState().panelDocked).toBe(false)
  })

  it('toggle does not affect panel when toggling drawer', () => {
    act(() => {
      useUi.setState({ panelOpen: true })
      getState().toggleDrawer()
    })
    expect(getState().drawerOpen).toBe(true)
    expect(getState().panelOpen).toBe(true)
  })
})
