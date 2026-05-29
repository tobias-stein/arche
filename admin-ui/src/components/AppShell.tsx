import { Outlet } from 'react-router-dom'
import { Building2, Moon, PanelRightClose, PanelRightOpen, Search, Sun } from 'lucide-react'
import { useTheme } from '@/stores/theme'
import { useUi } from '@/stores/ui'
import { useAuth } from '@/stores/auth'
import { useClient } from '@/api/generated/hooks'
import { AppSidebar } from './AppSidebar'
import { ActivityPanel } from './ActivityPanel'
import { GlobalSearch } from './GlobalSearch'
import { SidebarProvider, SidebarTrigger } from '@/components/ui/sidebar'

function ThemeToggle() {
  const { mode, toggle } = useTheme()
  return (
    <button
      onClick={toggle}
      className="inline-flex items-center justify-center h-9 w-9 rounded-md hover:bg-accent hover:text-accent-foreground"
      aria-label={mode === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}
    >
      {mode === 'dark' ? <Sun className="h-5 w-5" /> : <Moon className="h-5 w-5" />}
    </button>
  )
}

function Header() {
  const { panelOpen, togglePanel, openSearch } = useUi()
  const { isAuthenticated, isSuperAdmin, client_id: clientId } = useAuth()
  const { data: clientData } = useClient(clientId ?? '')

  const isClientScoped = isAuthenticated && !isSuperAdmin && clientId
  const title = isClientScoped ? (clientData?.name ?? '') : 'Arche Admin'

  return (
    <header className="sticky top-0 z-50 h-16 border-b bg-background flex items-center gap-2 md:gap-4 px-4">
      <SidebarTrigger className="h-9 w-9" />
      <span className="text-lg font-bold flex items-center gap-2 truncate">
        {isClientScoped && clientData && <Building2 className="h-4 w-4 shrink-0" />}
        {title}
      </span>
      <div className="flex-1 max-w-md mx-auto hidden sm:block">
        <button
          onClick={openSearch}
          className="w-full h-9 flex items-center gap-2 rounded-md border border-input bg-background px-3 text-sm text-muted-foreground hover:bg-accent hover:text-accent-foreground"
        >
          <Search className="h-4 w-4" />
          <span>Search...</span>
          <kbd className="ml-auto text-xs bg-muted px-1.5 py-0.5 rounded hidden sm:inline-flex items-center">
            <svg width="15" height="15" viewBox="0 0 15 15" fill="none" xmlns="http://www.w3.org/2000/svg" className="mr-0.5"><path d="M11.4669 2.02344C11.4669 1.59959 11.1222 1.25488 10.6984 1.25488C10.2745 1.25488 9.92983 1.59959 9.92983 2.02344V5.51836L6.50989 2.09841C6.13094 1.71946 5.51734 1.71946 5.13839 2.09841C4.75941 2.47739 4.75941 3.09099 5.13839 3.46994L8.66815 7.00002L5.13839 10.5301C4.75941 10.9091 4.75941 11.5227 5.13839 11.9016C5.51734 12.2806 6.13094 12.2806 6.50989 11.9016L9.92983 8.48169V11.9766C9.92983 12.4004 10.2745 12.7451 10.6984 12.7451C11.1222 12.7451 11.4669 12.4004 11.4669 11.9766V2.02344Z" fill="currentColor" fillRule="evenodd" clipRule="evenodd"></path></svg>
            K
          </kbd>
        </button>
      </div>
      <div className="flex items-center gap-1 md:gap-2">
        <ThemeToggle />
        <button
          onClick={togglePanel}
          className="inline-flex items-center justify-center h-9 w-9 rounded-md hover:bg-accent hover:text-accent-foreground"
          aria-label={panelOpen ? 'Close activity panel' : 'Open activity panel'}
        >
          {panelOpen ? <PanelRightClose className="h-5 w-5" /> : <PanelRightOpen className="h-5 w-5" />}
        </button>
      </div>
    </header>
  )
}

export function AppShell() {
  const { panelOpen } = useUi()

  return (
    <SidebarProvider defaultOpen={true}>
      <div className="flex min-h-screen w-full bg-background">
        <AppSidebar />
        <div className="flex flex-1 flex-col min-w-0">
          <Header />
          <main className="flex-1 p-4">
            <Outlet />
          </main>
        </div>
        {panelOpen && <ActivityPanel />}
      </div>
      <GlobalSearch />
    </SidebarProvider>
  )
}
