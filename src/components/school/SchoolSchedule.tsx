import { useMemo, useState } from 'react'
import { Link } from 'react-router-dom'
import { AlertTriangle, CalendarDays, Clock3, MapPin } from 'lucide-react'
import { useSchoolSchedule } from '@/hooks/useSchoolSchedule'
import type { ExternalEvent } from '@/lib/db/types'
import {
  eventOccursOn,
  latestSuccessfulSync,
  overlappingEventIds,
  scheduleFreshness,
  sortEvents,
  todayEvents,
  upcomingEvents,
  weekDays,
  type SchoolView,
} from '@/lib/school/schedule'
import { cn } from '@/lib/utils'

const TABS: Array<{ id: SchoolView; label: string }> = [
  { id: 'today', label: 'Today' },
  { id: 'week', label: 'Week' },
  { id: 'upcoming', label: 'Upcoming' },
]

const timeFormatter = new Intl.DateTimeFormat(undefined, {
  hour: '2-digit',
  minute: '2-digit',
})
const dayFormatter = new Intl.DateTimeFormat(undefined, {
  weekday: 'short',
  day: 'numeric',
  month: 'short',
})
const dateTimeFormatter = new Intl.DateTimeFormat(undefined, {
  day: 'numeric',
  month: 'short',
  hour: '2-digit',
  minute: '2-digit',
})

function eventTime(event: ExternalEvent) {
  if (event.time_kind === 'all_day') return 'All day'
  if (!event.start_at_utc || !event.end_at_utc) return 'Time unavailable'
  return `${timeFormatter.format(new Date(event.start_at_utc))}–${timeFormatter.format(new Date(event.end_at_utc))}`
}

function EventCard({
  event,
  conflict = false,
}: {
  event: ExternalEvent
  conflict?: boolean
}) {
  const cancelled = event.status === 'cancelled'
  return (
    <article
      className={cn('rounded-lg border px-3 py-2.5', cancelled && 'opacity-70')}
      style={{
        borderColor: conflict ? 'var(--color-warning)' : 'var(--color-border)',
        background: 'var(--color-bg-secondary)',
      }}
    >
      <div className="flex flex-wrap items-center gap-1.5 text-[0.6875rem] font-medium text-[var(--color-text-tertiary)]">
        <span>{eventTime(event)}</span>
        {cancelled && (
          <span className="rounded px-1.5 py-0.5 text-[var(--color-danger)] bg-[color-mix(in_srgb,var(--color-danger)_10%,transparent)]">
            Cancelled
          </span>
        )}
        {conflict && (
          <span className="inline-flex items-center gap-1 rounded px-1.5 py-0.5 text-[var(--color-warning)] bg-[color-mix(in_srgb,var(--color-warning)_10%,transparent)]">
            <AlertTriangle size={10} aria-hidden="true" /> Overlap
          </span>
        )}
      </div>
      <h4
        className={cn(
          'mt-1 text-sm font-medium leading-snug text-[var(--color-text-primary)]',
          cancelled && 'line-through',
        )}
      >
        {event.title}
      </h4>
      {event.location && (
        <p className="mt-1 flex items-start gap-1 text-xs text-[var(--color-text-secondary)]">
          <MapPin size={12} className="mt-0.5 shrink-0" aria-hidden="true" />
          <span>{event.location}</span>
        </p>
      )}
    </article>
  )
}

function EmptySchedule({ view }: { view: SchoolView }) {
  const message =
    view === 'today'
      ? 'No school events today.'
      : view === 'week'
        ? 'No school events this week.'
        : 'No upcoming school events in the next 90 days.'
  return (
    <div className="rounded-xl border border-dashed border-[var(--color-border)] px-5 py-8 text-center">
      <CalendarDays
        size={22}
        className="mx-auto text-[var(--color-text-tertiary)]"
        aria-hidden="true"
      />
      <p className="mt-2 text-sm font-medium text-[var(--color-text-primary)]">
        {message}
      </p>
      <p className="mt-1 text-xs text-[var(--color-text-tertiary)]">
        Synced calendar data will appear here when available.
      </p>
    </div>
  )
}

