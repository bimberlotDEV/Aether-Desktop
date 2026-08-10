import { useState } from 'react'
import { X } from 'lucide-react'
import type { MemoryCategory, MemoryInput, MemoryItem, Space } from '@/lib/db/types'

const categories: { value: MemoryCategory; label: string }[] = [
  { value: 'preference', label: 'Preference' },
  { value: 'decision', label: 'Decision' },
  { value: 'recurring_context', label: 'Recurring context' },
  { value: 'terminology', label: 'Terminology' },
  { value: 'goal', label: 'Goal' },
  { value: 'constraint', label: 'Constraint' },
]

export function MemoryEditor({
  item,
  spaces,
  lockedSpaceId,
  onSave,
  onClose,
}: {
  item?: MemoryItem
  spaces: Space[]
  lockedSpaceId?: string
  onSave: (input: MemoryInput) => Promise<void>
  onClose: () => void
}) {
  const [title, setTitle] = useState(item?.title ?? '')
  const [content, setContent] = useState(item?.content ?? '')
  const [reason, setReason] = useState(item?.reason ?? '')
  const [category, setCategory] = useState<MemoryCategory>(
    item?.category ?? 'recurring_context',
  )
  const [spaceId, setSpaceId] = useState(lockedSpaceId ?? item?.space_id ?? '')
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function submit(event: React.FormEvent) {
    event.preventDefault()
    setSaving(true)
    setError(null)
    try {
      await onSave({
        spaceId: spaceId || null,
        title: title.trim(),
        content: content.trim(),
        reason: reason.trim(),
        category,
      })
      onClose()
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Could not save Memory.')
    } finally {
      setSaving(false)
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      <div className="absolute inset-0 bg-[var(--color-bg-overlay)]" onClick={onClose} />
      <form
        onSubmit={(event) => void submit(event)}
        role="dialog"
        aria-modal="true"
        aria-labelledby="memory-editor-title"
        className="relative w-full max-w-[600px] rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-elevated)] p-5 shadow-xl"
      >
        <div className="mb-5 flex items-center gap-3">
          <h2 id="memory-editor-title" className="flex-1 text-base font-semibold">
            {item ? 'Edit Memory' : 'Remember something'}
          </h2>
          <button
            type="button"
            onClick={onClose}
            aria-label="Close editor"
            className="rounded-md p-1.5 hover:bg-[var(--color-bg-tertiary)] focus-ring"
          >
            <X size={16} />
          </button>
        </div>
        <div className="space-y-4">
          <label className="block text-sm font-medium">
            Title
            <input
              autoFocus
              required
              maxLength={200}
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              className="mt-1.5 w-full rounded-md border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2 outline-none focus:border-[var(--color-accent)]"
            />
          </label>
          <label className="block text-sm font-medium">
            What should Aether remember?
            <textarea
              required
              maxLength={20000}
              rows={5}
              value={content}
              onChange={(e) => setContent(e.target.value)}
              className="mt-1.5 w-full resize-y rounded-md border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2 outline-none focus:border-[var(--color-accent)]"
            />
          </label>
          <label className="block text-sm font-medium">
            Why is this useful?
            <textarea
              required
              maxLength={500}
              rows={2}
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              className="mt-1.5 w-full resize-none rounded-md border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2 outline-none focus:border-[var(--color-accent)]"
            />
          </label>
          <div className="grid grid-cols-2 gap-3">
            <label className="text-sm font-medium">
              Category
              <select
                value={category}
                onChange={(e) => setCategory(e.target.value as MemoryCategory)}
                className="mt-1.5 w-full rounded-md border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2"
              >
                {categories.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </label>
            {!lockedSpaceId && (
              <label className="text-sm font-medium">
                Applies to
                <select
                  value={spaceId}
                  onChange={(e) => setSpaceId(e.target.value)}
                  className="mt-1.5 w-full rounded-md border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2"
                >
                  <option value="">Everywhere (global)</option>
                  {spaces.map((space) => (
                    <option key={space.id} value={space.id}>
                      {space.name}
                    </option>
                  ))}
                </select>
              </label>
            )}
          </div>
        </div>
        <p className="mt-4 text-xs text-[var(--color-text-tertiary)]">
          Source: you. Aether never turns chats into permanent Memory automatically.
        </p>
        {error && (
          <p role="alert" className="mt-3 text-sm text-[var(--color-danger)]">
            {error}
          </p>
        )}
        <div className="mt-5 flex justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            className="rounded-md border border-[var(--color-border)] px-3 py-2 text-sm font-medium focus-ring"
          >
            Cancel
          </button>
          <button
            disabled={saving || !title.trim() || !content.trim() || !reason.trim()}
            className="rounded-md bg-[var(--color-accent)] px-3 py-2 text-sm font-medium text-[var(--color-accent-text)] disabled:opacity-50 focus-ring"
          >
            {saving ? 'Saving…' : 'Save Memory'}
          </button>
        </div>
      </form>
    </div>
  )
}
