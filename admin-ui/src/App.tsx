import { HashRouter, Navigate, Route, Routes } from 'react-router-dom'

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

function AppShell() {
  return (
    <div className="min-h-screen bg-background text-foreground">
      <header className="border-b px-4 py-2 flex items-center gap-4">
        <h1 className="text-lg font-bold">Arche Admin</h1>
      </header>
      <main className="p-4">
        <Routes>
          <Route path="/login" element={<Login />} />
          <Route path="/dashboard" element={<Dashboard />} />
          <Route path="/blueprints" element={<Blueprints />} />
          <Route path="/affixes" element={<Affixes />} />
          <Route path="/global-meta-attributes" element={<GlobalMetaAttributes />} />
          <Route path="/clients" element={<Clients />} />
          <Route path="/audit-log" element={<AuditLog />} />
          <Route path="/export" element={<ExportPage />} />
          <Route path="/import" element={<ImportPage />} />
          <Route path="*" element={<Navigate to="/dashboard" replace />} />
        </Routes>
      </main>
    </div>
  )
}

export default function App() {
  return (
    <HashRouter>
      <AppShell />
      <Toaster />
    </HashRouter>
  )
}
