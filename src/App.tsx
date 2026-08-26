import { lazy, Suspense } from 'react'
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

const Sources = lazy(() =>
  import('@/routes/Sources').then((module) => ({ default: module.Sources })),
)

export function App() {
  return (
    <div className="aether-shell">
      <Sidebar />
      <main className="aether-main">
        <Routes>
          <Route path="/" element={<Pulse />} />
          <Route path="/spaces" element={<Spaces />} />
          <Route path="/spaces/archived" element={<ArchivedSpaces />} />
          <Route path="/spaces/:spaceId/*" element={<SpaceDetailLayout />} />
          <Route path="/tasks" element={<Tasks />} />
          <Route path="/vault" element={<Vault />} />
          <Route
            path="/sources"
            element={
              <Suspense
                fallback={
                  <div className="aether-page p-8 text-sm text-[var(--color-text-tertiary)]">
                    Loading Sources…
                  </div>
                }
              >
                <Sources />
              </Suspense>
            }
          />
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
