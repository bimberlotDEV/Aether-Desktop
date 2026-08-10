import { useEffect, useMemo, useState } from 'react'
import { X } from 'lucide-react'
import type { Space, Task, TaskInput, TaskPriority, TaskStatus } from '@/lib/db/types'

const STATUS_OPTIONS: { value: TaskStatus; label: string }[] = [
  { value: 'inbox', label: 'Inbox' },
  { value: 'planned', label: 'Planned' },
  { value: 'in_progress', label: 'In progress' },
  { value: 'done', label: 'Done' },
]

const PRIORITY_OPTIONS: { value: TaskPriority; label: string }[] = [
  { value: 'none', label: 'No priority' },
  { value: 'low', label: 'Low' },
  { value: 'medium', label: 'Medium' },
  { value: 'high', label: 'High' },
]

interface TaskEditorProps {
  task?: Task | null
  defaultSpaceId: string | null
  defaultParentId?: string | null
  spaces: Space[]
  possibleParents: Task[]
  onSave: (input: TaskInput) => Promise<unknown>
  onClose: () => void
}

function initialInput(
  task: Task | null | undefined,
  defaultSpaceId: string | null,
  defaultParentId: string | null,
): TaskInput {
  return task
    ? {
        spaceId: task.space_id,
        parentTaskId: task.parent_task_id,
        title: task.title,
        description: task.description,
        status: task.status,
        priority: task.priority,
        dueDate: task.due_date,
        tags: task.tags,
      }
    : {
        spaceId: defaultSpaceId,
        parentTaskId: defaultParentId,
        title: '',
        description: '',
        status: 'inbox',
        priority: 'none',
        dueDate: null,
        tags: [],
      }
}

