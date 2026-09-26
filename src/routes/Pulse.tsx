import { useMemo } from 'react'
import { useNavigate } from 'react-router-dom'
import {
  AlertTriangle,
  ArrowRight,
  CalendarDays,
  CheckCircle2,
  Clock3,
  Database,
  Layers,
  ListTodo,
  RefreshCw,
  ShieldCheck,
} from 'lucide-react'
import { usePulse } from '@/hooks/usePulse'
import type {
  PulseContinuityItem,
  PulseEvent,
  PulseSnapshot,
  PulseTask,
} from '@/lib/db/types'
import {
  Button,
  EmptyState,
  Page,
  PageHeader,
  SectionLabel,
  StatusDot,
  Surface,
} from '@/components/ui/AetherUI'

function greeting() {
  const hour = new Date().getHours()
  if (hour < 12) return 'Good morning'
  if (hour < 17) return 'Good afternoon'
  return 'Good evening'
}

function displayDate() {
  return new Date().toLocaleDateString('en-US', {
    weekday: 'long',
    month: 'long',
    day: 'numeric',
  })
}

function time(value: string | null) {
  if (!value) return ''
  return new Date(value).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

function eventTime(event: PulseEvent) {
  if (event.timeKind === 'all_day') return 'All day'
  return `${time(event.startAt)}–${time(event.endAt)}`
}

function eventDay(event: PulseEvent) {
  const value = event.startDate
    ? new Date(`${event.startDate}T12:00:00`)
    : event.startAt
      ? new Date(event.startAt)
      : null
  return value?.toLocaleDateString([], { weekday: 'short', day: 'numeric' }) ?? ''
}

export function Pulse() {
  const navigate = useNavigate()
  const { data, loading, error, isDesktop, reload } = usePulse()
  const title = useMemo(greeting, [])
  const date = useMemo(displayDate, [])

  return (
    <Page width="wide" className="aether-pulse">
      <PageHeader
        eyebrow="Pulse"
        title={title}
        description={date}
        actions={
          <div className="aether-presence">
            <StatusDot />
            <span>{isDesktop ? 'Local snapshot' : 'Desktop data unavailable'}</span>
          </div>
        }
      />

      {!isDesktop ? (
        <Surface className="aether-pulse-disclosure" role="status">
          <Database size={18} aria-hidden="true" />
          <div>
            <strong>Pulse reads your local workspace in the installed app</strong>
            <p>This browser preview does not invent schedule, Tasks, or source state.</p>
          </div>
        </Surface>
      ) : loading ? (
        <Surface className="aether-pulse-status" role="status" aria-live="polite">
          <Clock3 size={18} aria-hidden="true" />
          <p>Building today’s local snapshot…</p>
        </Surface>
      ) : error ? (
        <Surface className="aether-pulse-status" role="alert">
          <div>
            <strong>Pulse could not read your workspace</strong>
            <p>{error}</p>
          </div>
          <Button variant="secondary" icon={RefreshCw} onClick={() => void reload()}>
            Try again
          </Button>
        </Surface>
      ) : data ? (
        <PulseContent data={data} onOpen={navigate} />
      ) : (
        <EmptyState
          icon={CalendarDays}
          title="Pulse is quiet"
          description="No local snapshot is available yet."
        />
      )}
    </Page>
  )
}

function PulseContent({ data, onOpen }: { data: PulseSnapshot; onOpen: (path: string) => void }) {
  const hasAttention =
    data.now.length ||
    data.next ||
    data.today.length ||
    data.tasks.length ||
    data.continuity.length

  return (
    <div className="space-y-5">
      {data.issues.length ? (
        <Surface className="aether-pulse-status" role="status">
          <AlertTriangle size={17} aria-hidden="true" />
          <p>{data.issues.map((issue) => issue.message).join(' · ')}</p>
        </Surface>
      ) : null}

      <div className="aether-pulse-lead-grid">
        <NowPanel events={data.now} />
        <NextPanel event={data.next} />
      </div>

      {hasAttention ? (
        <div className="aether-pulse-grid">
          <SchedulePanel events={data.today} />
          <TasksPanel tasks={data.tasks} onOpen={onOpen} />
        </div>
      ) : (
        <Surface className="aether-clear-state">
          <span className="aether-icon-frame aether-icon-frame--accent">
            <CheckCircle2 size={17} aria-hidden="true" />
          </span>
          <div>
            <strong>Nothing needs your attention right now</strong>
            <p>No current schedule or dated open Tasks are in the next seven days.</p>
          </div>
        </Surface>
      )}

      {data.conflicts.length ? <ConflictsPanel data={data} /> : null}

      <div className="aether-pulse-secondary-grid">
        <UpcomingPanel events={data.upcoming} />
        <ContinuityPanel items={data.continuity} onOpen={onOpen} />
      </div>

      {data.trust.length ? <TrustPanel data={data} /> : null}
    </div>
  )
}

function NowPanel({ events }: { events: PulseEvent[] }) {
  return (
    <Surface className="aether-pulse-focus">
      <SectionLabel meta={events.length > 1 ? `${events.length} overlapping` : 'Current'}>
        Now
      </SectionLabel>
      {events.length ? (
        <div className="aether-pulse-list">
          {events.map((event) => (
            <EventRow key={event.id} event={event} />
          ))}
        </div>
      ) : (
        <QuietState text="No timed event is happening now." />
      )}
    </Surface>
  )
}

function NextPanel({ event }: { event: PulseEvent | null }) {
  return (
    <Surface className="p-4">
      <SectionLabel meta="Next timed event">Next</SectionLabel>
      {event ? <EventRow event={event} /> : <QuietState text="No timed event is coming up." />}
    </Surface>
  )
}

function SchedulePanel({ events }: { events: PulseEvent[] }) {
  return (
    <Surface className="p-4">
      <SectionLabel meta={events.length ? `${events.length} scheduled` : 'Clear'}>
        Today
      </SectionLabel>
      {events.length ? (
        <div className="aether-pulse-list">
          {events.map((event) => (
            <EventRow key={event.id} event={event} />
          ))}
        </div>
      ) : (
        <QuietState text="Your authorized calendars have no events today." />
      )}
    </Surface>
  )
}

function UpcomingPanel({ events }: { events: PulseEvent[] }) {
  return (
    <Surface className="p-4">
      <SectionLabel meta={events.length ? `${events.length} in seven days` : 'Clear'}>
        Upcoming
      </SectionLabel>
      {events.length ? (
        <div className="aether-pulse-list">
          {events.map((event) => (
            <EventRow key={event.id} event={event} showDate />
          ))}
        </div>
      ) : (
        <QuietState text="No events are scheduled in the next seven days." />
      )}
    </Surface>
  )
}

function EventRow({ event, showDate = false }: { event: PulseEvent; showDate?: boolean }) {
  const prefix = showDate ? `${eventDay(event)} · ` : ''
  return (
    <div className={`aether-pulse-row${event.cancelled ? ' opacity-60' : ''}`}>
      <CalendarDays size={15} aria-hidden="true" />
      <span>
        <strong>{event.cancelled ? `Cancelled · ${event.title}` : event.title}</strong>
        <small>
          {prefix}
          {eventTime(event)} · {event.sourceLabel}
          {event.location ? ` · ${event.location}` : ''}
        </small>
      </span>
    </div>
  )
}

function TasksPanel({ tasks, onOpen }: { tasks: PulseTask[]; onOpen: (path: string) => void }) {
  return (
    <Surface className="p-4">
      <SectionLabel meta={tasks.length ? `${tasks.length} open` : 'Clear'}>Tasks</SectionLabel>
      {tasks.length ? (
        <div className="aether-pulse-list">
          {tasks.map((task) => (
            <button
              key={task.id}
              className="aether-pulse-row focus-ring"
              onClick={() => onOpen(task.destination)}
            >
              <ListTodo size={15} aria-hidden="true" />
              <span>
                <strong>{task.title}</strong>
                <small>
                  {task.category === 'overdue'
                    ? 'Overdue'
                    : task.category === 'today'
                      ? 'Due today'
                      : `Due ${task.dueDate}`}
                  {task.spaceName ? ` · ${task.spaceName}` : ''}
                </small>
              </span>
              <ArrowRight size={13} aria-hidden="true" />
            </button>
          ))}
        </div>
      ) : (
        <QuietState text="No overdue or near-term dated Tasks need attention." />
      )}
      <Button variant="quiet" icon={ListTodo} onClick={() => onOpen('/tasks')}>
        Open Tasks
      </Button>
    </Surface>
  )
}

function ConflictsPanel({ data }: { data: PulseSnapshot }) {
  return (
    <Surface className="p-4">
      <SectionLabel meta={`${data.conflicts.length} detected`}>Conflicts</SectionLabel>
      <div className="aether-pulse-list">
        {data.conflicts.map((conflict) => (
          <div key={conflict.id} className="aether-pulse-row">
            <AlertTriangle size={15} aria-hidden="true" />
            <span>
              <strong>
                {conflict.first.title} overlaps {conflict.second.title}
              </strong>
              <small>
                {eventTime(conflict.first)} · {conflict.first.sourceLabel}
              </small>
            </span>
          </div>
        ))}
      </div>
    </Surface>
  )
}

function ContinuityPanel({
  items,
  onOpen,
}: {
  items: PulseContinuityItem[]
  onOpen: (path: string) => void
}) {
  return (
    <Surface className="p-4">
      <SectionLabel meta={items.length ? `${items.length} recent` : 'Quiet'}>Continue</SectionLabel>
      {items.length ? (
        <div className="aether-pulse-list">
          {items.map((item) => (
            <button
              key={item.id}
              className="aether-pulse-row focus-ring"
              onClick={() => onOpen(item.destination)}
            >
              <Layers size={15} aria-hidden="true" />
              <span>
                <strong>{item.name}</strong>
                <small>{item.reason}</small>
              </span>
              <ArrowRight size={13} aria-hidden="true" />
            </button>
          ))}
        </div>
      ) : (
        <QuietState text="Work in a Space and Pulse will keep your place." />
      )}
    </Surface>
  )
}

function TrustPanel({ data }: { data: PulseSnapshot }) {
  const labels: Record<PulseSnapshot['trust'][number]['state'], string> = {
    fresh: 'Fresh',
    stale: 'Stale',
    syncing: 'Syncing',
    disabled: 'Disabled',
    disconnected: 'Disconnected',
    degraded: 'Degraded',
    never_synced: 'Never synced',
  }
  return (
    <Surface className="p-4">
      <SectionLabel meta="Native source state">Trust</SectionLabel>
      <div className="aether-pulse-trust-grid">
        {data.trust.map((item) => (
          <div key={item.sourceId} className="aether-pulse-trust-item">
            <ShieldCheck size={15} aria-hidden="true" />
            <span>
              <strong>{item.sourceLabel}</strong>
              <small>
                {labels[item.state]}
                {item.showingCachedData ? ' · showing cached data' : ''}
              </small>
            </span>
          </div>
        ))}
      </div>
    </Surface>
  )
}

function QuietState({ text }: { text: string }) {
  return <p className="aether-pulse-quiet">{text}</p>
}
