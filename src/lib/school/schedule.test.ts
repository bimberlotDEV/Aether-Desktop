import { describe, expect, it } from 'vitest'
import type { ExternalEvent, SchoolCalendarSource } from '@/lib/db/types'
import {
  eventOccursOn,
  overlappingEventIds,
  scheduleFreshness,
  schoolScheduleRequest,
  todayEvents,
  upcomingEvents,
  weekDays,
} from '@/lib/school/schedule'

function event(
  id: string,
  start: Date,
  end: Date,
  overrides: Partial<ExternalEvent> = {},
): ExternalEvent {
  return {
    id,
    connection_id: 'school-source',
    external_id: id,
    occurrence_id: null,
    title: id,
    description: null,
    time_kind: 'timed',
    start_at_utc: start.toISOString(),
    end_at_utc: end.toISOString(),
    start_date: null,
    end_date: null,
    timezone: 'Europe/Berlin',
    location: null,
    course_reference: null,
    group_references: ['ADSAI-ZM-1.a'],
    event_kind: 'lesson',
    status: 'active',
    source_url: null,
    ingestion_provenance: 'ics',
    source_version: '1',
    content_hash: 'hash',
    first_seen_at: '2026-09-23T06:00:00Z',
    last_seen_at: '2026-09-23T06:00:00Z',
    synchronized_at: '2026-09-23T06:00:00Z',
    created_at: '2026-09-23T06:00:00Z',
    updated_at: '2026-09-23T06:00:00Z',
    ...overrides,
  }
}

function source(overrides: Partial<SchoolCalendarSource> = {}): SchoolCalendarSource {
  return {
    connection_id: 'school-source',
    enabled: true,
    connection_status: 'connected',
    sync_status: 'succeeded',
    last_successful_sync_at: '2026-09-23T09:30:00Z',
    last_sync_error_code: null,
    last_sync_error_message: null,
    ...overrides,
  }
}

describe('School schedule local presentation', () => {
  const now = new Date(2026, 8, 23, 12, 0)

  it('builds one bounded local query spanning the current week and upcoming horizon', () => {
    const request = schoolScheduleRequest('school', now)
    expect(request.spaceId).toBe('school')
    expect(request.startDate).toBe('2026-09-21')
    expect(request.endDate).toBe('2026-12-23')
    expect(request.limit).toBe(250)
    expect(new Date(request.startUtc).getHours()).toBe(0)
    expect(new Date(request.endUtc).getHours()).toBe(0)
  })

  it('uses local day boundaries for today and preserves all-day semantics', () => {
    const before = event(
      'before',
      new Date(2026, 8, 22, 23, 0),
      new Date(2026, 8, 22, 23, 30),
    )
    const crossing = event(
      'crossing',
      new Date(2026, 8, 22, 23, 30),
      new Date(2026, 8, 23, 0, 30),
    )
    const allDay = event('all-day', now, now, {
      time_kind: 'all_day',
      start_at_utc: null,
      end_at_utc: null,
      start_date: '2026-09-23',
      end_date: '2026-09-24',
    })
    expect(todayEvents([before, crossing, allDay], now).map((value) => value.id)).toEqual(
      ['all-day', 'crossing'],
    )
    expect(eventOccursOn(allDay, now)).toBe(true)
    expect(eventOccursOn(allDay, new Date(2026, 8, 24))).toBe(false)
  })

  it('returns a Monday-first week and retains every overlapping event', () => {
    expect(weekDays(now).map((day) => day.getDay())).toEqual([1, 2, 3, 4, 5, 6, 0])
    const first = event('first', new Date(2026, 8, 23, 9), new Date(2026, 8, 23, 11))
    const second = event('second', new Date(2026, 8, 23, 10), new Date(2026, 8, 23, 12))
    const cancelled = event(
      'cancelled',
      new Date(2026, 8, 23, 10),
      new Date(2026, 8, 23, 12),
      { status: 'cancelled' },
    )
    expect([...overlappingEventIds([first, second, cancelled])].sort()).toEqual([
      'first',
      'second',
    ])
  })

  it('keeps upcoming chronological and bounded while retaining cancellations', () => {
    const values = Array.from({ length: 45 }, (_, index) =>
      event(
        `event-${index}`,
        new Date(2026, 8, 24 + index, 9),
        new Date(2026, 8, 24 + index, 10),
        index === 2 ? { status: 'cancelled' } : {},
      ),
    )
    const result = upcomingEvents(values.reverse(), now)
    expect(result).toHaveLength(40)
    expect(result[0].id).toBe('event-0')
    expect(result[2].status).toBe('cancelled')
  })

  it('classifies fresh, stale, failed, disconnected, and unavailable sources truthfully', () => {
    const instant = new Date('2026-09-23T10:00:00Z')
    expect(scheduleFreshness([source()], instant)).toBe('fresh')
    expect(
      scheduleFreshness(
        [source({ last_successful_sync_at: '2026-09-23T06:00:00Z' })],
        instant,
      ),
    ).toBe('stale')
    expect(scheduleFreshness([source({ sync_status: 'failed' })], instant)).toBe('error')
    expect(
      scheduleFreshness(
        [source({ enabled: false, connection_status: 'disconnected' })],
        instant,
      ),
    ).toBe('disconnected')
    expect(scheduleFreshness([], instant)).toBe('unavailable')
    expect(scheduleFreshness([source({ last_successful_sync_at: null })], instant)).toBe(
      'unavailable',
    )
  })
})
