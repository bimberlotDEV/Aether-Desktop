import { useEffect, useRef, useState } from 'react'
import { FileInput, Link2, X } from 'lucide-react'
import type {
  Space,
  VaultImportInput,
  VaultItem,
  VaultStorageMode,
  VaultUpdateInput,
} from '@/lib/db/types'
import { cn } from '@/lib/utils'

interface VaultEditorProps {
  path?: string
  item?: VaultItem
  spaces: Space[]
  fixedSpaceId?: string
  onImport: (input: VaultImportInput) => Promise<unknown>
  onUpdate: (id: string, input: VaultUpdateInput) => Promise<unknown>
  onClose: () => void
}

function filename(path: string) {
  return path.split(/[\\/]/).pop() || path
}

function parseTags(value: string) {
  return [
    ...new Set(
      value
        .split(',')
        .map((tag) => tag.trim())
        .filter(Boolean),
    ),
  ]
}

export function VaultEditor({
  path,
  item,
  spaces,
  fixedSpaceId,
  onImport,
  onUpdate,
  onClose,
}: VaultEditorProps) {
  const isEdit = !!item
  const initialTitle =
    item?.display_title ?? (path ? filename(path).replace(/\.[^.]+$/, '') : '')
  const [title, setTitle] = useState(initialTitle)
  const [tags, setTags] = useState(item?.tags.join(', ') ?? '')
  const [spaceId, setSpaceId] = useState<string | null>(
    fixedSpaceId ?? item?.space_id ?? null,
  )
  const [storageMode, setStorageMode] = useState<VaultStorageMode>('linked')
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const titleRef = useRef<HTMLInputElement>(null)

  useEffect(() => titleRef.current?.focus(), [])
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && !saving) onClose()
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [onClose, saving])

  async function submit(event: React.FormEvent) {
    event.preventDefault()
    const cleanTitle = title.trim()
    if (!cleanTitle) {
      setError('Give this file a title.')
      return
    }
    const parsedTags = parseTags(tags)
    if (parsedTags.length > 20 || parsedTags.some((tag) => tag.length > 40)) {
      setError('Use at most 20 tags, each no longer than 40 characters.')
      return
    }
    setSaving(true)
    setError(null)
    try {
      if (item) {
        await onUpdate(item.id, { displayTitle: cleanTitle, spaceId, tags: parsedTags })
      } else if (path) {
        await onImport({
          path,
          storageMode,
          spaceId,
          displayTitle: cleanTitle,
          tags: parsedTags,
        })
      }
      onClose()
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Could not save this file.')
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
      <form
        onSubmit={submit}
        onClick={(event) => event.stopPropagation()}
        className="relative w-full max-w-[520px] rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-elevated)] p-6 shadow-xl"
      >
        <div className="mb-5 flex items-start gap-3">
          <div className="flex-1">
            <h2 className="text-lg font-semibold text-[var(--color-text-primary)]">
              {isEdit ? 'Edit Vault item' : 'Add to Vault'}
            </h2>
            <p className="mt-1 truncate text-xs text-[var(--color-text-tertiary)]">
              {item?.original_name ?? (path ? filename(path) : '')}
            </p>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="rounded-md p-1.5 hover:bg-[var(--color-bg-tertiary)] focus-ring"
            aria-label="Close editor"
          >
            <X size={16} />
          </button>
        </div>

        {!isEdit && (
          <fieldset className="mb-5">
            <legend className="mb-2 text-xs font-semibold uppercase tracking-wide text-[var(--color-text-tertiary)]">
              Storage
            </legend>
            <div className="grid grid-cols-2 gap-2">
              {(['linked', 'managed'] as const).map((mode) => {
                const Icon = mode === 'linked' ? Link2 : FileInput
                return (
                  <label
                    key={mode}
                    className={cn(
                      'cursor-pointer rounded-lg border p-3 transition-colors',
                      storageMode === mode
                        ? 'border-[var(--color-accent)] bg-[var(--color-accent-muted)]'
                        : 'border-[var(--color-border)] hover:border-[var(--color-border-hover)]',
                    )}
                  >
                    <input
                      type="radio"
                      name="storage"
                      value={mode}
                      checked={storageMode === mode}
                      onChange={() => setStorageMode(mode)}
                      className="sr-only"
                    />
                    <span className="flex items-center gap-2 text-sm font-medium">
                      <Icon size={15} />
                      {mode === 'linked' ? 'Keep linked' : 'Copy into Vault'}
                    </span>
                    <span className="mt-1 block text-xs leading-relaxed text-[var(--color-text-tertiary)]">
                      {mode === 'linked'
                        ? 'The original stays where it is.'
                        : 'A private managed copy is stored by Aether.'}
                    </span>
                  </label>
                )
              })}
            </div>
          </fieldset>
        )}

        <div className="space-y-4">
          <label className="block text-sm font-medium">
            Title
            <input
              ref={titleRef}
              value={title}
              onChange={(event) => setTitle(event.target.value)}
              maxLength={200}
              className="mt-1.5 w-full rounded-md border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2 text-sm outline-none focus:border-[var(--color-accent)]"
            />
          </label>
          {!fixedSpaceId && (
            <label className="block text-sm font-medium">
              Space
              <select
                value={spaceId ?? ''}
                onChange={(event) => setSpaceId(event.target.value || null)}
                className="mt-1.5 w-full rounded-md border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2 text-sm outline-none focus:border-[var(--color-accent)]"
              >
                <option value="">No Space</option>
                {spaces.map((space) => (
                  <option key={space.id} value={space.id}>
                    {space.name}
                  </option>
                ))}
              </select>
            </label>
          )}
          <label className="block text-sm font-medium">
            Tags{' '}
            <span className="font-normal text-[var(--color-text-tertiary)]">
              (comma separated)
            </span>
            <input
              value={tags}
              onChange={(event) => setTags(event.target.value)}
              placeholder="reference, project"
              className="mt-1.5 w-full rounded-md border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2 text-sm outline-none focus:border-[var(--color-accent)]"
            />
          </label>
        </div>

        {error && (
          <p role="alert" className="mt-4 text-sm text-[var(--color-danger)]">
            {error}
          </p>
        )}
        <div className="mt-6 flex justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            disabled={saving}
            className="rounded-md border border-[var(--color-border)] px-4 py-2 text-sm font-medium hover:bg-[var(--color-bg-tertiary)] focus-ring"
          >
            Cancel
          </button>
          <button
            disabled={saving}
            className="rounded-md bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-[var(--color-accent-text)] disabled:opacity-50 focus-ring"
          >
            {saving ? 'Saving…' : isEdit ? 'Save changes' : 'Add to Vault'}
          </button>
        </div>
      </form>
    </div>
  )
}
