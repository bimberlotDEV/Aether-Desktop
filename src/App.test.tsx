import { act, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { UserProfile } from '@/lib/db/types'

const initializeProfile = vi.hoisted(() => vi.fn())

vi.mock('@/lib/db/tauri', () => ({ initializeProfile }))
vi.mock('@/components/Sidebar', () => ({ Sidebar: () => <nav>Sidebar</nav> }))
vi.mock('@/components/CommandPalette', () => ({ CommandPalette: () => null }))
vi.mock('@/routes/Pulse', () => ({ Pulse: () => <p>Pulse workspace</p> }))
vi.mock('@/components/Onboarding', () => ({
  START_ONBOARDING_TOUR_EVENT: 'aether:start-onboarding-tour',
  Onboarding: ({
    tourMode,
    onComplete,
  }: {
    tourMode?: boolean
    onComplete: (profile: UserProfile | null) => void
  }) => (
    <div>
      <p>{tourMode ? 'Workspace tour' : 'First-run onboarding'}</p>
      <button onClick={() => onComplete(completedProfile)}>Complete onboarding</button>
    </div>
  ),
}))

import { App } from '@/App'

const completedProfile: UserProfile = {
  id: 'profile-1',
  display_name: null,
  onboarding_completed: true,
  created_at: '2026-08-29T00:00:00Z',
  updated_at: '2026-08-29T00:00:00Z',
}

describe('App onboarding gate', () => {
  beforeEach(() => {
    initializeProfile.mockReset()
    Object.defineProperty(window, '__TAURI_INTERNALS__', {
      configurable: true,
      value: {},
    })
  })

  it('opens an upgraded or completed workspace without onboarding', async () => {
    initializeProfile.mockResolvedValue(completedProfile)
    render(
      <MemoryRouter>
        <App />
      </MemoryRouter>,
    )

    expect(screen.getByText(/opening your local workspace/i)).toBeVisible()
    expect(await screen.findByText('Pulse workspace')).toBeVisible()
  })

  it('gates a genuinely fresh workspace behind onboarding', async () => {
    initializeProfile.mockResolvedValue({
      ...completedProfile,
      onboarding_completed: false,
    })
    render(
      <MemoryRouter>
        <App />
      </MemoryRouter>,
    )

    expect(await screen.findByText('First-run onboarding')).toBeVisible()
    fireEvent.click(screen.getByRole('button', { name: /complete onboarding/i }))
    expect(await screen.findByText('Pulse workspace')).toBeVisible()
  })

  it('surfaces initialization failure and retries safely', async () => {
    initializeProfile
      .mockRejectedValueOnce(new Error('Database unavailable'))
      .mockResolvedValueOnce(completedProfile)
    render(
      <MemoryRouter>
        <App />
      </MemoryRouter>,
    )

    expect(await screen.findByRole('alert')).toHaveTextContent('Database unavailable')
    fireEvent.click(screen.getByRole('button', { name: /try again/i }))
    expect(await screen.findByText('Pulse workspace')).toBeVisible()
    expect(initializeProfile).toHaveBeenCalledTimes(2)
  })

  it('opens and closes the non-destructive tour from a shell event', async () => {
    initializeProfile.mockResolvedValue(completedProfile)
    render(
      <MemoryRouter>
        <App />
      </MemoryRouter>,
    )
    await screen.findByText('Pulse workspace')

    act(() => window.dispatchEvent(new Event('aether:start-onboarding-tour')))
    expect(screen.getByText('Workspace tour')).toBeVisible()
    fireEvent.click(screen.getByRole('button', { name: /complete onboarding/i }))
    await waitFor(() => expect(screen.getByText('Pulse workspace')).toBeVisible())
  })
})
