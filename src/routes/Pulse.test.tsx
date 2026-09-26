import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter, Route, Routes, useLocation } from 'react-router-dom'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { PulseEvent, PulseSnapshot } from '@/lib/db/types'

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

const current: PulseEvent = {
  id: 'class',
  sourceId: 'mtt',
  sourceLabel: 'Study timetable',
  sourceType: 'school_calendar',
  title: 'Data systems',
  timeKind: 'timed',
  startAt: '2026-09-26T10:00:00Z',
  endAt: '2026-09-26T11:00:00Z',
  startDate: null,
  endDate: null,
  location: 'B2.14',
  cancelled: false,
}

const next: PulseEvent = {
  ...current,
  id: 'lab',
  title: 'Studio lab',
  startAt: '2026-09-26T12:00:00Z',
  endAt: '2026-09-26T13:00:00Z',
}

const snapshot: PulseSnapshot = {
  generatedAt: '2026-09-26T10:30:00Z',
  localDate: '2026-09-26',
  now: [current],
  next,
  today: [current, next],
  upcoming: [{ ...next, id: 'tomorrow', title: 'Tomorrow seminar', startAt: '2026-09-27T08:00:00Z' }],
  tasks: [
    {
      id: 'late',
      title: 'Send invoice',
      spaceId: 'work',
      spaceName: 'Work',
      dueDate: '2026-09-25',
      priority: 'high',
      category: 'overdue',
      destination: '/tasks',
    },
  ],
  conflicts: [{ id: 'class:lab', first: current, second: next }],
  continuity: [
    {
      id: 'work',
      name: 'Work',
      reason: 'Recent Note work',
      lastWorkedAt: '2026-09-26 10:00:00',
      destination: '/spaces/work',
    },
  ],
  trust: [
    {
      sourceId: 'mtt',
      sourceLabel: 'Study timetable',
      providerLabel: 'MyTimetable',
      enabled: true,
      state: 'stale',
      freshness: 'stale',
      lastSuccessfulSyncAt: '2026-09-26T06:00:00Z',
      lastAttemptedAt: '2026-09-26T06:00:00Z',
      errorCategory: null,
      showingCachedData: true,
    },
  ],
  academicDeadlinesAvailable: false,
  issues: [],
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

describe('connected Pulse 2.0', () => {
  beforeEach(() => {
    reload.mockClear()
    pulseState = { data: snapshot, loading: false, error: null, isDesktop: true, reload }
  })

  it('renders now, next, schedule, tasks, conflicts, continuity, and trust', async () => {
    const user = userEvent.setup()
    renderPulse()

    expect(screen.getByText('Now')).toBeInTheDocument()
    expect(screen.getAllByText('Data systems').length).toBeGreaterThan(0)
    expect(screen.getAllByText('Studio lab').length).toBeGreaterThan(0)
    expect(screen.getByText('Send invoice')).toBeInTheDocument()
    expect(screen.getByText(/Data systems overlaps Studio lab/)).toBeInTheDocument()
    expect(screen.getByText('Recent Note work')).toBeInTheDocument()
    expect(screen.getByText(/Stale · showing cached data/)).toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: 'Open Tasks' }))
    expect(screen.getByTestId('location')).toHaveTextContent('/tasks')
  })

  it('shows one intentional empty overview without fake records', () => {
    pulseState = {
      ...pulseState,
      data: {
        ...snapshot,
        now: [],
        next: null,
        today: [],
        upcoming: [],
        tasks: [],
        conflicts: [],
        continuity: [],
        trust: [],
      },
    }
    renderPulse()
    expect(screen.getByText(/Nothing needs your attention/)).toBeInTheDocument()
    expect(screen.getByText(/No timed event is happening now/)).toBeInTheDocument()
  })

  it('surfaces sanitized section degradation while retaining available data', () => {
    pulseState = {
      ...pulseState,
      data: {
        ...snapshot,
        issues: [
          { section: 'schedule', state: 'degraded', message: 'schedule is temporarily unavailable' },
        ],
      },
    }
    renderPulse()
    expect(screen.getByRole('status')).toHaveTextContent('schedule is temporarily unavailable')
    expect(screen.getByText('Send invoice')).toBeInTheDocument()
  })

  it('never fabricates persisted state in browser mode', () => {
    pulseState = { data: null, loading: false, error: null, isDesktop: false, reload }
    renderPulse()
    expect(screen.getByText(/installed app/i)).toBeInTheDocument()
    expect(screen.queryByText('Send invoice')).not.toBeInTheDocument()
  })

  it('shows an intentional loading state', () => {
    pulseState = { data: null, loading: true, error: null, isDesktop: true, reload }
    renderPulse()
    expect(screen.getByRole('status')).toHaveTextContent(/Building today’s local snapshot/)
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
