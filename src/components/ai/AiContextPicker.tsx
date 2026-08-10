import { useEffect, useMemo, useState } from 'react'
import { Check, CheckSquare, File, FileText, Search, X } from 'lucide-react'
import type { AiContextItem, NoteListItem, Task, VaultItem } from '@/lib/db/types'
import * as db from '@/lib/db/tauri'

type Candidate = {
  id: string
  type: 'note' | 'task' | 'vault'
  title: string
  detail: string
}

export function AiContextPicker({
  spaceId,
  attached,
  onAttach,
  onClose,
}: {
  spaceId?: string
  attached: AiContextItem[]
  onAttach: (type: 'note' | 'task' | 'vault', id: string) => Promise<void>
  onClose: () => void
}) {
  const [candidates, setCandidates] = useState<Candidate[]>([])
  const [search, setSearch] = useState('')
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [adding, setAdding] = useState<string | null>(null)

  useEffect(() => {
    let active = true
    Promise.all([
      db.listRecentNotes(spaceId, 100),
      db.listTasks({ ...(spaceId ? { spaceId } : {}), limit: 100 }),
      db.listVaultItems({ ...(spaceId ? { spaceId } : {}), limit: 100 }),
    ])
      .then(([notes, tasks, files]) => {
        if (!active) return
        setCandidates([
          ...notes.map((note: NoteListItem) => ({
            id: note.id,
            type: 'note' as const,
            title: note.title || 'Untitled note',
            detail: note.excerpt || 'Markdown note',
          })),
          ...tasks.map((task: Task) => ({
            id: task.id,
            type: 'task' as const,
            title: task.title,
            detail: `${task.status.replace('_', ' ')}${task.due_date ? ` · due ${task.due_date}` : ''}`,
          })),
          ...files.map((file: VaultItem) => ({
            id: file.id,
            type: 'vault' as const,
            title: file.display_title,
            detail: `${file.original_name} · metadata only`,
          })),
        ])
      })
      .catch((cause) => {
        if (active)
          setError(
            cause instanceof Error ? cause.message : 'Could not load context choices.',
          )
      })
      .finally(() => {
        if (active) setLoading(false)
      })
    return () => {
      active = false
    }
  }, [spaceId])

  const attachedKeys = new Set(
    attached.map((item) => `${item.entity_type}:${item.entity_id}`),
  )
  const filtered = useMemo(() => {
    const query = search.trim().toLocaleLowerCase()
    return query
      ? candidates.filter((item) =>
          `${item.title} ${item.detail} ${item.type}`.toLocaleLowerCase().includes(query),
        )
      : candidates
  }, [candidates, search])

  const icons = { note: FileText, task: CheckSquare, vault: File }
  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4"
      onClick={onClose}
    >
      <div className="absolute inset-0 bg-[var(--color-bg-overlay)]" />
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby="context-picker-title"
        onClick={(event) => event.stopPropagation()}
        className="relative flex max-h-[72vh] w-full max-w-[560px] flex-col rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-elevated)] shadow-xl"
      >
        <div className="flex items-start gap-3 border-b border-[var(--color-border)] p-5">
          <div className="flex-1">
            <h2 id="context-picker-title" className="text-base font-semibold">
              Attach context
            </h2>
            <p className="mt-1 text-xs text-[var(--color-text-tertiary)]">
              Only items you attach here are sent with this conversation. Vault files
              share metadata, never file content.
            </p>
          </div>
          <button
            onClick={onClose}
            aria-label="Close context picker"
            className="rounded-md p-1.5 hover:bg-[var(--color-bg-tertiary)] focus-ring"
          >
            <X size={16} />
          </button>
        </div>
        <label className="m-4 flex items-center gap-2 rounded-md border border-[var(--color-border)] px-3 py-2">
          <Search size={14} className="text-[var(--color-text-tertiary)]" />
          <span className="sr-only">Search context</span>
          <input
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder="Search Notes, Tasks, and Vault"
            className="w-full bg-transparent text-sm outline-none"
            autoFocus
          />
        </label>
        <div className="min-h-0 flex-1 overflow-y-auto px-3 pb-4">
          {loading ? (
            <p className="py-10 text-center text-sm text-[var(--color-text-tertiary)]">
              Loading context…
            </p>
          ) : error ? (
            <p role="alert" className="px-2 py-4 text-sm text-[var(--color-danger)]">
              {error}
            </p>
          ) : filtered.length === 0 ? (
            <p className="py-10 text-center text-sm text-[var(--color-text-tertiary)]">
              No matching items
            </p>
          ) : (
            filtered.map((item) => {
              const Icon = icons[item.type]
              const isAttached = attachedKeys.has(`${item.type}:${item.id}`)
              return (
                <button
                  key={`${item.type}:${item.id}`}
                  disabled={isAttached || adding !== null}
                  onClick={async () => {
                    setAdding(item.id)
                    try {
                      await onAttach(item.type, item.id)
                    } finally {
                      setAdding(null)
                    }
                  }}
                  className="flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-left hover:bg-[var(--color-bg-tertiary)] disabled:opacity-60 focus-ring"
                >
                  <Icon size={16} className="text-[var(--color-text-tertiary)]" />
                  <span className="min-w-0 flex-1">
                    <span className="block truncate text-sm font-medium">
                      {item.title}
                    </span>
                    <span className="block truncate text-xs capitalize text-[var(--color-text-tertiary)]">
                      {item.type} · {item.detail}
                    </span>
                  </span>
                  {isAttached && (
                    <Check size={15} className="text-[var(--color-success)]" />
                  )}
                </button>
              )
            })
          )}
        </div>
      </div>
    </div>
  )
}
