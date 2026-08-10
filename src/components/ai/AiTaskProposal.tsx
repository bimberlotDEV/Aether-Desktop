import { useMemo, useState } from 'react'
import { CheckSquare, X } from 'lucide-react'
import type { TaskInput } from '@/lib/db/types'
import * as db from '@/lib/db/tauri'
import type { TaskProposal } from '@/lib/aiProposal'

export function AiTaskProposal({
  tasks,
  spaceId,
  onClose,
  onCreated,
}: {
  tasks: TaskProposal
  spaceId?: string
  onClose: () => void
  onCreated: () => void
}) {
  const [selected, setSelected] = useState(() => new Set(tasks.map((_, index) => index)))
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const selectedTasks = useMemo(
    () => tasks.filter((_, index) => selected.has(index)),
    [selected, tasks],
  )

  async function create() {
    if (!selectedTasks.length) return
    setSaving(true)
    setError(null)
    const inputs: TaskInput[] = selectedTasks.map((task) => ({
      spaceId: spaceId ?? null,
      parentTaskId: null,
      title: task.title,
      description: task.description,
      status: 'inbox',
      priority: task.priority,
      dueDate: task.dueDate,
      tags: [...new Set(task.tags)],
    }))
    try {
      await db.createTasksBatch(inputs)
      onCreated()
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Could not create the Tasks.')
    } finally {
      setSaving(false)
    }
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4"
      onClick={onClose}
    >
      <div className="absolute inset-0 bg-[var(--color-bg-overlay)]" />
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby="proposal-title"
        onClick={(event) => event.stopPropagation()}
        className="relative w-full max-w-[560px] rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-elevated)] p-5 shadow-xl"
      >
        <div className="mb-4 flex items-start gap-3">
          <div className="flex-1">
            <h2 id="proposal-title" className="text-base font-semibold">
              Review proposed Tasks
            </h2>
            <p className="mt-1 text-xs text-[var(--color-text-tertiary)]">
              Nothing is created until you confirm. Selected Tasks are created together or
              not at all.
            </p>
          </div>
          <button
            onClick={onClose}
            aria-label="Close proposal"
            className="rounded-md p-1.5 hover:bg-[var(--color-bg-tertiary)] focus-ring"
          >
            <X size={16} />
          </button>
        </div>
        <div className="max-h-[50vh] space-y-1 overflow-y-auto">
          {tasks.map((task, index) => (
            <label
              key={`${task.title}-${index}`}
              className="flex cursor-pointer items-start gap-3 rounded-lg p-3 hover:bg-[var(--color-bg-tertiary)]"
            >
              <input
                type="checkbox"
                checked={selected.has(index)}
                onChange={() =>
                  setSelected((current) => {
                    const next = new Set(current)
                    if (next.has(index)) next.delete(index)
                    else next.add(index)
                    return next
                  })
                }
                className="mt-1"
              />
              <CheckSquare size={16} className="mt-0.5 text-[var(--color-accent)]" />
              <span className="min-w-0 flex-1">
                <span className="block text-sm font-medium">{task.title}</span>
                {task.description && (
                  <span className="mt-0.5 block text-xs text-[var(--color-text-tertiary)]">
                    {task.description}
                  </span>
                )}
                <span className="mt-1 block text-xs capitalize text-[var(--color-text-tertiary)]">
                  {task.priority} priority{task.dueDate ? ` · ${task.dueDate}` : ''}
                </span>
              </span>
            </label>
          ))}
        </div>
        {error && (
          <p role="alert" className="mt-3 text-sm text-[var(--color-danger)]">
            {error}
          </p>
        )}
        <div className="mt-5 flex justify-end gap-2">
          <button
            onClick={onClose}
            disabled={saving}
            className="rounded-md border border-[var(--color-border)] px-3 py-2 text-sm font-medium hover:bg-[var(--color-bg-tertiary)] focus-ring"
          >
            Cancel
          </button>
          <button
            onClick={() => void create()}
            disabled={saving || !selectedTasks.length}
            className="rounded-md bg-[var(--color-accent)] px-3 py-2 text-sm font-medium text-[var(--color-accent-text)] disabled:opacity-50 focus-ring"
          >
            {saving
              ? 'Creating…'
              : `Create ${selectedTasks.length} Task${selectedTasks.length === 1 ? '' : 's'}`}
          </button>
        </div>
      </div>
    </div>
  )
}
