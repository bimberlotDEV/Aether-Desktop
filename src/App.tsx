import { lazy, Suspense, useCallback, useEffect, useState } from 'react'
import { Navigate, Routes, Route } from 'react-router-dom'
import { Sidebar } from '@/components/Sidebar'
import { CommandPalette } from '@/components/CommandPalette'
import { Pulse } from '@/routes/Pulse'
import { Spaces } from '@/routes/Spaces'
import { ArchivedSpaces } from '@/routes/ArchivedSpaces'
import { Vault } from '@/routes/Vault'
import { Settings } from '@/routes/Settings'
import { Tasks } from '@/routes/Tasks'
import { AI } from '@/routes/AI'
import { Memory } from '@/routes/Memory'
import { Onboarding, START_ONBOARDING_TOUR_EVENT } from '@/components/Onboarding'
import * as db from '@/lib/db/tauri'
import type { UserProfile } from '@/lib/db/types'

const Sources = lazy(() =>
  import('@/routes/Sources').then((module) => ({ default: module.Sources })),
)
const Activity = lazy(() =>
  import('@/routes/Activity').then((module) => ({ default: module.Activity })),
)
const Actions = lazy(() =>
  import('@/routes/Actions').then((module) => ({ default: module.Actions })),
)
const SpaceDetailLayout = lazy(() =>
  import('@/routes/SpaceDetail').then((module) => ({
    default: module.SpaceDetailLayout,
  })),
)

export function App() {
  const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  const [profile, setProfile] = useState<UserProfile | null>(null)
  const [gate, setGate] = useState<'loading' | 'ready' | 'error'>(
    isTauri ? 'loading' : 'ready',
  )
  const [gateError, setGateError] = useState<string | null>(null)
  const [tourMode, setTourMode] = useState(false)

  const initialize = useCallback(async () => {
    if (!isTauri) {
      setGate('ready')
      return
    }
    setGate('loading')
    setGateError(null)
    try {
      setProfile(await db.initializeProfile())
      setGate('ready')
    } catch (cause) {
      setGateError(
        cause instanceof Error && cause.message
          ? cause.message
          : 'Aether could not initialize the local profile.',
      )
      setGate('error')
    }
  }, [isTauri])

  useEffect(() => {
    void initialize()
  }, [initialize])

  useEffect(() => {
    const openTour = () => setTourMode(true)
    window.addEventListener(START_ONBOARDING_TOUR_EVENT, openTour)
    return () => window.removeEventListener(START_ONBOARDING_TOUR_EVENT, openTour)
  }, [])

  if (tourMode) {
    return <Onboarding profile={profile} tourMode onComplete={() => setTourMode(false)} />
  }

  if (gate === 'loading') {
    return (
      <main className="onboarding-shell" aria-busy="true">
        <div className="onboarding-gate-message">
          <span className="onboarding-wordmark">Aether</span>
          <p>Opening your local workspace…</p>
        </div>
      </main>
    )
  }

  if (gate === 'error') {
    return (
      <main className="onboarding-shell">
        <div className="onboarding-gate-message" role="alert">
          <span className="onboarding-wordmark">Aether</span>
          <h1>Could not open the local workspace</h1>
          <p>{gateError}</p>
          <button
            type="button"
            className="aether-button aether-button--primary focus-ring"
            onClick={() => void initialize()}
          >
            Try again
          </button>
        </div>
      </main>
    )
  }

  if (isTauri && profile && !profile.onboarding_completed) {
    return <Onboarding profile={profile} onComplete={setProfile} />
  }

  return <AppShell />
}

function AppShell() {
  return (
    <div className="aether-shell">
      <Sidebar />
      <main className="aether-main">
        <Routes>
          <Route path="/" element={<Pulse />} />
          <Route path="/spaces" element={<Spaces />} />
          <Route path="/spaces/archived" element={<ArchivedSpaces />} />
          <Route
            path="/spaces/:spaceId/*"
            element={
              <Suspense
                fallback={
                  <div className="aether-page p-8 text-sm text-[var(--color-text-tertiary)]">
                    Opening Space…
                  </div>
                }
              >
                <SpaceDetailLayout />
              </Suspense>
            }
          />
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
          <Route
            path="/actions"
            element={
              <Suspense
                fallback={
                  <div className="aether-page p-8 text-sm text-[var(--color-text-tertiary)]">
                    Loading Actions…
                  </div>
                }
              >
                <Actions />
              </Suspense>
            }
          />
          <Route
            path="/activity"
            element={
              <Suspense
                fallback={
                  <div className="aether-page p-8 text-sm text-[var(--color-text-tertiary)]">
                    Loading Activity…
                  </div>
                }
              >
                <Activity />
              </Suspense>
            }
          />
          <Route path="/settings" element={<Settings />} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </main>
      <CommandPalette />
    </div>
  )
}
