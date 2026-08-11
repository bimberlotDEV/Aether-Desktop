import { Navigate, Routes, Route } from 'react-router-dom'
import { Sidebar } from '@/components/Sidebar'
import { CommandPalette } from '@/components/CommandPalette'
import { Pulse } from '@/routes/Pulse'
import { Spaces } from '@/routes/Spaces'
import { ArchivedSpaces } from '@/routes/ArchivedSpaces'
import { SpaceDetailLayout } from '@/routes/SpaceDetail'
import { Vault } from '@/routes/Vault'
import { Activity } from '@/routes/Activity'
import { Settings } from '@/routes/Settings'
import { Tasks } from '@/routes/Tasks'
import { AI } from '@/routes/AI'
import { Memory } from '@/routes/Memory'

export function App() {
  return (
    <div
      className="flex h-screen w-screen overflow-hidden"
      style={{ backgroundColor: 'var(--color-bg)' }}
    >
      <Sidebar />
      <main className="flex-1 flex flex-col min-w-0 overflow-hidden">
        <Routes>
          <Route path="/" element={<Pulse />} />
          <Route path="/spaces" element={<Spaces />} />
          <Route path="/spaces/archived" element={<ArchivedSpaces />} />
          <Route path="/spaces/:spaceId/*" element={<SpaceDetailLayout />} />
          <Route path="/tasks" element={<Tasks />} />
          <Route path="/vault" element={<Vault />} />
          <Route path="/ai" element={<AI />} />
          <Route path="/memory" element={<Memory />} />
          <Route path="/activity" element={<Activity />} />
          <Route path="/settings" element={<Settings />} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </main>
      <CommandPalette />
    </div>
  )
}