export function TaskEditor({
  task,
  defaultSpaceId,
  defaultParentId = null,
  spaces,
  possibleParents,
  onSave,
  onClose,
}: TaskEditorProps) {
  const [input, setInput] = useState(() =>
    initialInput(task, defaultSpaceId, defaultParentId),
  )
  const [tagText, setTagText] = useState(() => task?.tags.join(', ') ?? '')
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const parentOptions = useMemo(
    () =>
      possibleParents.filter(
        (candidate) => candidate.id !== task?.id && candidate.space_id === input.spaceId,
      ),
    [input.spaceId, possibleParents, task?.id],
  )

  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onClose()
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [onClose])

  async function submit(event: React.FormEvent) {
    event.preventDefault()
    if (!input.title.trim()) {
      setError('Give the Task a title.')
      return
    }
    setSaving(true)
    setError(null)
    try {
      await onSave({
        ...input,
        title: input.title.trim(),
        tags: tagText
          .split(',')
          .map((tag) => tag.trim())
          .filter((tag, index, tags) => !!tag && tags.indexOf(tag) === index),
      })
      onClose()
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Could not save the Task.')
    } finally {
      setSaving(false)
    }
  }

  const fieldClass =
    'w-full rounded-md border px-3 py-2 text-sm bg-[var(--color-bg)] text-[var(--color-text-primary)] border-[var(--color-border)] focus-ring'

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-6"
      role="dialog"
      aria-modal="true"
      aria-labelledby="task-editor-title"
      onMouseDown={(event) => {
        if (event.currentTarget === event.target) onClose()
      }}
    >
      <div className="absolute inset-0 bg-[var(--color-bg-overlay)]" />
      <form
        onSubmit={submit}
        className="relative w-full max-w-[620px] max-h-[86vh] overflow-y-auto rounded-xl border bg-[var(--color-bg-elevated)] border-[var(--color-border)] shadow-lg"
      >
        <div className="sticky top-0 z-10 flex items-center px-5 h-14 border-b bg-[var(--color-bg-elevated)] border-[var(--color-border)]">
          <h2 id="task-editor-title" className="text-base font-semibold flex-1">
            {task ? 'Edit Task' : defaultParentId ? 'Add subtask' : 'New Task'}
          </h2>
          <button
            type="button"
            onClick={onClose}
            className="p-2 rounded-md hover:bg-[var(--color-bg-tertiary)] focus-ring"
            aria-label="Close editor"
          >
            <X size={16} />
          </button>
        </div>

        <div className="p-5 space-y-4">
          <label className="block text-xs font-medium text-[var(--color-text-secondary)]">
            Title
            <input
              autoFocus
              value={input.title}
              onChange={(event) => setInput({ ...input, title: event.target.value })}
              className={`${fieldClass} mt-1.5`}
              maxLength={200}
            />
          </label>

          <label className="block text-xs font-medium text-[var(--color-text-secondary)]">
            Description
            <textarea
              value={input.description}
              onChange={(event) =>
                setInput({ ...input, description: event.target.value })
              }
              className={`${fieldClass} mt-1.5 min-h-24 resize-y`}
              maxLength={10_000}
            />
          </label>

          <div className="grid grid-cols-2 gap-4">
            <label className="block text-xs font-medium text-[var(--color-text-secondary)]">
              Status
              <select
                value={input.status}
                onChange={(event) =>
                  setInput({ ...input, status: event.target.value as TaskStatus })
                }
                className={`${fieldClass} mt-1.5`}
              >
                {STATUS_OPTIONS.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </label>
            <label className="block text-xs font-medium text-[var(--color-text-secondary)]">
              Priority
              <select
                value={input.priority}
                onChange={(event) =>
                  setInput({ ...input, priority: event.target.value as TaskPriority })
                }
                className={`${fieldClass} mt-1.5`}
              >
                {PRIORITY_OPTIONS.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </label>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <label className="block text-xs font-medium text-[var(--color-text-secondary)]">
              Due date
              <input
                type="date"
                value={input.dueDate ?? ''}
                onChange={(event) =>
                  setInput({ ...input, dueDate: event.target.value || null })
                }
                className={`${fieldClass} mt-1.5`}
              />
            </label>
            <label className="block text-xs font-medium text-[var(--color-text-secondary)]">
              Space
              <select
                value={input.spaceId ?? ''}
                onChange={(event) =>
                  setInput({
                    ...input,
                    spaceId: event.target.value || null,
                    parentTaskId: null,
                  })
                }
                className={`${fieldClass} mt-1.5`}
              >
                <option value="">Global Inbox</option>
                {spaces.map((space) => (
                  <option key={space.id} value={space.id}>
                    {space.name}
                  </option>
                ))}
              </select>
            </label>
          </div>

          <label className="block text-xs font-medium text-[var(--color-text-secondary)]">
            Parent Task
            <select
              value={input.parentTaskId ?? ''}
              onChange={(event) =>
                setInput({ ...input, parentTaskId: event.target.value || null })
              }
              className={`${fieldClass} mt-1.5`}
            >
              <option value="">No parent</option>
              {parentOptions.map((candidate) => (
                <option key={candidate.id} value={candidate.id}>
                  {candidate.title}
                </option>
              ))}
            </select>
          </label>

          <label className="block text-xs font-medium text-[var(--color-text-secondary)]">
            Tags{' '}
            <span className="font-normal text-[var(--color-text-tertiary)]">
              — comma separated
            </span>
            <input
              value={tagText}
              onChange={(event) => setTagText(event.target.value)}
              className={`${fieldClass} mt-1.5`}
              placeholder="research, personal"
            />
          </label>

          {error && (
            <p role="alert" className="text-sm text-[var(--color-danger)]">
              {error}
            </p>
          )}
        </div>

        <div className="flex justify-end gap-2 px-5 py-4 border-t border-[var(--color-border)]">
          <button
            type="button"
            onClick={onClose}
            className="px-3 py-2 rounded-md text-sm hover:bg-[var(--color-bg-tertiary)] focus-ring"
          >
            Cancel
          </button>
          <button
            disabled={saving}
            className="px-4 py-2 rounded-md text-sm font-medium bg-[var(--color-accent)] text-[var(--color-accent-text)] disabled:opacity-50 focus-ring"
          >
            {saving ? 'Saving…' : 'Save Task'}
          </button>
        </div>
      </form>
    </div>
  )
}
