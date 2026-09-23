import type {
  ExternalEvent,
  SchoolCalendarSource,
  SchoolScheduleRequest,
} from '@/lib/db/types'

export type SchoolView = 'today' | 'week' | 'upcoming'
export type ScheduleFreshness =
  'fresh' | 'syncing' | 'stale' | 'error' | 'disconnected' | 'unavailable'

const UPCOMING_DAYS = 90
export const UPCOMING_LIMIT = 40
export const SCHOOL_QUERY_LIMIT = 250
export const STALE_AFTER_MS = 2 * 60 * 60 * 1000

function localMidnight(value: Date) {
  return new Date(value.getFullYear(), value.getMonth(), value.getDate())
}

export function addLocalDays(value: Date, days: number) {
  return new Date(value.getFullYear(), value.getMonth(), value.getDate() + days)
}

export function localDateKey(value: Date) {
  const year = value.getFullYear()
  const month = String(value.getMonth() + 1).padStart(2, '0')
  const day = String(value.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

export function startOfLocalWeek(value: Date) {
  const date = localMidnight(value)
  const mondayOffset = (date.getDay() + 6) % 7
  return addLocalDays(date, -mondayOffset)
}

export function schoolScheduleRequest(spaceId: string, now: Date): SchoolScheduleRequest {
  const start = startOfLocalWeek(now)
  const end = addLocalDays(localMidnight(now), UPCOMING_DAYS + 1)
  return {
    spaceId,
    startUtc: start.toISOString(),
    endUtc: end.toISOString(),
    startDate: localDateKey(start),
    endDate: localDateKey(end),
    limit: SCHOOL_QUERY_LIMIT,
  }
}

function timedInterval(event: ExternalEvent) {
  if (event.time_kind !== 'timed' || !event.start_at_utc || !event.end_at_utc) {
    return null
  }
  return {
    start: new Date(event.start_at_utc).getTime(),
    end: new Date(event.end_at_utc).getTime(),
  }
}

export function eventOccursOn(event: ExternalEvent, day: Date) {
  const key = localDateKey(day)
  if (event.time_kind === 'all_day') {
    return (
      !!event.start_date &&
      !!event.end_date &&
      event.start_date <= key &&
      key < event.end_date
    )
  }
  const interval = timedInterval(event)
  if (!interval) return false
  const start = localMidnight(day).getTime()
  const end = addLocalDays(day, 1).getTime()
  return interval.start < end && interval.end > start
}

function eventSortValue(event: ExternalEvent) {
  if (event.time_kind === 'all_day') {
    return event.start_date ? new Date(`${event.start_date}T00:00:00`).getTime() : 0
  }
  return event.start_at_utc ? new Date(event.start_at_utc).getTime() : 0
}

export function sortEvents(events: ExternalEvent[]) {
  return [...events].sort((left, right) => {
    if (left.time_kind !== right.time_kind) return left.time_kind === 'all_day' ? -1 : 1
    return eventSortValue(left) - eventSortValue(right) || left.id.localeCompare(right.id)
  })
}

export function todayEvents(events: ExternalEvent[], now: Date) {
  return sortEvents(events.filter((event) => eventOccursOn(event, now)))
}

export function weekDays(now: Date) {
  const start = startOfLocalWeek(now)
  return Array.from({ length: 7 }, (_, index) => addLocalDays(start, index))
}

export function upcomingEvents(events: ExternalEvent[], now: Date) {
  const today = localDateKey(now)
  return sortEvents(
    events.filter((event) => {
      if (event.time_kind === 'all_day') return !!event.end_date && event.end_date > today
      const interval = timedInterval(event)
      return !!interval && interval.end > now.getTime()
    }),
  ).slice(0, UPCOMING_LIMIT)
}

export function overlappingEventIds(events: ExternalEvent[]) {
  const timed = sortEvents(events).filter(
    (event) => event.status === 'active' && event.time_kind === 'timed',
  )
  const overlaps = new Set<string>()
  for (let leftIndex = 0; leftIndex < timed.length; leftIndex += 1) {
    const left = timedInterval(timed[leftIndex])!
    for (let rightIndex = leftIndex + 1; rightIndex < timed.length; rightIndex += 1) {
      const right = timedInterval(timed[rightIndex])!
      if (right.start >= left.end) break
      if (left.start < right.end && left.end > right.start) {
        overlaps.add(timed[leftIndex].id)
        overlaps.add(timed[rightIndex].id)
      }
    }
  }
  return overlaps
}

export function scheduleFreshness(
  sources: SchoolCalendarSource[],
  now: Date,
): ScheduleFreshness {
  if (sources.length === 0) return 'unavailable'
  if (
    sources.every(
      (source) => !source.enabled || source.connection_status === 'disconnected',
    )
  ) {
    return 'disconnected'
  }
  if (
    sources.some(
      (source) =>
        source.sync_status === 'failed' ||
        [
          'degraded',
          'reauthentication_required',
          'permission_denied',
          'rate_limited',
          'institution_configuration_required',
          'unsupported',
        ].includes(source.connection_status),
    )
  ) {
    return 'error'
  }
  if (
    sources.some(
      (source) => source.sync_status === 'syncing' || source.sync_status === 'pending',
    )
  ) {
    return 'syncing'
  }
  const successful = sources
    .map((source) => source.last_successful_sync_at)
    .filter((value): value is string => !!value)
    .map((value) => new Date(value).getTime())
    .filter(Number.isFinite)
  if (successful.length === 0) return 'unavailable'
  return now.getTime() - Math.max(...successful) > STALE_AFTER_MS ? 'stale' : 'fresh'
}

export function latestSuccessfulSync(sources: SchoolCalendarSource[]) {
  return sources
    .map((source) => source.last_successful_sync_at)
    .filter((value): value is string => !!value)
    .sort()
    .at(-1)
}
