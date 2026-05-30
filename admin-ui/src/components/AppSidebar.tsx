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
import { useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useAuth } from '@/stores/auth'
import { useUi } from '@/stores/ui'
import { useTheme } from '@/stores/theme'
import { useClient, getClient } from '@/api/generated/hooks'
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

export function AppSidebar() {
  const location = useLocation()
  const navigate = useNavigate()
  const { toggleSidebar } = useSidebar()
  const { isSuperAdmin, isAuthenticated, keyName, logout, client_id } = useAuth()
  const { selectedClientId, setSelectedClientId, panelOpen, togglePanel, openSearch } = useUi()
  const { mode, toggle: toggleTheme } = useTheme()
  const { data: clientsData, isLoading: clientsLoading } = useQuery({
    queryKey: ['clients', 'list'],
    queryFn: () => getClient().listClients(),
    enabled: isSuperAdmin,
  })
  const { data: clientData } = useClient(
    !isSuperAdmin && selectedClientId ? selectedClientId : '',
  )

  const navItems = useMemo(() => [
    { to: '/dashboard', icon: LayoutDashboard, label: 'Dashboard' },
    { to: '/blueprints', icon: Box, label: 'Blueprints' },
    { to: '/affixes', icon: Puzzle, label: 'Affixes' },
    { to: '/global-meta-attributes', icon: Globe, label: 'Global Meta Attributes' },
    { to: '/audit-log', icon: History, label: 'Audit Log' },
    ...(isSuperAdmin
      ? [{ to: '/import', icon: Upload, label: 'Import' },
         { to: '/export', icon: Download, label: 'Export' }]
      : []),
    { to: isSuperAdmin ? '/clients' : `/clients/${client_id}`, icon: Key, label: 'API Keys' },
  ], [isSuperAdmin, client_id])

  const handleLogout = () => {
    logout()
  }

  const isActive = (to: string) => {
    if (to === '/dashboard') return location.pathname === to
    return location.pathname.startsWith(to)
  }

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
            <SidebarMenuButton variant="outline" onClick={openSearch} tooltip="Search" className="h-9 shadow-none border border-input hover:bg-accent hover:text-accent-foreground">
              <Search className="h-4 w-4 text-muted-foreground" />
              <span className="flex-1 text-left text-muted-foreground">Search</span>
              <kbd className="ml-auto text-xs bg-sidebar-accent px-1.5 py-0.5 rounded items-center gap-0.5 hidden sm:inline-flex">
                <span>⌘</span>
                <span>K</span>
              </kbd>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
        <SidebarMenu>
          {navItems.map((item) => (
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
            <SidebarMenuButton
              onClick={toggleTheme}
              aria-label={mode === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}
              tooltip={mode === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}
            >
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