function TodayView({ events, now }: { events: ExternalEvent[]; now: Date }) {
  const values = todayEvents(events, now)
  if (values.length === 0) return <EmptySchedule view="today" />
  const conflicts = overlappingEventIds(values)
  return (
    <div className="space-y-2">
      {values.map((event) => (
        <EventCard key={event.id} event={event} conflict={conflicts.has(event.id)} />
      ))}
    </div>
  )
}

function WeekView({ events, now }: { events: ExternalEvent[]; now: Date }) {
  const days = weekDays(now)
  const hasEvents = days.some((day) => events.some((event) => eventOccursOn(event, day)))
  if (!hasEvents) return <EmptySchedule view="week" />
  return (
    <div className="overflow-x-auto pb-2">
      <div className="grid min-w-[980px] grid-cols-7 gap-2">
        {days.map((day) => {
          const values = sortEvents(events.filter((event) => eventOccursOn(event, day)))
          const conflicts = overlappingEventIds(values)
          const today = day.toDateString() === now.toDateString()
          return (
            <section key={day.toISOString()} aria-label={dayFormatter.format(day)}>
              <div
                className={cn(
                  'mb-2 border-b px-1 pb-2 text-xs font-semibold',
                  today
                    ? 'border-[var(--color-accent)] text-[var(--color-accent)]'
                    : 'border-[var(--color-border)] text-[var(--color-text-secondary)]',
                )}
              >
                {dayFormatter.format(day)}
              </div>
              <div className="space-y-2">
                {values.length === 0 ? (
                  <p className="px-1 py-3 text-xs text-[var(--color-text-tertiary)]">
                    Free
                  </p>
                ) : (
                  values.map((event) => (
                    <EventCard
                      key={`${day.toISOString()}-${event.id}`}
                      event={event}
                      conflict={conflicts.has(event.id)}
                    />
                  ))
                )}
              </div>
            </section>
          )
        })}
      </div>
    </div>
  )
}

function UpcomingView({ events, now }: { events: ExternalEvent[]; now: Date }) {
  const values = upcomingEvents(events, now)
  if (values.length === 0) return <EmptySchedule view="upcoming" />
  return (
    <div className="divide-y divide-[var(--color-border)]">
      {values.map((event) => {
        const date =
          event.time_kind === 'all_day' && event.start_date
            ? new Date(`${event.start_date}T00:00:00`)
            : new Date(event.start_at_utc ?? 0)
        return (
          <div
            key={event.id}
            className="grid gap-2 py-3 sm:grid-cols-[8rem_minmax(0,1fr)]"
          >
            <p className="text-xs font-medium text-[var(--color-text-tertiary)]">
              {dayFormatter.format(date)}
            </p>
            <EventCard event={event} />
          </div>
        )
      })}
    </div>
  )
}

