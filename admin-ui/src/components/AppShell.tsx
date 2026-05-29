import { useState } from 'react'
import { Outlet } from 'react-router-dom'
import { PanelRightClose, PanelRightOpen } from 'lucide-react'
import { useUi } from '@/stores/ui'
import { AppSidebar } from './AppSidebar'
import { ActivityPanel } from './ActivityPanel'
import { GlobalSearch } from './GlobalSearch'
import { SidebarProvider, useSidebar } from '@/components/ui/sidebar'

function ShellInner() {
  const { panelOpen, togglePanel } = useUi()
  const { state } = useSidebar()

  return (
    <div className="flex w-full min-h-svh">
      <AppSidebar />
      <button
        onClick={togglePanel}
        className="fixed top-2 z-50 text-foreground/30 hover:text-foreground transition-colors"
        style={{ right: panelOpen ? 'calc(20rem + 1.5rem)' : '1rem' }}
        aria-label={panelOpen ? 'Close activity panel' : 'Open activity panel'}
      >
        {panelOpen ? <PanelRightClose className="h-4 w-4" /> : <PanelRightOpen className="h-4 w-4" />}
      </button>
      <div
        className="flex flex-1 flex-col min-w-0 transition-all duration-200 ease-linear"
        style={{
          paddingLeft: state === 'expanded' ? 'var(--sidebar-width)' : 'var(--sidebar-width-icon)',
        }}
      >
        <main className="flex-1 p-4">
          <Outlet />
        </main>
      </div>
      {panelOpen && <ActivityPanel />}
    </div>
  )
}

export function AppShell() {
  const [sidebarOpen, setSidebarOpen] = useState(true)

  return (
    <SidebarProvider open={sidebarOpen} onOpenChange={setSidebarOpen}>
      <ShellInner />
      <GlobalSearch />
    </SidebarProvider>
  )
}
