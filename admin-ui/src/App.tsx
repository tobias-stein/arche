import { HashRouter, Navigate, Route, Routes } from 'react-router-dom'
import { Moon, Sun } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Toaster } from '@/components/ui/toaster'
import { useTheme } from '@/stores/theme'
import Affixes from '@/pages/Affixes'
import AuditLog from '@/pages/AuditLog'
import Blueprints from '@/pages/Blueprints'
import Clients from '@/pages/Clients'
import Dashboard from '@/pages/Dashboard'
import ExportPage from '@/pages/Export'
import GlobalMetaAttributes from '@/pages/GlobalMetaAttributes'
import ImportPage from '@/pages/Import'
import Login from '@/pages/Login'

function ThemeToggle() {
  const { mode, toggle } = useTheme()
  return (
    <Button
      variant="ghost"
      size="icon"
      onClick={toggle}
      aria-label={mode === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}
    >
      {mode === 'dark' ? <Sun className="h-[1.2rem] w-[1.2rem]" /> : <Moon className="h-[1.2rem] w-[1.2rem]" />}
    </Button>
  )
}

function AppShell() {
  return (
    <div className="min-h-screen bg-background text-foreground">
      <header className="border-b px-4 py-2 flex items-center gap-4">
        <h1 className="text-lg font-bold">Arche Admin</h1>
        <div className="ml-auto flex items-center gap-2">
          <ThemeToggle />
        </div>
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
