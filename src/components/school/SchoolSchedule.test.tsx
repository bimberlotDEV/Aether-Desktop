import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { describe, expect, it, vi } from 'vitest'
import type { ExternalEvent, SchoolSchedule as SchoolScheduleData } from '@/lib/db/types'

const state = vi.hoisted(() => ({
  data: null as SchoolScheduleData | null,
}))

vi.mock('@/hooks/useSchoolSchedule', () => ({
  useSchoolSchedule: () => ({
    data: state.data,
    loading: false,
    error: null,
    isTauri: true,
    reload: vi.fn(),
  }),
}))

import { SchoolSchedule } from '@/components/school/SchoolSchedule'

function event(
  id: string,
  title: string,
  start: Date,
  end: Date,
  overrides: Partial<ExternalEvent> = {},
): ExternalEvent {
  return {
    id,
    connection_id: 'school-source',
    external_id: id,
    occurrence_id: null,
    title,
    description: null,
    time_kind: 'timed',
    start_at_utc: start.toISOString(),
    end_at_utc: end.toISOString(),
    start_date: null,
    end_date: null,
    timezone: 'Europe/Berlin',
    location: 'Room 2.14',
    course_reference: null,
    event_kind: 'lesson',
    status: 'active',
    source_url: null,
    ingestion_provenance: 'ics',
    source_version: '1',
    content_hash: 'hash',
    first_seen_at: '2026-09-23T08:00:00Z',
    last_seen_at: '2026-09-23T08:00:00Z',
    synchronized_at: '2026-09-23T08:00:00Z',
    created_at: '2026-09-23T08:00:00Z',
    updated_at: '2026-09-23T08:00:00Z',
    ...overrides,
  }
}

const now = new Date(2026, 8, 23, 10, 0)

function renderSchedule() {
  return render(
    <MemoryRouter>
      <SchoolSchedule spaceId="school" now={now} />
    </MemoryRouter>,
  )
}

describe('School schedule', () => {
  it('defaults to Today and switches locally between representative views', async () => {
    state.data = {
      sources: [
        {
          connection_id: 'school-source',
          enabled: true,
          connection_status: 'connected',
          sync_status: 'succeeded',
          last_successful_sync_at: new Date(2026, 8, 23, 9, 30).toISOString(),
          last_sync_error_code: null,
          last_sync_error_message: null,
        },
      ],
      events: [
        event(
          'today-a',
          'Programming',
          new Date(2026, 8, 23, 9),
          new Date(2026, 8, 23, 11),
        ),
        event(
          'today-b',
          'Machine Learning',
          new Date(2026, 8, 23, 10),
          new Date(2026, 8, 23, 12),
          { status: 'cancelled' },
        ),
        event('tomorrow', 'Datalab', new Date(2026, 8, 24, 9), new Date(2026, 8, 24, 10)),
      ],
    }
    renderSchedule()

    expect(screen.getByRole('tab', { name: 'Today' })).toHaveAttribute(
      'aria-selected',
      'true',
    )
    expect(screen.getByText('Programming')).toBeInTheDocument()
    expect(screen.getByText('Cancelled')).toBeInTheDocument()
    expect(screen.queryByText('Datalab')).not.toBeInTheDocument()
    expect(screen.getAllByText('Room 2.14')).toHaveLength(2)

    await userEvent.click(screen.getByRole('tab', { name: 'Week' }))
    expect(screen.getByText('Datalab')).toBeInTheDocument()
    expect(screen.queryAllByText('Overlap')).toHaveLength(0)

    await userEvent.click(screen.getByRole('tab', { name: 'Upcoming' }))
    expect(screen.getByText('Programming')).toBeInTheDocument()
    expect(screen.getByText('Datalab')).toBeInTheDocument()
  })

  it('shows every active overlap and an honest stale empty state', async () => {
    state.data = {
      sources: [
        {
          connection_id: 'school-source',
          enabled: true,
          connection_status: 'connected',
          sync_status: 'succeeded',
          last_successful_sync_at: new Date(2026, 8, 22, 8).toISOString(),
          last_sync_error_code: null,
          last_sync_error_message: null,
        },
      ],
      events: [],
    }
    const view = renderSchedule()
    expect(screen.getByText(/may be out of date/)).toBeInTheDocument()
    expect(screen.getByText('No school events today.')).toBeInTheDocument()

    state.data = {
      ...state.data,
      events: [
        event('one', 'First lesson', new Date(2026, 8, 23, 9), new Date(2026, 8, 23, 11)),
        event(
          'two',
          'Second lesson',
          new Date(2026, 8, 23, 10),
          new Date(2026, 8, 23, 12),
        ),
      ],
    }
    view.rerender(
      <MemoryRouter>
        <SchoolSchedule spaceId="school" now={now} />
      </MemoryRouter>,
    )
    expect(screen.getAllByText('Overlap')).toHaveLength(2)
    expect(screen.getByText('First lesson')).toBeInTheDocument()
    expect(screen.getByText('Second lesson')).toBeInTheDocument()
  })

  it('shows disconnected and unavailable connection states without pretending data is current', () => {
    state.data = { events: [], sources: [] }
    const view = renderSchedule()
    expect(screen.getByText('No School calendar is connected.')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'Open Connections' })).toHaveAttribute(
      'href',
      '/settings',
    )

    state.data = {
      events: [],
      sources: [
        {
          connection_id: 'school-source',
          enabled: false,
          connection_status: 'disconnected',
          sync_status: 'idle',
          last_successful_sync_at: null,
          last_sync_error_code: null,
          last_sync_error_message: null,
        },
      ],
    }
    view.rerender(
      <MemoryRouter>
        <SchoolSchedule spaceId="school" now={now} />
      </MemoryRouter>,
    )
    expect(screen.getByText(/disconnected or disabled/)).toBeInTheDocument()
  })

  it('keeps cached events visible when the latest synchronization failed', () => {
    state.data = {
      events: [
        event(
          'cached',
          'Cached lesson',
          new Date(2026, 8, 23, 13),
          new Date(2026, 8, 23, 14),
        ),
      ],
      sources: [
        {
          connection_id: 'school-source',
          enabled: true,
          connection_status: 'degraded',
          sync_status: 'failed',
          last_successful_sync_at: new Date(2026, 8, 22, 8).toISOString(),
          last_sync_error_code: 'network',
          last_sync_error_message: 'Offline',
        },
      ],
    }
    renderSchedule()
    expect(screen.getByText(/latest calendar sync failed/)).toBeInTheDocument()
    expect(screen.getByText('Cached lesson')).toBeInTheDocument()
  })
})
