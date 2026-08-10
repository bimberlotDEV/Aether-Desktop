import { useState } from 'react'
import { Brain, Pencil, Plus, Search, Trash2 } from 'lucide-react'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { MemoryEditor } from '@/components/memory/MemoryEditor'
import { useMemory } from '@/hooks/useMemory'
import type { MemoryCategory, MemoryItem, Space } from '@/lib/db/types'

const labels: Record<MemoryCategory, string> = {
  preference: 'Preference',
  decision: 'Decision',
  recurring_context: 'Recurring context',
  terminology: 'Terminology',
  goal: 'Goal',
  constraint: 'Constraint',
}

export function MemoryView({ spaceId, spaces }: { spaceId?: string; spaces: Space[] }) {
  const [search, setSearch] = useState('')
  const [category, setCategory] = useState<MemoryCategory | ''>('')
  const [editing, setEditing] = useState<MemoryItem | 'new' | null>(null)
  const [deleting, setDeleting] = useState<MemoryItem | null>(null)
  const memory = useMemory({
    ...(spaceId ? { spaceId } : {}),
    ...(category ? { category } : {}),
    ...(search.trim() ? { search } : {}),
  })

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="border-b border-[var(--color-border)] px-8 py-6">
        <div className="flex items-start gap-4">
          <div className="flex-1">
            <h1 className="text-xl font-semibold tracking-tight">Memory</h1>
            <p className="mt-1 text-sm text-[var(--color-text-secondary)]">
              Explicit context you control. Nothing is remembered from conversations
              automatically.
            </p>
          </div>
          <button
            onClick={() => setEditing('new')}
            disabled={!memory.isTauri}
            className="inline-flex items-center gap-2 rounded-md bg-[var(--color-accent)] px-3 py-2 text-sm font-medium text-[var(--color-accent-text)] disabled:opacity-50 focus-ring"
          >
            <Plus size={15} /> Remember
          </button>
        </div>
        <div className="mt-5 flex gap-2">
          <label className="flex min-w-0 flex-1 items-center gap-2 rounded-md border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2">
            <Search size={14} className="text-[var(--color-text-tertiary)]" />
            <span className="sr-only">Search Memory</span>
            <input
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="Search Memory"
              className="w-full bg-transparent text-sm outline-none"
            />
          </label>
          <label>
            <span className="sr-only">Filter by category</span>
            <select
              value={category}
              onChange={(e) => setCategory(e.target.value as MemoryCategory | '')}
              className="h-full rounded-md border border-[var(--color-border)] bg-[var(--color-bg)] px-3 text-sm"
            >
              <option value="">All categories</option>
              {Object.entries(labels).map(([value, label]) => (
                <option key={value} value={value}>
                  {label}
                </option>
              ))}
            </select>
          </label>
        </div>
      </div>
      <div className="min-h-0 flex-1 overflow-y-auto px-8 py-6">
        {!memory.isTauri ? (
          <Empty
            title="Memory is available in the desktop app"
            detail="Aether does not fabricate Memory in browser preview mode."
          />
        ) : memory.loading ? (
          <p className="py-12 text-center text-sm text-[var(--color-text-tertiary)]">
            Loading Memory…
          </p>
        ) : memory.error ? (
          <p
            role="alert"
            className="rounded-md border border-[var(--color-danger)] p-3 text-sm text-[var(--color-danger)]"
          >
            {memory.error}
          </p>
        ) : memory.items.length === 0 ? (
          <Empty
            title={search || category ? 'No matching Memory' : 'Nothing remembered yet'}
            detail={
              search || category
                ? 'Try another search or category.'
                : 'Add only durable context you want to review and control later.'
            }
          />
        ) : (
          <div className="grid grid-cols-1 gap-3 xl:grid-cols-2">
            {memory.items.map((item) => (
              <article
                key={item.id}
                className="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4"
              >
                <div className="flex items-start gap-3">
                  <Brain size={16} className="mt-0.5 text-[var(--color-accent)]" />
                  <div className="min-w-0 flex-1">
                    <div className="flex items-start gap-2">
                      <h2 className="flex-1 text-sm font-semibold">{item.title}</h2>
                      <span className="rounded bg-[var(--color-bg-tertiary)] px-1.5 py-0.5 text-[11px] text-[var(--color-text-tertiary)]">
                        {labels[item.category]}
                      </span>
                    </div>
                    <p className="mt-2 whitespace-pre-wrap text-sm leading-relaxed text-[var(--color-text-secondary)]">
                      {item.content}
                    </p>
                    <p className="mt-3 text-xs text-[var(--color-text-tertiary)]">
                      <span className="font-medium">Why:</span> {item.reason}
                    </p>
                    <div className="mt-3 flex items-center gap-2 text-xs text-[var(--color-text-tertiary)]">
                      <span>
                        {item.space_id
                          ? (spaces.find((space) => space.id === item.space_id)?.name ??
                            'Space')
                          : 'Global'}
                      </span>
                      <span>·</span>
                      <span>Added by you</span>
                      <span className="flex-1" />
                      <button
                        onClick={() => setEditing(item)}
                        aria-label={`Edit ${item.title}`}
                        className="rounded p-1 hover:bg-[var(--color-bg-tertiary)] focus-ring"
                      >
                        <Pencil size={14} />
                      </button>
                      <button
                        onClick={() => setDeleting(item)}
                        aria-label={`Delete ${item.title}`}
                        className="rounded p-1 text-[var(--color-danger)] hover:bg-[var(--color-bg-tertiary)] focus-ring"
                      >
                        <Trash2 size={14} />
                      </button>
                    </div>
                  </div>
                </div>
              </article>
            ))}
          </div>
        )}
      </div>
      {editing && (
        <MemoryEditor
          item={editing === 'new' ? undefined : editing}
          spaces={spaces}
          lockedSpaceId={spaceId}
          onClose={() => setEditing(null)}
          onSave={async (input) => {
            if (editing === 'new') await memory.create(input)
            else await memory.update(editing.id, input)
          }}
        />
      )}
      {deleting && (
        <ConfirmDialog
          title="Delete Memory?"
          message={`Permanently delete “${deleting.title}”? It will also disappear from any AI conversation context.`}
          confirmLabel="Delete Memory"
          danger
          onCancel={() => setDeleting(null)}
          onConfirm={() => {
            const id = deleting.id
            setDeleting(null)
            void memory.remove(id)
          }}
        />
      )}
    </div>
  )
}

function Empty({ title, detail }: { title: string; detail: string }) {
  return (
    <div className="flex flex-col items-center py-16 text-center">
      <Brain size={28} className="mb-3 text-[var(--color-text-tertiary)]" />
      <h2 className="text-sm font-semibold">{title}</h2>
      <p className="mt-1 max-w-sm text-sm text-[var(--color-text-tertiary)]">{detail}</p>
    </div>
  )
}
