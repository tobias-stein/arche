import { HashRouter, Navigate, Route, Routes } from 'react-router-dom'
import { MutationCache, QueryCache, QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ArcheClient } from '@/api/generated/client'
import { setClient } from '@/api/generated/hooks'
import { ApiError } from '@/api/generated/errors'
import { useAuth } from '@/stores/auth'
import { AppShell } from '@/components/AppShell'
import { AuthGuard } from '@/components/AuthGuard'
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

function autoLogoutOnAuthError(error: unknown) {
  if (error instanceof ApiError && (error.status === 401 || error.status === 403)) {
    useAuth.getState().logout()
  }
}

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: false,
      refetchOnWindowFocus: false,
    },
  },
  queryCache: new QueryCache({
    onError: autoLogoutOnAuthError,
  }),
  mutationCache: new MutationCache({
    onError: autoLogoutOnAuthError,
  }),
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
          <Route element={<AuthGuard />}>
            <Route element={<AppShell />}>
              <Route path="/dashboard" element={<Dashboard />} />
              <Route path="/blueprints" element={<Blueprints />} />
              <Route path="/blueprints/:id" element={<Blueprints />} />
              <Route path="/affixes" element={<Affixes />} />
              <Route path="/affixes/:id" element={<Affixes />} />
              <Route path="/global-meta-attributes" element={<GlobalMetaAttributes />} />
              <Route path="/clients" element={<Clients />} />
              <Route path="/audit-log" element={<AuditLog />} />
              <Route path="/export" element={<ExportPage />} />
              <Route path="/import" element={<ImportPage />} />
              <Route path="*" element={<Navigate to="/dashboard" replace />} />
            </Route>
          </Route>
        </Routes>
        <Toaster />
      </HashRouter>
    </QueryClientProvider>
  )
}
