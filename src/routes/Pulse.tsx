import { useMemo } from 'react'
import { useNavigate } from 'react-router-dom'
import {
  ArrowRight,
  CalendarDays,
  CheckCircle2,
  Clock3,
  Database,
  History,
  Layers,
  RefreshCw,
  Sparkles,
} from 'lucide-react'
import { usePulse } from '@/hooks/usePulse'
import type { PulseSnapshot, PulseTask } from '@/lib/db/types'
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
            <span>{isDesktop ? 'Local signals ready' : 'Desktop data unavailable'}</span>
          </div>
        }
      />

      {!isDesktop ? (
        <Surface className="aether-pulse-disclosure" role="status">
          <Database size={18} aria-hidden="true" />
          <div>
            <strong>Pulse reads your local workspace in the installed app</strong>
            <p>
              This browser preview does not invent Tasks, files, activity, or relevance.
            </p>
          </div>
        </Surface>
      ) : loading ? (
        <Surface className="aether-pulse-status" role="status" aria-live="polite">
          <Clock3 size={18} aria-hidden="true" />
          <p>Finding the few things that matter today…</p>
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
        <div className="space-y-6">
          <div className="aether-pulse-lead-grid">
            <Surface className="aether-pulse-focus">
              <SectionLabel meta="Grounded in local state">
                Suggested next step
              </SectionLabel>
              <p className="aether-pulse-focus-title">{data.suggestedNextStep.title}</p>
              <p>{data.suggestedNextStep.detail}</p>
              <Button
                variant="primary"
                icon={ArrowRight}
                onClick={() => navigate(data.suggestedNextStep.destination)}
              >
                Continue
              </Button>
            </Surface>
            <Surface className="aether-pulse-ask">
              <span className="aether-icon-frame aether-icon-frame--accent">
                <Sparkles size={17} aria-hidden="true" />
              </span>
              <div>
                <SectionLabel>Ask Aether</SectionLabel>
                <strong>Think with the context you choose</strong>
                <p>
                  Open a private conversation, then attach only the Notes, Tasks, files,
                  or Memory you want.
                </p>
              </div>
              <Button variant="secondary" onClick={() => navigate('/ai')}>
                Open AI
              </Button>
            </Surface>
          </div>

          <div className="aether-pulse-grid">
            <TodayPanel data={data} onOpen={navigate} />
            <ContinuePanel spaces={data.continueSpaces} onOpen={navigate} />
          </div>

          <div className="aether-pulse-secondary-grid">
            <Surface className="p-4">
              <SectionLabel
                meta={data.newFiles.length ? `${data.newFiles.length} detected` : 'Quiet'}
              >
                New
              </SectionLabel>
              {data.newFiles.length ? (
                <div className="aether-pulse-list">
                  {data.newFiles.map((file) => (
                    <button
                      key={file.id}
                      className="aether-pulse-row focus-ring"
                      onClick={() => navigate(file.destination)}
                    >
                      <Database size={15} aria-hidden="true" />
                      <span>
                        <strong>{file.title}</strong>
                        <small>{file.detail}</small>
                      </span>
                      <ArrowRight size={13} aria-hidden="true" />
                    </button>
                  ))}
                </div>
              ) : (
                <QuietState text="No newly detected Source files in the last seven days." />
              )}
            </Surface>

            <Surface className="p-4">
              <SectionLabel
                meta={
                  data.recentActivity.length
                    ? `${data.recentActivity.length} meaningful`
                    : 'Quiet'
                }
              >
                Recent
              </SectionLabel>
              {data.recentActivity.length ? (
                <div className="aether-pulse-list">
                  {data.recentActivity.map((item) => (
                    <button
                      key={item.id}
                      className="aether-pulse-row focus-ring"
                      onClick={() => navigate(item.destination)}
                    >
                      <History size={15} aria-hidden="true" />
                      <span>
                        <strong>{item.title}</strong>
                        <small>{item.spaceName ?? 'Workspace'}</small>
                      </span>
                      <ArrowRight size={13} aria-hidden="true" />
                    </button>
                  ))}
                </div>
              ) : (
                <QuietState text="Meaningful local changes will appear here." />
              )}
            </Surface>
          </div>
        </div>
      ) : (
        <EmptyState
          icon={Sparkles}
          title="Pulse is quiet"
          description="No local snapshot is available yet."
        />
      )}
    </Page>
  )
}

