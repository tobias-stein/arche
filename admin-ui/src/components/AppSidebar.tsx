import { useLocation, useNavigate } from 'react-router-dom'
import {
  Box,
  Building2,
  Check,
  ChevronDown,
  Download,
  Globe,
  History,
  Key,
  LayoutDashboard,
  LogOut,
  Moon,
  PanelLeft,
  PanelRightClose,
  PanelRightOpen,
  Puzzle,
  Search,
  Sun,
  Upload,
} from 'lucide-react'
import { useAuth } from '@/stores/auth'
import { useUi } from '@/stores/ui'
import { useTheme } from '@/stores/theme'
import { useClient, useClientsList } from '@/api/generated/hooks'
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
  useSidebar,
} from '@/components/ui/sidebar'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { Badge } from '@/components/ui/badge'

const navItems = [
  { to: '/dashboard', icon: LayoutDashboard, label: 'Dashboard' },
  { to: '/blueprints', icon: Box, label: 'Blueprints' },
  { to: '/affixes', icon: Puzzle, label: 'Affixes' },
  { to: '/global-meta-attributes', icon: Globe, label: 'Global Meta Attributes' },
  { to: '/audit-log', icon: History, label: 'Audit Log' },
  { to: '/import', icon: Upload, label: 'Import' },
  { to: '/export', icon: Download, label: 'Export' },
  { to: '/clients', icon: Key, label: 'API Keys' },
]

export function AppSidebar() {
  const location = useLocation()
  const navigate = useNavigate()
  const { toggleSidebar } = useSidebar()
  const { isSuperAdmin, isAuthenticated, keyName, logout } = useAuth()
  const { selectedClientId, setSelectedClientId, panelOpen, togglePanel, openSearch } = useUi()
  const { mode, toggle: toggleTheme } = useTheme()
  const { data: clientsData, isLoading: clientsLoading } = useClientsList()
  const { data: clientData } = useClient(
    !isSuperAdmin && selectedClientId ? selectedClientId : '',
  )

  const handleLogout = () => {
    logout()
  }

  const isActive = (to: string) => {
    if (to === '/dashboard') return location.pathname === to
    return location.pathname.startsWith(to)
  }

  const visibleNavItems = navItems.filter((item) => {
    if (item.to === '/import' || item.to === '/export') return isSuperAdmin
    return true
  })

  const displayName = isSuperAdmin ? 'Super Admin' : (keyName ?? 'API Key')

  return (
    <Sidebar collapsible="icon">
      <SidebarHeader className="relative">
        <button
          onClick={toggleSidebar}
          className="absolute -right-8 top-2 z-50 text-sidebar-foreground/30 hover:text-sidebar-foreground transition-colors"
          aria-label="Toggle sidebar"
        >
          <PanelLeft className="h-4 w-4" />
        </button>
        <SidebarMenu>
          {isAuthenticated && isSuperAdmin ? (
            <SidebarMenuItem>
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <SidebarMenuButton tooltip="Switch client">
                    <Building2 className="h-4 w-4 shrink-0" />
                    <span>
                      {clientsLoading
                        ? 'Loading...'
                        : selectedClientId
                          ? clientsData?.data.find((c) => c.id === selectedClientId)?.name
                          : 'All clients'}
                    </span>
                    <ChevronDown className="h-3 w-3 opacity-50 shrink-0 ml-auto" />
                  </SidebarMenuButton>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="start" className="w-56">
                  <DropdownMenuItem onClick={() => setSelectedClientId(null)}>
                    <span>All clients</span>
                    {!selectedClientId && <Check className="h-4 w-4 ml-auto" />}
                  </DropdownMenuItem>
                  {(clientsData?.data ?? []).map((client) => (
                    <DropdownMenuItem
                      key={client.id}
                      onClick={() => setSelectedClientId(client.id)}
                    >
                      <span className="truncate">{client.name}</span>
                      {selectedClientId === client.id && <Check className="h-4 w-4 ml-auto" />}
                    </DropdownMenuItem>
                  ))}
                </DropdownMenuContent>
              </DropdownMenu>
            </SidebarMenuItem>
          ) : isAuthenticated && selectedClientId && clientData ? (
            <SidebarMenuItem>
              <SidebarMenuButton tooltip={clientData.name}>
                <Building2 className="h-4 w-4 shrink-0" />
                <span>{clientData.name}</span>
                <Badge variant="secondary" className="ml-auto text-xs">scoped</Badge>
              </SidebarMenuButton>
            </SidebarMenuItem>
          ) : isAuthenticated && selectedClientId ? (
            <SidebarMenuItem>
              <SidebarMenuButton tooltip="Loading...">
                <Building2 className="h-4 w-4 shrink-0" />
                <span>Loading...</span>
              </SidebarMenuButton>
            </SidebarMenuItem>
          ) : null}
        </SidebarMenu>
      </SidebarHeader>
      <SidebarContent className="p-2">
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton onClick={openSearch} tooltip="Search">
              <Search className="h-4 w-4" />
              <span>Search</span>
              <kbd className="ml-auto text-xs bg-sidebar-accent px-1.5 py-0.5 rounded hidden sm:inline-flex items-center gap-0.5">
                <svg width="15" height="15" viewBox="0 0 15 15" fill="none" xmlns="http://www.w3.org/2000/svg" className="size-3"><path d="M11.4669 2.02344C11.4669 1.59959 11.1222 1.25488 10.6984 1.25488C10.2745 1.25488 9.92983 1.59959 9.92983 2.02344V5.51836L6.50989 2.09841C6.13094 1.71946 5.51734 1.71946 5.13839 2.09841C4.75941 2.47739 4.75941 3.09099 5.13839 3.46994L8.66815 7.00002L5.13839 10.5301C4.75941 10.9091 4.75941 11.5227 5.13839 11.9016C5.51734 12.2806 6.13094 12.2806 6.50989 11.9016L9.92983 8.48169V11.9766C9.92983 12.4004 10.2745 12.7451 10.6984 12.7451C11.1222 12.7451 11.4669 12.4004 11.4669 11.9766V2.02344Z" fill="currentColor" fillRule="evenodd" clipRule="evenodd"></path></svg>
                K
              </kbd>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
        <SidebarMenu>
          {visibleNavItems.map((item) => (
            <SidebarMenuItem key={item.to}>
              <SidebarMenuButton asChild isActive={isActive(item.to)} tooltip={item.label}>
                <a href={`#${item.to}`} onClick={(e) => { e.preventDefault(); navigate(item.to) }}>
                  <item.icon />
                  <span>{item.label}</span>
                </a>
              </SidebarMenuButton>
            </SidebarMenuItem>
          ))}
        </SidebarMenu>
      </SidebarContent>
      <SidebarFooter>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton onClick={toggleTheme} tooltip={mode === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}>
              {mode === 'dark' ? <Sun className="h-4 w-4" /> : <Moon className="h-4 w-4" />}
              <span>{mode === 'dark' ? 'Light mode' : 'Dark mode'}</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <SidebarMenuButton onClick={togglePanel} tooltip={panelOpen ? 'Close activity panel' : 'Open activity panel'}>
              {panelOpen ? <PanelRightClose className="h-4 w-4" /> : <PanelRightOpen className="h-4 w-4" />}
              <span>{panelOpen ? 'Close activity' : 'Activity log'}</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <SidebarMenuButton tooltip={displayName}>
              <Key className="h-4 w-4" />
              <span>{displayName}</span>
            </SidebarMenuButton>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="w-56">
            <DropdownMenuItem onClick={handleLogout}>
              <LogOut className="h-4 w-4" />
              <span>Logout</span>
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  )
}
