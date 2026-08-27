import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter, Route, Routes, useLocation } from 'react-router-dom'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { PulseSnapshot } from '@/lib/db/types'

const reload = vi.fn().mockResolvedValue(undefined)
let pulseState: {
  data: PulseSnapshot | null
  loading: boolean
  error: string | null
  isDesktop: boolean
  reload: typeof reload
}

vi.mock('@/hooks/usePulse', () => ({ usePulse: () => pulseState }))

import { Pulse } from '@/routes/Pulse'

const snapshot: PulseSnapshot = {
  today: '2026-08-27',
  overdue: [
    {
      id: 'late',
      title: 'Send invoice',
      spaceId: 'work',
      spaceName: 'Work',
      dueDate: '2026-08-26',
      priority: 'high',
      destination: '/tasks',
    },
  ],
  dueToday: [],
  upcoming: [],
  continueSpaces: [
    {
      id: 'work',
      name: 'Work',
      reason: 'Recent Note work',
      lastWorkedAt: '2026-08-27 10:00:00',
      destination: '/spaces/work',
    },
  ],
  newFiles: [
    {
      id: 'file',
      title: 'brief.pdf',
      detail: 'Work files · briefs/brief.pdf',
      spaceId: 'work',
      spaceName: 'Work',
      detectedAt: '2026-08-27 09:00:00',
      destination: '/sources',
    },
  ],
  recentActivity: [
    {
      id: 'activity',
      eventType: 'note_edited',
      title: 'Edited note Plan',
      detail: null,
      spaceId: 'work',
      spaceName: 'Work',
      entityType: 'note',
      entityId: 'note',
      destination: '/spaces/work/notes',
      createdAt: '2026-08-27 08:00:00',
    },
  ],
  suggestedNextStep: {
    title: 'Continue Send invoice',
    detail: 'This open Task is overdue',
    destination: '/tasks',
    sourceType: 'task',
    sourceId: 'late',
  },
}

function Location() {
  return <span data-testid="location">{useLocation().pathname}</span>
}

function renderPulse() {
  return render(
    <MemoryRouter>
      <Routes>
        <Route
          path="*"
          element={
            <>
              <Pulse />
              <Location />
            </>
          }
        />
      </Routes>
    </MemoryRouter>,
  )
}

describe('Pulse 2.0', () => {
  beforeEach(() => {
    reload.mockClear()
    pulseState = { data: snapshot, loading: false, error: null, isDesktop: true, reload }
  })

  it('shows factual daily sections and navigates the grounded suggestion', async () => {
    const user = userEvent.setup()
    renderPulse()

    expect(screen.getByText('Today')).toBeInTheDocument()
    expect(screen.getByText('Send invoice')).toBeInTheDocument()
    expect(screen.getByText('Recent Note work')).toBeInTheDocument()
    expect(screen.getByText('Work files · briefs/brief.pdf')).toBeInTheDocument()
    expect(screen.getByText('Edited note Plan')).toBeInTheDocument()
    expect(screen.getByText('This open Task is overdue')).toBeInTheDocument()
    expect(
      screen.getByText(/attach only the Notes, Tasks, files, or Memory/i),
    ).toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: 'Continue' }))
    expect(screen.getByTestId('location')).toHaveTextContent('/tasks')
  })

  it('never fabricates persisted state in browser mode', () => {
    pulseState = { data: null, loading: false, error: null, isDesktop: false, reload }
    renderPulse()
    expect(screen.getByText(/installed app/i)).toBeInTheDocument()
    expect(screen.queryByText('Send invoice')).not.toBeInTheDocument()
  })

  it('surfaces read errors and offers a retry', async () => {
    pulseState = {
      data: null,
      loading: false,
      error: 'Database unavailable',
      isDesktop: true,
      reload,
    }
    const user = userEvent.setup()
    renderPulse()
    expect(screen.getByRole('alert')).toHaveTextContent('Database unavailable')
    await user.click(screen.getByRole('button', { name: 'Try again' }))
    expect(reload).toHaveBeenCalledOnce()
  })
})
