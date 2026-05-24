import { HashRouter, Navigate, Route, Routes } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ArcheClient } from '@/api/generated/client'
import { setClient } from '@/api/generated/hooks'
import { AppShell } from '@/components/AppShell'
import { Toaster } from '@/components/ui/toaster'
import Affixes from '@/pages/Affixes'
import AuditLog from '@/pages/AuditLog'
import Blueprints from '@/pages/Blueprints'
import Clients from '@/pages/Clients'
import Dashboard from '@/pages/Dashboard'
import ExportPage from '@/pages/Export'
import GlobalMetaAttributes from '@/pages/GlobalMetaAttributes'
import ImportPage from '@/pages/Import'
import Login from '@/pages/Login'

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: false,
      refetchOnWindowFocus: false,
    },
  },
})

const archeClient = new ArcheClient({
  baseUrl: import.meta.env.VITE_ARCHE_API_URL || 'http://localhost:3000',
})
setClient(archeClient)

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <HashRouter>
        <Routes>
          <Route path="/login" element={<Login />} />
          <Route element={<AppShell />}>
            <Route path="/dashboard" element={<Dashboard />} />
            <Route path="/blueprints" element={<Blueprints />} />
            <Route path="/affixes" element={<Affixes />} />
            <Route path="/global-meta-attributes" element={<GlobalMetaAttributes />} />
            <Route path="/clients" element={<Clients />} />
            <Route path="/audit-log" element={<AuditLog />} />
            <Route path="/export" element={<ExportPage />} />
            <Route path="/import" element={<ImportPage />} />
            <Route path="*" element={<Navigate to="/dashboard" replace />} />
          </Route>
        </Routes>
        <Toaster />
      </HashRouter>
    </QueryClientProvider>
  )
}
