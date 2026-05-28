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
  Puzzle,
  Upload,
} from 'lucide-react'
import { useAuth } from '@/stores/auth'
import { useUi } from '@/stores/ui'
import { useClient, useClientsList } from '@/api/generated/hooks'
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
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
  const { isSuperAdmin, isAuthenticated, keyName, logout } = useAuth()
  const { selectedClientId, setSelectedClientId } = useUi()
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
      <SidebarHeader>
        {isAuthenticated && isSuperAdmin ? (
          <DropdownMenu>
            <DropdownMenuTrigger className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-sm font-medium hover:bg-sidebar-accent hover:text-sidebar-accent-foreground transition-colors [&[data-state=open]>svg:last-child]:rotate-180">
              <Building2 className="h-4 w-4 shrink-0" />
              <span className="truncate flex-1 text-left">
                {clientsLoading
                  ? 'Loading...'
                  : selectedClientId
                    ? clientsData?.data.find((c) => c.id === selectedClientId)?.name
                    : 'All clients'}
              </span>
              <ChevronDown className="h-3 w-3 opacity-50 shrink-0 transition-transform" />
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
        ) : isAuthenticated && selectedClientId && clientData ? (
          <div className="flex items-center gap-2 px-2 py-1.5">
            <Building2 className="h-4 w-4 shrink-0" />
            <span className="truncate text-sm font-medium">{clientData.name}</span>
            <Badge variant="secondary" className="ml-auto text-xs">scoped</Badge>
          </div>
        ) : isAuthenticated && selectedClientId ? (
          <div className="flex items-center gap-2 px-2 py-1.5">
            <Building2 className="h-4 w-4 shrink-0" />
            <span className="truncate text-sm font-medium">Loading...</span>
          </div>
        ) : null}
      </SidebarHeader>
      <SidebarContent>
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
    </Sidebar>
  )
}
