import { NavLink, useNavigate } from 'react-router-dom'
import {
  Box,
  Download,
  Globe,
  History,
  Key,
  LayoutDashboard,
  LogIn,
  LogOut,
  Puzzle,
  Upload,
} from 'lucide-react'
import { useAuth } from '@/stores/auth'
import { useUi } from '@/stores/ui'
import { cn } from '@/lib/utils'

const buttonClass =
  'flex w-full items-center gap-3 px-3 py-2 rounded-md text-sm font-medium text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors'

const navItems = [
  { to: '/dashboard', icon: LayoutDashboard, label: 'Dashboard' },
  { to: '/blueprints', icon: Box, label: 'Blueprints' },
  { to: '/affixes', icon: Puzzle, label: 'Affixes' },
  { to: '/global-meta-attributes', icon: Globe, label: 'Global Meta Attributes' },
  { to: '/audit-log', icon: History, label: 'Audit Log' },
  { to: '/import', icon: Upload, label: 'Import' },
  { to: '/export', icon: Download, label: 'Export' },
]

export function NavigationDrawer() {
  const { drawerOpen, closeDrawer } = useUi()
  const { isAuthenticated, isSuperAdmin, logout } = useAuth()
  const navigate = useNavigate()

  const handleLogout = () => {
    logout()
    closeDrawer()
    navigate('/login')
  }

  const handleLogin = () => {
    closeDrawer()
    navigate('/login')
  }

  const navLinkClass = ({ isActive }: { isActive: boolean }) =>
    cn(
      'flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium transition-colors',
      isActive
        ? 'bg-accent text-accent-foreground'
        : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground',
    )

  const visibleNavItems = navItems.filter((item) => {
    if (item.to === '/import' || item.to === '/export') return isSuperAdmin
    return true
  })

  return (
    <>
      {drawerOpen && (
        <div
          className="fixed inset-0 z-30 bg-black/80 md:hidden"
          onClick={closeDrawer}
          aria-hidden="true"
        />
      )}
      <aside
        className={cn(
          'fixed top-0 left-0 bottom-0 z-40 w-64 border-r bg-background flex flex-col transition-transform duration-300',
          drawerOpen ? 'translate-x-0' : '-translate-x-full',
          'md:relative md:top-auto md:left-auto md:bottom-auto md:z-10 md:transition-[width] md:shrink-0',
          drawerOpen ? 'md:w-64' : 'md:w-0 md:border-r-0 md:overflow-hidden',
        )}
      >
        <nav className="overflow-y-auto px-4 pb-4 space-y-1 pt-20 md:pt-4">
          {visibleNavItems.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              end={item.to === '/dashboard'}
              className={navLinkClass}
              onClick={closeDrawer}
            >
              <item.icon className="h-4 w-4" />
              {item.label}
            </NavLink>
          ))}
        </nav>

        <div className="border-t p-4 space-y-1 mt-auto">
          <NavLink
            to="/clients"
            className={navLinkClass}
            onClick={closeDrawer}
          >
            <Key className="h-4 w-4" />
            API Keys
          </NavLink>
          {isAuthenticated ? (
            <button onClick={handleLogout} className={buttonClass}>
              <LogOut className="h-4 w-4" />
              Logout
            </button>
          ) : (
            <button onClick={handleLogin} className={buttonClass}>
              <LogIn className="h-4 w-4" />
              Login
            </button>
          )}
        </div>
      </aside>
    </>
  )
}
