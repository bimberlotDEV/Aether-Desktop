import { useMemo } from 'react'
import { useNavigate } from 'react-router-dom'
import {
  Sparkles,
  ArrowRight,
  Layers,
  Star,
  Pin,
  CheckSquare,
  CalendarDays,
  Check,
} from 'lucide-react'
import { useSpaces } from '@/hooks/useSpaces'
import { useGlobalNotes } from '@/hooks/useNotes'
import { FileText } from 'lucide-react'
import { useTaskAttention } from '@/hooks/useTasks'
import type { Task } from '@/lib/db/types'

function getGreeting(): string {
  const hour = new Date().getHours()
  if (hour < 12) return 'Good morning'
  if (hour < 17) return 'Good afternoon'
  return 'Good evening'
}

function formatDate(): string {
  const now = new Date()
  return now.toLocaleDateString('en-US', {
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
  const activeSpaces = spaces.filter((s) => !s.archived_at)
  const favouriteSpaces = activeSpaces.filter((s) => s.favourite)
  const recentSpaces = activeSpaces.filter((s) => !s.favourite).slice(0, 4)

  return (
    <div className="flex-1 flex flex-col items-center justify-center px-8 py-12 min-h-0">
      <div className="w-full max-w-[560px] mx-auto text-center">
        {/* Greeting */}
        <h1
          className="text-3xl font-semibold tracking-tight mb-2"
          style={{ color: 'var(--color-text-primary)' }}
        >
          {greeting}
        </h1>
        <p className="text-sm mb-12" style={{ color: 'var(--color-text-tertiary)' }}>
          {date}
        </p>

        {activeSpaces.length === 0 ? (
          /* Empty state */
          <div
            className="rounded-2xl p-10 text-center"
            style={{
              backgroundColor: 'var(--color-bg-secondary)',
              border: `1px solid var(--color-border)`,
            }}
          >
            <div
              className="w-12 h-12 rounded-xl flex items-center justify-center mx-auto mb-5"
              style={{ backgroundColor: 'var(--color-accent-muted)' }}
            >
              <Sparkles
                size={22}
                strokeWidth={1.75}
                style={{ color: 'var(--color-accent)' }}
              />
            </div>
            <h2
              className="text-lg font-semibold mb-2"
              style={{ color: 'var(--color-text-primary)' }}
            >
              Welcome to Aether
            </h2>
            <p
              className="text-sm max-w-[360px] mx-auto mb-6 leading-relaxed"
              style={{ color: 'var(--color-text-secondary)' }}
            >
              Your personal workspace is ready. Create your first Space to begin
              organizing notes, tasks, files, and AI-assisted thinking.
            </p>
            <button
              onClick={() => navigate('/spaces')}
              className="inline-flex items-center gap-2 px-5 py-2.5 rounded-lg text-sm font-medium transition-colors duration-100"
              style={{
                backgroundColor: 'var(--color-accent)',
                color: 'var(--color-accent-text)',
              }}
            >
              <span>Create your first Space</span>
              <ArrowRight size={14} strokeWidth={2} />
            </button>
          </div>
        ) : (
          /* Spaces overview */
          <div className="space-y-6 text-left">
            {favouriteSpaces.length > 0 && (
              <div>
                <h3
                  className="text-xs font-semibold uppercase tracking-wider mb-2 flex items-center gap-1.5"
                  style={{ color: 'var(--color-text-tertiary)' }}
                >
                  <Star size={11} style={{ color: 'var(--color-warning)' }} /> Favourites
                </h3>
                <div className="space-y-1">
                  {favouriteSpaces.map((s) => (
                    <button
                      key={s.id}
                      onClick={() => navigate(`/spaces/${s.id}`)}
                      className="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-left transition-colors hover:bg-[var(--color-bg-tertiary)]"
                    >
                      <span className="text-lg">{s.icon ? '📦' : '📚'}</span>
                      <span
                        className="text-sm font-medium"
                        style={{ color: 'var(--color-text-primary)' }}
                      >
                        {s.name}
                      </span>
                    </button>
                  ))}
                </div>
              </div>
            )}
            {recentSpaces.length > 0 && (
              <div>
                <h3
                  className="text-xs font-semibold uppercase tracking-wider mb-2"
                  style={{ color: 'var(--color-text-tertiary)' }}
                >
                  Recent Spaces
                </h3>
                <div className="space-y-1">
                  {recentSpaces.map((s) => (
                    <button
                      key={s.id}
                      onClick={() => navigate(`/spaces/${s.id}`)}
                      className="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-left transition-colors hover:bg-[var(--color-bg-tertiary)]"
                    >
                      <span className="text-lg">{s.icon ? '📦' : '📚'}</span>
                      <span
                        className="text-sm font-medium"
                        style={{ color: 'var(--color-text-primary)' }}
                      >
                        {s.name}
                      </span>
                      <span className="flex-1" />
                      <span
                        className="text-xs"
                        style={{ color: 'var(--color-text-tertiary)' }}
                      >
                        {s.template_type === 'school' ? 'School' : 'Blank'}
                      </span>
                    </button>
                  ))}
                </div>
              </div>
            )}
            <PulseTasks />
            <PulseNotes />
            <button
              onClick={() => navigate('/spaces')}
              className="inline-flex items-center gap-1.5 text-sm font-medium transition-colors"
              style={{ color: 'var(--color-accent)' }}
            >
              <Layers size={14} /> View all Spaces
            </button>
          </div>
        )}

        {activeSpaces.length === 0 && (
          <div className="mt-6 text-left">
            <PulseTasks />
          </div>
        )}

        {/* Subtle hint about Command palette */}
        <p className="mt-8 text-xs" style={{ color: 'var(--color-text-tertiary)' }}>
          Press{' '}
          <kbd
            className="text-[10px] font-medium px-1.5 py-0.5 rounded mx-0.5"
            style={{
              backgroundColor: 'var(--color-bg-tertiary)',
              color: 'var(--color-text-secondary)',
              border: `1px solid var(--color-border)`,
            }}
          >
            Ctrl+K
          </kbd>{' '}
          to open the Command palette
        </p>
      </div>
    </div>
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

  if (!loading && tasks.length === 0 && !error) return null

  const group = (label: string, items: Task[], danger = false) =>
    items.length > 0 && (
      <div className="space-y-1">
        <p
          className={
            danger
              ? 'text-xs font-medium text-[var(--color-danger)]'
              : 'text-xs font-medium text-[var(--color-text-tertiary)]'
          }
        >
          {label}
        </p>
        {items.map((task) => (
          <div
            key={task.id}
            className="group flex items-center gap-2 px-3 py-2 rounded-lg hover:bg-[var(--color-bg-tertiary)]"
          >
            <button
              onClick={() => runQuietly(toggleComplete(task))}
              className="w-4 h-4 rounded-full border border-[var(--color-border-hover)] flex items-center justify-center hover:border-[var(--color-accent)] focus-ring"
              aria-label={`Complete ${task.title}`}
            >
              <Check size={10} className="opacity-0 group-hover:opacity-50" />
            </button>
            <button
              onClick={() =>
                navigate(task.space_id ? `/spaces/${task.space_id}/tasks` : '/tasks')
              }
              className="flex-1 min-w-0 text-left focus-ring rounded-sm"
            >
              <span className="block text-sm truncate text-[var(--color-text-primary)]">
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
    )

  return (
    <div>
      <h3 className="text-xs font-semibold uppercase tracking-wider mb-2 flex items-center gap-1.5 text-[var(--color-text-tertiary)]">
        <CheckSquare size={11} /> Tasks
      </h3>
      {loading ? (
        <p className="px-3 py-2 text-xs text-[var(--color-text-tertiary)]">
          Checking what needs attention…
        </p>
      ) : error ? (
        <p className="px-3 py-2 text-xs text-[var(--color-danger)]">{error}</p>
      ) : (
        <div className="space-y-3">
          {group('Overdue', overdue, true)}
          {group('Coming up', upcoming)}
        </div>
      )}
      <button
        onClick={() => navigate('/tasks')}
        className="mt-2 inline-flex items-center gap-1.5 text-xs font-medium text-[var(--color-accent)] focus-ring rounded-sm"
      >
        <CalendarDays size={12} /> Open Task Inbox
      </button>
    </div>
  )
}

function PulseNotes() {
  const navigate = useNavigate()
  const { recent, pinned } = useGlobalNotes()

  if (recent.length === 0 && pinned.length === 0) return null

  return (
    <div>
      <h3
        className="text-xs font-semibold uppercase tracking-wider mb-2 flex items-center gap-1.5"
        style={{ color: 'var(--color-text-tertiary)' }}
      >
        <FileText size={11} /> Notes
      </h3>
      <div className="space-y-1">
        {pinned.slice(0, 2).map((n) => (
          <button
            key={n.id}
            onClick={() => navigate(`/spaces/${n.space_id}/notes`)}
            className="w-full flex items-center gap-2 px-3 py-2 rounded-lg text-left text-sm transition-colors hover:bg-[var(--color-bg-tertiary)]"
          >
            <Pin size={11} style={{ color: 'var(--color-warning)' }} />
            <span className="truncate" style={{ color: 'var(--color-text-primary)' }}>
              {n.title || 'Untitled'}
            </span>
          </button>
        ))}
        {recent.slice(0, 3).map((n) => (
          <button
            key={n.id}
            onClick={() => navigate(`/spaces/${n.space_id}/notes`)}
            className="w-full flex items-center gap-2 px-3 py-2 rounded-lg text-left text-sm transition-colors hover:bg-[var(--color-bg-tertiary)]"
          >
            <FileText size={11} style={{ color: 'var(--color-text-tertiary)' }} />
            <span className="truncate" style={{ color: 'var(--color-text-primary)' }}>
              {n.title || 'Untitled'}
            </span>
          </button>
        ))}
      </div>
    </div>
  )
}
