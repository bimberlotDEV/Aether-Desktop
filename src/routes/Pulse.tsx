import { useMemo } from 'react'
import { useNavigate } from 'react-router-dom'
import {
  ArrowRight,
  CalendarDays,
  Check,
  CheckSquare,
  FileText,
  Layers,
  Pin,
  Sparkles,
  Star,
} from 'lucide-react'
import { useSpaces } from '@/hooks/useSpaces'
import { useGlobalNotes } from '@/hooks/useNotes'
import { useTaskAttention } from '@/hooks/useTasks'
import type { Task } from '@/lib/db/types'
import { iconToEmoji } from '@/lib/iconToEmoji'
import {
  Button,
  EmptyState,
  Page,
  PageHeader,
  SectionLabel,
  StatusDot,
  Surface,
} from '@/components/ui/AetherUI'

function getGreeting(): string {
  const hour = new Date().getHours()
  if (hour < 12) return 'Good morning'
  if (hour < 17) return 'Good afternoon'
  return 'Good evening'
}

function formatDate(): string {
  return new Date().toLocaleDateString('en-US', {
    weekday: 'long',
    month: 'long',
    day: 'numeric',
  })
}

export function Pulse() {
  const navigate = useNavigate()
  const { spaces } = useSpaces()
  const greeting = useMemo(() => getGreeting(), [])
  const date = useMemo(() => formatDate(), [])
  const activeSpaces = spaces.filter((space) => !space.archived_at)
  const favouriteSpaces = activeSpaces.filter((space) => space.favourite)
  const recentSpaces = activeSpaces.filter((space) => !space.favourite).slice(0, 4)

  return (
    <Page width="wide" className="aether-pulse">
      <PageHeader
        eyebrow="Today"
        title={greeting}
        description={date}
        actions={
          <div className="aether-presence">
            <StatusDot />
            <span>Workspace ready</span>
          </div>
        }
      />

      {activeSpaces.length === 0 ? (
        <div className="space-y-4">
          <div className="aether-pulse-empty-grid">
            <EmptyState
              icon={Sparkles}
              eyebrow="A quiet place to begin"
              title="Shape your first Space"
              description="Bring notes, tasks, files, and AI-assisted thinking together around something that matters to you."
              action={{ label: 'Create a Space', onClick: () => navigate('/spaces') }}
            />
            <Surface className="aether-launch-panel">
              <SectionLabel meta="Ctrl K">Start anywhere</SectionLabel>
              <button
                onClick={() => navigate('/spaces')}
                className="aether-launch-row focus-ring"
              >
                <span className="aether-icon-frame">
                  <Layers size={16} />
                </span>
                <span>
                  <strong>Organize a project</strong>
                  <small>Create a focused Space</small>
                </span>
                <ArrowRight size={14} />
              </button>
              <button
                onClick={() => navigate('/tasks')}
                className="aether-launch-row focus-ring"
              >
                <span className="aether-icon-frame">
                  <CheckSquare size={16} />
                </span>
                <span>
                  <strong>Capture something</strong>
                  <small>Add it to your Task Inbox</small>
                </span>
                <ArrowRight size={14} />
              </button>
              <button
                onClick={() => navigate('/ai')}
                className="aether-launch-row focus-ring"
              >
                <span className="aether-icon-frame">
                  <Sparkles size={16} />
                </span>
                <span>
                  <strong>Think with AI</strong>
                  <small>Open a private conversation</small>
                </span>
                <ArrowRight size={14} />
              </button>
            </Surface>
          </div>
          <PulseTasks />
        </div>
      ) : (
        <div className="aether-pulse-grid">
          <div className="space-y-6">
            <Surface className="p-4">
              <SectionLabel meta={`${activeSpaces.length} active`}>Spaces</SectionLabel>
              <div className="aether-space-launcher-grid">
                {[...favouriteSpaces, ...recentSpaces].slice(0, 6).map((space) => (
                  <button
                    key={space.id}
                    onClick={() => navigate(`/spaces/${space.id}`)}
                    className="aether-space-launcher focus-ring"
                  >
                    <span
                      className="aether-space-emoji"
                      style={{
                        backgroundColor: space.accent
                          ? `${space.accent}18`
                          : 'var(--color-accent-muted)',
                      }}
                    >
                      {space.icon ? iconToEmoji(space.icon) : '📚'}
                    </span>
                    <span className="min-w-0 flex-1">
                      <strong>{space.name}</strong>
                      <small>{space.favourite ? 'Favourite Space' : 'Open Space'}</small>
                    </span>
                    {space.favourite ? <Star size={13} /> : <ArrowRight size={13} />}
                  </button>
                ))}
              </div>
              <Button variant="quiet" icon={Layers} onClick={() => navigate('/spaces')}>
                View all Spaces
              </Button>
            </Surface>
            <PulseNotes />
          </div>
          <PulseTasks />
        </div>
      )}
    </Page>
  )
}

