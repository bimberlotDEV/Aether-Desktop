import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { describe, expect, it, vi } from 'vitest'
import type {
  ExternalEvent,
  SchoolCalendarSource,
  SchoolSchedule as SchoolScheduleData,
} from '@/lib/db/types'

const state = vi.hoisted(() => ({
  data: null as SchoolScheduleData | null,
  setSourceAssociated: vi.fn(),
  selectSourceGroup: vi.fn(),
}))

vi.mock('@/hooks/useSchoolSchedule', () => ({
  useSchoolSchedule: () => ({
    data: state.data,
    loading: false,
    error: null,
    isTauri: true,
    reload: vi.fn(),
    setSourceAssociated: state.setSourceAssociated,
    selectSourceGroup: state.selectSourceGroup,
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
    title,
    time_kind: 'timed',
    start_at_utc: start.toISOString(),
    end_at_utc: end.toISOString(),
    start_date: null,
    end_date: null,
    location: 'Room 2.14',
    status: 'active',
    ...overrides,
  }
}

function source(overrides: Partial<SchoolCalendarSource> = {}): SchoolCalendarSource {
  return {
    connection_id: 'school-source',
    provider_id: 'my_timetable',
    display_name: 'MyTimetable',
    associated: true,
    enabled: true,
    connection_status: 'connected',
    sync_status: 'succeeded',
    last_successful_sync_at: new Date(2026, 8, 23, 9, 30).toISOString(),
    last_sync_error_code: null,
    last_sync_error_message: null,
    selected_groups: ['ADSAI-ZM-1.a'],
    group_options: ['ADSAI-ZM-1.a', 'ADSAI-ZM-2.a'],
    group_selection_valid: true,
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
  it('preserves Today, Week, Upcoming, cancellation, and overlap behavior', async () => {
    state.data = {
      sources: [source()],
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
    expect(screen.queryAllByText('Overlap')).toHaveLength(0)
    expect(screen.queryByText('Datalab')).not.toBeInTheDocument()

    await userEvent.click(screen.getByRole('tab', { name: 'Week' }))
    expect(screen.getByText('Datalab')).toBeInTheDocument()
    await userEvent.click(screen.getByRole('tab', { name: 'Upcoming' }))
    expect(screen.getByText('Programming')).toBeInTheDocument()
  })

  it('requires an explicit source before exposing scoped groups', async () => {
    state.setSourceAssociated.mockClear()
    state.data = {
      events: [],
      sources: [
        source({
          connection_id: 'mtt-x',
          associated: false,
          selected_groups: [],
          group_options: [],
          group_selection_valid: false,
        }),
      ],
    }
    renderSchedule()

    expect(screen.getByText('Finish timetable setup')).toBeInTheDocument()
    expect(screen.queryByRole('combobox')).not.toBeInTheDocument()
    await userEvent.click(screen.getByRole('checkbox', { name: /MyTimetable/ }))
    expect(state.setSourceAssociated).toHaveBeenCalledWith('mtt-x', true)
  })

  it('selects a group only for its associated MyTimetable connection', async () => {
    state.selectSourceGroup.mockClear()
    state.data = {
      events: [],
      sources: [
        source({
          connection_id: 'mtt-x',
          selected_groups: [],
          group_selection_valid: false,
        }),
      ],
    }
    renderSchedule()

    const selector = screen.getByRole('combobox', {
      name: 'Group for MyTimetable',
    })
    expect(selector).toHaveTextContent('ADSAI-ZM-1.a')
    await userEvent.selectOptions(selector, 'ADSAI-ZM-1.a')
    expect(state.selectSourceGroup).toHaveBeenCalledWith('mtt-x', 'ADSAI-ZM-1.a')
  })

  it('distinguishes same-provider connections without exposing feed details', () => {
    state.data = {
      events: [],
      sources: [
        source({
          connection_id: 'mtt-x',
          associated: false,
          selected_groups: [],
          group_options: [],
        }),
        source({
          connection_id: 'mtt-y',
          associated: false,
          selected_groups: [],
          group_options: [],
        }),
      ],
    }
    renderSchedule()
    expect(screen.getByText('MyTimetable connection 1')).toBeInTheDocument()
    expect(screen.getByText('MyTimetable connection 2')).toBeInTheDocument()
    expect(screen.queryByText(/https?:\/\//)).not.toBeInTheDocument()
  })

  it('shows disabled cached MyTimetable state', () => {
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
        source({
          enabled: false,
          connection_status: 'degraded',
          sync_status: 'failed',
          last_sync_error_code: 'network',
          last_sync_error_message: 'Offline',
        }),
      ],
    }
    renderSchedule()
    expect(screen.getByText(/disconnected or disabled/)).toBeInTheDocument()
    expect(screen.getByText('Cached lesson')).toBeInTheDocument()
    expect(screen.getByText('Disabled')).toBeInTheDocument()
  })
})
