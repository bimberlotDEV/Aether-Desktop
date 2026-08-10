import { useMemo, useState } from 'react'
import {
  Archive,
  CalendarDays,
  Check,
  ChevronDown,
  ChevronRight,
  ListFilter,
  Plus,
  RotateCcw,
  Search,
  Trash2,
} from 'lucide-react'
import type { Space, Task, TaskInput, TaskPriority, TaskStatus } from '@/lib/db/types'
import { useTasks } from '@/hooks/useTasks'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { TaskEditor } from '@/components/tasks/TaskEditor'
import { cn } from '@/lib/utils'

interface TaskViewProps {
  spaceId?: string
  spaces: Space[]
  title?: string
  description?: string
}

const STATUS_LABELS: Record<TaskStatus, string> = {
  inbox: 'Inbox',
  planned: 'Planned',
  in_progress: 'In progress',
  done: 'Done',
}

const PRIORITY_LABELS: Record<TaskPriority, string> = {
  none: 'All priorities',
  low: 'Low',
  medium: 'Medium',
  high: 'High',
}

function dateLabel(value: string): string {
  const [year, month, day] = value.split('-').map(Number)
  return new Date(year, month - 1, day).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
  })
}

function localToday(): string {
  const now = new Date()
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`
}

function runQuietly(operation: Promise<unknown>) {
  void operation.catch(() => undefined)
}

function TaskRow({
  task,
  depth,
  childCount,
  onToggle,
  onEdit,
  onAddSubtask,
  onArchive,
}: {
  task: Task
  depth: number
  childCount: number
  onToggle: () => void
  onEdit: () => void
  onAddSubtask: () => void
  onArchive: () => void
}) {
  const overdue =
    !!task.due_date && task.due_date < localToday() && task.status !== 'done'
  return (
    <div
      className="group flex items-start gap-3 py-3 pr-2 border-b border-[var(--color-border)]"
      style={{ paddingLeft: `${Math.min(depth, 3) * 24 + 8}px` }}
    >
      <button
        onClick={onToggle}
        className={cn(
          'mt-0.5 w-5 h-5 shrink-0 rounded-full border flex items-center justify-center transition-colors focus-ring',
          task.status === 'done'
            ? 'bg-[var(--color-success)] border-[var(--color-success)] text-white'
            : 'border-[var(--color-border-hover)] hover:border-[var(--color-accent)]',
        )}
        aria-label={
          task.status === 'done' ? `Reopen ${task.title}` : `Complete ${task.title}`
        }
      >
        {task.status === 'done' && <Check size={12} strokeWidth={2.5} />}
      </button>

      <button onClick={onEdit} className="min-w-0 flex-1 text-left focus-ring rounded-sm">
        <span
          className={cn(
            'block text-sm font-medium truncate',
            task.status === 'done' && 'line-through text-[var(--color-text-tertiary)]',
          )}
        >
          {task.title}
        </span>
        <span className="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1 text-xs text-[var(--color-text-tertiary)]">
          <span>{STATUS_LABELS[task.status]}</span>
          {task.priority !== 'none' && (
            <span
              className={cn(
                task.priority === 'high' && 'text-[var(--color-danger)]',
                task.priority === 'medium' && 'text-[var(--color-warning)]',
              )}
            >
              {PRIORITY_LABELS[task.priority]}
            </span>
          )}
          {task.due_date && (
            <span
              className={cn(
                'inline-flex items-center gap-1',
                overdue && 'text-[var(--color-danger)]',
              )}
            >
              <CalendarDays size={11} /> {dateLabel(task.due_date)}
            </span>
          )}
          {childCount > 0 && (
            <span>
              {childCount} subtask{childCount === 1 ? '' : 's'}
            </span>
          )}
          {task.tags.slice(0, 3).map((tag) => (
            <span key={tag}>#{tag}</span>
          ))}
        </span>
      </button>

      <div className="flex items-center opacity-0 group-hover:opacity-100 focus-within:opacity-100 transition-opacity">
        <button
          onClick={onAddSubtask}
          className="p-1.5 rounded-md hover:bg-[var(--color-bg-tertiary)] focus-ring"
          aria-label={`Add subtask to ${task.title}`}
          title="Add subtask"
        >
          <Plus size={14} />
        </button>
        <button
          onClick={onArchive}
          className="p-1.5 rounded-md hover:bg-[var(--color-bg-tertiary)] focus-ring"
          aria-label={`Archive ${task.title}`}
          title="Archive"
        >
          <Archive size={14} />
        </button>
      </div>
    </div>
  )
}

export function TaskView({
  spaceId,
  spaces,
  title = spaceId ? 'Tasks' : 'Inbox',
  description = spaceId
    ? 'Keep the work in this Space clear and lightweight.'
    : 'Capture Tasks first, then move them into a Space when they are ready.',
}: TaskViewProps) {
  const [search, setSearch] = useState('')
  const [status, setStatus] = useState<TaskStatus | 'all'>('all')
  const [priority, setPriority] = useState<TaskPriority | 'all'>('all')
  const [quickTitle, setQuickTitle] = useState('')
  const [showArchived, setShowArchived] = useState(false)
  const [editor, setEditor] = useState<{ task?: Task; parentId?: string } | null>(null)
  const [deleteTarget, setDeleteTarget] = useState<Task | null>(null)

  const filter = useMemo(
    () => ({
      ...(spaceId ? { spaceId } : {}),
      ...(!spaceId ? { unassignedOnly: true } : {}),
      ...(search.trim() ? { search: search.trim() } : {}),
      ...(status !== 'all' ? { status } : {}),
      ...(priority !== 'all' ? { priority } : {}),
    }),
    [priority, search, spaceId, status],
  )
  const active = useTasks(filter)
  const archivedSource = useTasks({
    ...(spaceId ? { spaceId } : { unassignedOnly: true }),
    includeArchived: true,
  })
  const archived = archivedSource.tasks.filter((task) => !!task.archived_at)

  const visibleTasks = useMemo(() => {
    const byParent = new Map<string | null, Task[]>()
    for (const task of active.tasks) {
      const parent = active.tasks.some(
        (candidate) => candidate.id === task.parent_task_id,
      )
        ? task.parent_task_id
        : null
      byParent.set(parent, [...(byParent.get(parent) ?? []), task])
    }
    const flattened: { task: Task; depth: number }[] = []
    const append = (parentId: string | null, depth: number) => {
      for (const task of byParent.get(parentId) ?? []) {
        flattened.push({ task, depth })
        append(task.id, depth + 1)
      }
    }
    append(null, 0)
    return flattened
  }, [active.tasks])

  async function quickCreate(event: React.FormEvent) {
    event.preventDefault()
    const titleValue = quickTitle.trim()
    if (!titleValue) return
    try {
      await active.create({
        spaceId: spaceId ?? null,
        parentTaskId: null,
        title: titleValue,
        description: '',
        status: 'inbox',
        priority: 'none',
        dueDate: null,
        tags: [],
      })
      setQuickTitle('')
    } catch {
      // The hook exposes the actionable error next to the form.
    }
  }

  return (
    <div className="max-w-[820px] mx-auto px-8 py-7">
      <div className="flex items-start gap-4 mb-6">
        <div className="flex-1">
          <h1 className="text-xl font-semibold text-[var(--color-text-primary)]">
            {title}
          </h1>
          <p className="text-sm mt-1 text-[var(--color-text-secondary)]">{description}</p>
        </div>
        <button
          onClick={() => setEditor({})}
          className="inline-flex items-center gap-2 px-3 py-2 rounded-md bg-[var(--color-accent)] text-[var(--color-accent-text)] text-sm font-medium focus-ring"
        >
          <Plus size={15} /> New Task
        </button>
      </div>

      <form
        onSubmit={quickCreate}
        className="flex items-center gap-2 p-2 mb-4 rounded-lg border bg-[var(--color-bg-secondary)] border-[var(--color-border)]"
      >
        <Plus size={16} className="ml-1 text-[var(--color-text-tertiary)]" />
        <input
          value={quickTitle}
          onChange={(event) => setQuickTitle(event.target.value)}
          placeholder={
            spaceId ? 'Add a Task to this Space…' : 'Capture something in Inbox…'
          }
          aria-label="Quick Task title"
          className="flex-1 bg-transparent text-sm outline-none text-[var(--color-text-primary)] placeholder:text-[var(--color-text-tertiary)]"
          maxLength={200}
        />
        <button
          disabled={!quickTitle.trim()}
          className="px-3 py-1.5 rounded-md text-xs font-medium bg-[var(--color-bg-elevated)] border border-[var(--color-border)] disabled:opacity-40 focus-ring"
        >
          Add
        </button>
      </form>

      <div className="flex flex-wrap items-center gap-2 mb-3">
        <label className="flex-1 min-w-52 flex items-center gap-2 px-3 h-9 rounded-md border border-[var(--color-border)] bg-[var(--color-bg)]">
          <Search size={14} className="text-[var(--color-text-tertiary)]" />
          <span className="sr-only">Search Tasks</span>
          <input
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder="Search Tasks"
            className="w-full bg-transparent outline-none text-sm"
          />
        </label>
        <ListFilter size={14} className="text-[var(--color-text-tertiary)]" />
        <select
          aria-label="Filter by status"
          value={status}
          onChange={(event) => setStatus(event.target.value as TaskStatus | 'all')}
          className="h-9 px-2 rounded-md border border-[var(--color-border)] bg-[var(--color-bg)] text-sm focus-ring"
        >
          <option value="all">All statuses</option>
          {Object.entries(STATUS_LABELS).map(([value, label]) => (
            <option key={value} value={value}>
              {label}
            </option>
          ))}
        </select>
        <select
          aria-label="Filter by priority"
          value={priority}
          onChange={(event) => setPriority(event.target.value as TaskPriority | 'all')}
          className="h-9 px-2 rounded-md border border-[var(--color-border)] bg-[var(--color-bg)] text-sm focus-ring"
        >
          <option value="all">All priorities</option>
          <option value="high">High</option>
          <option value="medium">Medium</option>
          <option value="low">Low</option>
          <option value="none">No priority</option>
        </select>
      </div>

      {(active.error || archivedSource.error) && (
        <p role="alert" className="mb-3 text-sm text-[var(--color-danger)]">
          {active.error || archivedSource.error}
        </p>
      )}

      <section
        aria-label="Active Tasks"
        className="border-t border-[var(--color-border)]"
      >
        {active.loading ? (
          <p className="py-10 text-center text-sm text-[var(--color-text-tertiary)]">
            Loading Tasks…
          </p>
        ) : visibleTasks.length === 0 ? (
          <div className="py-12 text-center">
            <p className="text-sm font-medium text-[var(--color-text-secondary)]">
              No Tasks here
            </p>
            <p className="text-xs mt-1 text-[var(--color-text-tertiary)]">
              {search || status !== 'all' || priority !== 'all'
                ? 'Try clearing a filter.'
                : 'Capture one above when something needs your attention.'}
            </p>
          </div>
        ) : (
          visibleTasks.map(({ task, depth }) => (
            <TaskRow
              key={task.id}
              task={task}
              depth={depth}
              childCount={
                active.tasks.filter((candidate) => candidate.parent_task_id === task.id)
                  .length
              }
              onToggle={() => runQuietly(active.toggleComplete(task))}
              onEdit={() => setEditor({ task })}
              onAddSubtask={() => setEditor({ parentId: task.id })}
              onArchive={() => runQuietly(active.archive(task.id))}
            />
          ))
        )}
      </section>

      <section className="mt-6">
        <button
          onClick={() => setShowArchived((value) => !value)}
          className="flex items-center gap-2 text-sm font-medium text-[var(--color-text-secondary)] focus-ring rounded-sm"
        >
          {showArchived ? <ChevronDown size={15} /> : <ChevronRight size={15} />}
          Archived ({archived.length})
        </button>
        {showArchived && (
          <div className="mt-2 border-t border-[var(--color-border)]">
            {archived.length === 0 ? (
              <p className="py-5 text-xs text-[var(--color-text-tertiary)]">
                No archived Tasks.
              </p>
            ) : (
              archived.map((task) => (
                <div
                  key={task.id}
                  className="flex items-center gap-3 py-3 px-2 border-b border-[var(--color-border)]"
                >
                  <Archive size={14} className="text-[var(--color-text-tertiary)]" />
                  <span className="flex-1 truncate text-sm text-[var(--color-text-secondary)]">
                    {task.title}
                  </span>
                  <button
                    onClick={() => runQuietly(archivedSource.restore(task.id))}
                    className="p-1.5 rounded-md hover:bg-[var(--color-bg-tertiary)] focus-ring"
                    aria-label={`Restore ${task.title}`}
                    title="Restore"
                  >
                    <RotateCcw size={14} />
                  </button>
                  <button
                    onClick={() => setDeleteTarget(task)}
                    className="p-1.5 rounded-md text-[var(--color-danger)] hover:bg-[var(--color-bg-tertiary)] focus-ring"
                    aria-label={`Delete ${task.title} permanently`}
                    title="Delete permanently"
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              ))
            )}
          </div>
        )}
      </section>

      {editor && (
        <TaskEditor
          task={editor.task}
          defaultSpaceId={spaceId ?? null}
          defaultParentId={editor.parentId}
          spaces={spaces}
          possibleParents={archivedSource.tasks.filter((task) => !task.archived_at)}
          onSave={(input: TaskInput) =>
            editor.task ? active.update(editor.task, input) : active.create(input)
          }
          onClose={() => setEditor(null)}
        />
      )}

      {deleteTarget && (
        <ConfirmDialog
          title="Delete Task permanently"
          message={`Permanently delete "${deleteTarget.title}" and its subtasks? This cannot be undone.`}
          confirmLabel="Delete permanently"
          danger
          onConfirm={async () => {
            await archivedSource.remove(deleteTarget.id)
            setDeleteTarget(null)
          }}
          onCancel={() => setDeleteTarget(null)}
        />
      )}
    </div>
  )
}