function TodayPanel({
  data,
  onOpen,
}: {
  data: PulseSnapshot
  onOpen: (path: string) => void
}) {
  const count = data.overdue.length + data.dueToday.length + data.upcoming.length
  return (
    <Surface className="aether-attention-panel">
      <SectionLabel meta={count ? `${count} open` : 'Clear'}>Today</SectionLabel>
      {count ? (
        <div className="space-y-5">
          <TaskGroup label="Overdue" tasks={data.overdue} danger onOpen={onOpen} />
          <TaskGroup label="Due today" tasks={data.dueToday} onOpen={onOpen} />
          <TaskGroup label="Next seven days" tasks={data.upcoming} onOpen={onOpen} />
        </div>
      ) : (
        <div className="aether-clear-state">
          <span className="aether-icon-frame aether-icon-frame--accent">
            <CheckCircle2 size={17} />
          </span>
          <div>
            <strong>Your dated Tasks are clear</strong>
            <p>No overdue or upcoming Tasks need attention.</p>
          </div>
        </div>
      )}
      <Button variant="quiet" icon={CalendarDays} onClick={() => onOpen('/tasks')}>
        Open Task Inbox
      </Button>
    </Surface>
  )
}

function TaskGroup({
  label,
  tasks,
  danger = false,
  onOpen,
}: {
  label: string
  tasks: PulseTask[]
  danger?: boolean
  onOpen: (path: string) => void
}) {
  if (!tasks.length) return null
  return (
    <div>
      <p
        className={
          danger
            ? 'aether-task-group-label text-[var(--color-danger)]'
            : 'aether-task-group-label'
        }
      >
        {label}
      </p>
      <div className="aether-pulse-list">
        {tasks.map((task) => (
          <button
            key={task.id}
            className="aether-pulse-row focus-ring"
            onClick={() => onOpen(task.destination)}
          >
            <Clock3 size={15} aria-hidden="true" />
            <span>
              <strong>{task.title}</strong>
              <small>
                {task.spaceName ?? 'Task Inbox'} · {task.dueDate}
              </small>
            </span>
            <ArrowRight size={13} aria-hidden="true" />
          </button>
        ))}
      </div>
    </div>
  )
}

function ContinuePanel({
  spaces,
  onOpen,
}: {
  spaces: PulseSnapshot['continueSpaces']
  onOpen: (path: string) => void
}) {
  return (
    <Surface className="p-4">
      <SectionLabel meta={spaces.length ? `${spaces.length} active` : 'Quiet'}>
        Continue
      </SectionLabel>
      {spaces.length ? (
        <div className="aether-pulse-list">
          {spaces.map((space) => (
            <button
              key={space.id}
              className="aether-pulse-row focus-ring"
              onClick={() => onOpen(space.destination)}
            >
              <Layers size={15} aria-hidden="true" />
              <span>
                <strong>{space.name}</strong>
                <small>{space.reason}</small>
              </span>
              <ArrowRight size={13} aria-hidden="true" />
            </button>
          ))}
        </div>
      ) : (
        <QuietState text="Work in a Space and Pulse will keep your place." />
      )}
      <Button variant="quiet" icon={Layers} onClick={() => onOpen('/spaces')}>
        View all Spaces
      </Button>
    </Surface>
  )
}

function QuietState({ text }: { text: string }) {
  return <p className="aether-pulse-quiet">{text}</p>
}
