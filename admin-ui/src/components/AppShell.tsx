import { Outlet } from 'react-router-dom'
import { Menu, Moon, PanelRightClose, PanelRightOpen, Search, Sun } from 'lucide-react'
import { useTheme } from '@/stores/theme'
import { useUi } from '@/stores/ui'
import { NavigationDrawer } from './NavigationDrawer'
import { ActivityPanel } from './ActivityPanel'
import { GlobalSearch } from './GlobalSearch'

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
  const { drawerOpen, panelOpen, toggleDrawer, togglePanel, openSearch } = useUi()

  return (
    <header className="fixed top-0 left-0 right-0 z-50 h-16 border-b bg-background flex items-center gap-4 px-4">
      <button
        onClick={toggleDrawer}
        className="inline-flex items-center justify-center h-9 w-9 rounded-md hover:bg-accent hover:text-accent-foreground"
        aria-label={drawerOpen ? 'Close navigation' : 'Open navigation'}
      >
        <Menu className="h-5 w-5" />
      </button>
      <span className="text-lg font-bold">Arche Admin</span>
      <div className="flex-1 max-w-md mx-auto">
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
      <div className="flex items-center gap-2">
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
    <div className="min-h-screen bg-background">
      <Header />
      <div className="flex pt-16">
        <NavigationDrawer />
        <main className="flex-1 min-w-0 transition-[margin] duration-300">
          <div className="p-4">
            <Outlet />
          </div>
        </main>
        {panelOpen && <ActivityPanel />}
      </div>
      <GlobalSearch />
    </div>
  )
}