export function SchoolSchedule({ spaceId, now }: { spaceId: string; now?: Date }) {
  const [referenceNow] = useState(() => now ?? new Date())
  const [view, setView] = useState<SchoolView>('today')
  const { data, loading, error, isTauri, reload, selectGroup } = useSchoolSchedule(
    spaceId,
    referenceNow,
  )
  const freshness = useMemo(
    () => scheduleFreshness(data?.sources ?? [], referenceNow),
    [data?.sources, referenceNow],
  )
  const latestSync = latestSuccessfulSync(data?.sources ?? [])

  const stateCopy = {
    fresh: 'Schedule is current from local synced data.',
    syncing: 'A calendar sync is in progress. Showing the latest local data.',
    stale: 'Schedule may be out of date. Showing the last local sync.',
    error: 'The latest calendar sync failed. Showing any previously saved events.',
    disconnected:
      'School calendar is disconnected or disabled. Saved events may be out of date.',
    unavailable: data?.sources.length
      ? 'This calendar has not completed a successful sync yet.'
      : 'No School calendar is connected.',
  }[freshness]

  return (
    <section aria-labelledby="school-schedule-heading" className="space-y-4">
      <div className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <p className="aether-eyebrow">School timetable</p>
          <h2
            id="school-schedule-heading"
            className="text-base font-semibold text-[var(--color-text-primary)]"
          >
            Your schedule
          </h2>
        </div>
        {data?.selected_group ? (
          <div
            role="tablist"
            aria-label="School schedule view"
            className="inline-flex rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-0.5"
          >
            {TABS.map((tab) => (
              <button
                key={tab.id}
                type="button"
                role="tab"
                aria-selected={view === tab.id}
                onClick={() => setView(tab.id)}
                className={cn(
                  'focus-ring rounded-md px-3 py-1.5 text-xs font-medium transition-colors',
                  view === tab.id
                    ? 'bg-[var(--color-bg-primary)] text-[var(--color-text-primary)] shadow-sm'
                    : 'text-[var(--color-text-tertiary)] hover:text-[var(--color-text-secondary)]',
                )}
              >
                {tab.label}
              </button>
            ))}
          </div>
        ) : null}
      </div>

      {!isTauri ? (
        <div className="aether-continuity-disclosure">
          Open Aether Desktop to read your private local school timetable.
        </div>
      ) : loading ? (
        <div className="aether-continuity-disclosure" role="status">
          Loading the local school timetable…
        </div>
      ) : error ? (
        <div className="aether-continuity-disclosure" role="alert">
          <p>{error}</p>
          <button
            type="button"
            className="mt-2 text-xs text-[var(--color-accent)]"
            onClick={() => void reload()}
          >
            Try again
          </button>
        </div>
      ) : data ? (
        <>
          <div className="flex flex-wrap items-end justify-between gap-3 rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)] px-3 py-3">
            <div>
              <label
                htmlFor="school-group"
                className="text-xs font-medium text-[var(--color-text-primary)]"
              >
                School group
              </label>
              <p className="mt-0.5 text-xs text-[var(--color-text-tertiary)]">
                Show events assigned to your group, including shared events and exams.
              </p>
            </div>
            <select
              id="school-group"
              aria-label="School group"
              value={data.selected_group ?? ''}
              disabled={data.group_options.length === 0}
              onChange={(event) => {
                if (event.target.value) void selectGroup(event.target.value)
              }}
              className="focus-ring min-w-48 rounded-md border border-[var(--color-border)] bg-[var(--color-bg-primary)] px-2.5 py-1.5 text-sm text-[var(--color-text-primary)]"
            >
              <option value="">Select your group</option>
              {data.group_options.map((group) => (
                <option key={group} value={group}>
                  {group}
                </option>
              ))}
            </select>
          </div>
          <div
            className={cn(
              'flex flex-wrap items-center justify-between gap-2 rounded-lg border px-3 py-2 text-xs',
              freshness === 'fresh'
                ? 'border-[var(--color-border)] text-[var(--color-text-secondary)]'
                : 'border-[var(--color-warning)] text-[var(--color-text-primary)]',
            )}
          >
            <span className="flex items-center gap-1.5">
              <Clock3 size={13} aria-hidden="true" /> {stateCopy}
            </span>
            {latestSync ? (
              <span className="text-[var(--color-text-tertiary)]">
                Last synced {dateTimeFormatter.format(new Date(latestSync))}
              </span>
            ) : (
              <Link to="/settings" className="text-[var(--color-accent)] hover:underline">
                Open Connections
              </Link>
            )}
          </div>
          {!data.selected_group ? (
            <div className="rounded-xl border border-dashed border-[var(--color-border)] px-5 py-8 text-center">
              <CalendarDays
                size={22}
                className="mx-auto text-[var(--color-text-tertiary)]"
                aria-hidden="true"
              />
              <p className="mt-2 text-sm font-medium text-[var(--color-text-primary)]">
                Select your school group
              </p>
              <p className="mx-auto mt-1 max-w-md text-xs text-[var(--color-text-tertiary)]">
                Your timetable stays empty until you choose a group from locally synced
                school events.
              </p>
              {data.group_options.length === 0 ? (
                <Link
                  to="/settings"
                  className="mt-3 inline-block text-xs text-[var(--color-accent)] hover:underline"
                >
                  Sync MyTimetable in Connections
                </Link>
              ) : null}
            </div>
          ) : view === 'today' ? (
            <TodayView events={data.events} now={referenceNow} />
          ) : view === 'week' ? (
            <WeekView events={data.events} now={referenceNow} />
          ) : (
            <UpcomingView events={data.events} now={referenceNow} />
          )}
        </>
      ) : null}
    </section>
  )
}