function localToday(): string {
  const now = new Date()
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`
}

function dueLabel(task: Task): string {
  if (!task.due_date) return ''
  const today = localToday()
  if (task.due_date === today) return 'Today'
  const [year, month, day] = task.due_date.split('-').map(Number)
  return new Date(year, month - 1, day).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
  })
}

function runQuietly(operation: Promise<unknown>) {
  void operation.catch(() => undefined)
}

function PulseTasks() {
  const navigate = useNavigate()
  const { tasks, loading, error, toggleComplete } = useTaskAttention()
  const today = localToday()
  const overdue = tasks.filter((task) => !!task.due_date && task.due_date < today)
  const upcoming = tasks.filter((task) => !!task.due_date && task.due_date >= today)

  return (
    <Surface className="aether-attention-panel">
      <SectionLabel meta={tasks.length ? `${tasks.length} due` : 'Clear'}>
        Your attention
      </SectionLabel>
      {loading ? (
        <p className="aether-inline-message">Checking what needs attention…</p>
      ) : error ? (
        <p className="aether-inline-message text-[var(--color-danger)]">{error}</p>
      ) : tasks.length === 0 ? (
        <div className="aether-clear-state">
          <span className="aether-icon-frame aether-icon-frame--accent">
            <Check size={17} />
          </span>
          <div>
            <strong>Your day is clear</strong>
            <p>No overdue or upcoming Tasks need attention.</p>
          </div>
        </div>
      ) : (
        <div className="space-y-5">
          <TaskGroup
            label="Overdue"
            tasks={overdue}
            danger
            onComplete={(task) => runQuietly(toggleComplete(task))}
            onOpen={(task) =>
              navigate(task.space_id ? `/spaces/${task.space_id}/tasks` : '/tasks')
            }
          />
          <TaskGroup
            label="Coming up"
            tasks={upcoming}
            onComplete={(task) => runQuietly(toggleComplete(task))}
            onOpen={(task) =>
              navigate(task.space_id ? `/spaces/${task.space_id}/tasks` : '/tasks')
            }
          />
        </div>
      )}
      <Button variant="quiet" icon={CalendarDays} onClick={() => navigate('/tasks')}>
        Open Task Inbox
      </Button>
    </Surface>
  )
}

function TaskGroup({
  label,
  tasks,
  danger = false,
  onComplete,
  onOpen,
}: {
  label: string
  tasks: Task[]
  danger?: boolean
  onComplete: (task: Task) => void
  onOpen: (task: Task) => void
}) {
  if (tasks.length === 0) return null
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
      <div className="space-y-1">
        {tasks.map((task) => (
          <div key={task.id} className="aether-attention-row">
            <button
              onClick={() => onComplete(task)}
              className="aether-task-check focus-ring"
              aria-label={`Complete ${task.title}`}
            >
              <Check size={10} />
            </button>
            <button
              onClick={() => onOpen(task)}
              className="min-w-0 flex-1 text-left focus-ring"
            >
              <span className="block truncate text-sm text-[var(--color-text-primary)]">
                {task.title}
              </span>
            </button>
            <span
              className={
                danger
                  ? 'text-xs text-[var(--color-danger)]'
                  : 'text-xs text-[var(--color-text-tertiary)]'
              }
            >
              {dueLabel(task)}
            </span>
          </div>
        ))}
      </div>
    </div>
  )
}

function PulseNotes() {
  const navigate = useNavigate()
  const { recent, pinned } = useGlobalNotes()
  const notes = [...pinned.slice(0, 2), ...recent.slice(0, 3)]
  if (notes.length === 0) return null

  return (
    <Surface className="p-4">
      <SectionLabel meta={`${notes.length} recent`}>Notes</SectionLabel>
      <div className="space-y-1">
        {notes.map((note, index) => (
          <button
            key={`${note.id}-${index}`}
            onClick={() => navigate(`/spaces/${note.space_id}/notes`)}
            className="aether-note-row focus-ring"
          >
            {note.pinned ? <Pin size={13} /> : <FileText size={13} />}
            <span>{note.title || 'Untitled'}</span>
            <ArrowRight size={12} />
          </button>
        ))}
      </div>
    </Surface>
  )
}
