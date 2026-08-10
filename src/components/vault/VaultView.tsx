import { useMemo, useState } from 'react'
import { open as selectFile } from '@tauri-apps/plugin-dialog'
import {
  ExternalLink,
  File,
  FileInput,
  FolderOpen,
  Link2,
  MoreHorizontal,
  Pencil,
  Plus,
  Search,
  Trash2,
} from 'lucide-react'
import type { Space, VaultItem, VaultStorageMode } from '@/lib/db/types'
import { useVault } from '@/hooks/useVault'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { VaultEditor } from '@/components/vault/VaultEditor'

interface VaultViewProps {
  spaces: Space[]
  spaceId?: string
  title?: string
  description?: string
}

function formatSize(bytes: number) {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(bytes < 10240 ? 1 : 0)} KB`
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / (1024 * 1024)).toFixed(bytes < 10 * 1024 * 1024 ? 1 : 0)} MB`
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`
}

function formatDate(value: string) {
  return new Date(value).toLocaleDateString(undefined, {
    month: 'short',
    day: 'numeric',
    year:
      new Date(value).getFullYear() === new Date().getFullYear() ? undefined : 'numeric',
  })
}

function VaultRow({
  item,
  space,
  onOpen,
  onReveal,
  onEdit,
  onRemove,
}: {
  item: VaultItem
  space?: Space
  onOpen: () => void
  onReveal: () => void
  onEdit: () => void
  onRemove: () => void
}) {
  const [menuOpen, setMenuOpen] = useState(false)
  return (
    <div className="group grid grid-cols-[minmax(0,1fr)_110px_110px_40px] items-center gap-4 border-b border-[var(--color-border)] px-3 py-3">
      <button
        onClick={onOpen}
        aria-label={`Open ${item.display_title}`}
        className="flex min-w-0 items-center gap-3 rounded-sm text-left focus-ring"
      >
        <span className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-[var(--color-bg-secondary)] text-[var(--color-text-secondary)]">
          <File size={17} strokeWidth={1.7} />
        </span>
        <span className="min-w-0">
          <span className="block truncate text-sm font-medium text-[var(--color-text-primary)]">
            {item.display_title}
          </span>
          <span className="mt-0.5 flex items-center gap-1.5 truncate text-xs text-[var(--color-text-tertiary)]">
            {item.storage_mode === 'linked' ? (
              <Link2 size={11} />
            ) : (
              <FileInput size={11} />
            )}
            <span>{item.storage_mode === 'linked' ? 'Linked' : 'Managed'}</span>
            <span>·</span>
            <span className="truncate">
              {item.original_name}
              {item.tags.length
                ? ` · ${item.tags.map((tag) => `#${tag}`).join(' ')}`
                : ''}
            </span>
          </span>
        </span>
      </button>
      <span className="truncate text-xs text-[var(--color-text-secondary)]">
        {space?.name ?? 'No Space'}
      </span>
      <span className="text-xs text-[var(--color-text-tertiary)]">
        {formatSize(item.size_bytes)} · {formatDate(item.updated_at)}
      </span>
      <div className="relative flex justify-end">
        <button
          onClick={() => setMenuOpen((open) => !open)}
          className="rounded-md p-1.5 text-[var(--color-text-tertiary)] opacity-0 hover:bg-[var(--color-bg-tertiary)] group-hover:opacity-100 focus:opacity-100 focus-ring"
          aria-label={`Actions for ${item.display_title}`}
        >
          <MoreHorizontal size={16} />
        </button>
        {menuOpen && (
          <div className="absolute right-0 top-8 z-20 w-44 rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-elevated)] p-1 shadow-lg">
            <button
              onClick={() => {
                setMenuOpen(false)
                onOpen()
              }}
              className="flex w-full items-center gap-2 rounded-md px-2.5 py-2 text-left text-sm hover:bg-[var(--color-bg-tertiary)]"
            >
              <ExternalLink size={14} /> Open
            </button>
            <button
              onClick={() => {
                setMenuOpen(false)
                onReveal()
              }}
              className="flex w-full items-center gap-2 rounded-md px-2.5 py-2 text-left text-sm hover:bg-[var(--color-bg-tertiary)]"
            >
              <FolderOpen size={14} /> Show in folder
            </button>
            <button
              onClick={() => {
                setMenuOpen(false)
                onEdit()
              }}
              className="flex w-full items-center gap-2 rounded-md px-2.5 py-2 text-left text-sm hover:bg-[var(--color-bg-tertiary)]"
            >
              <Pencil size={14} /> Edit details
            </button>
            <button
              onClick={() => {
                setMenuOpen(false)
                onRemove()
              }}
              className="flex w-full items-center gap-2 rounded-md px-2.5 py-2 text-left text-sm text-[var(--color-danger)] hover:bg-[var(--color-bg-tertiary)]"
            >
              <Trash2 size={14} /> Remove
            </button>
          </div>
        )}
      </div>
    </div>
  )
}

export function VaultView({
  spaces,
  spaceId,
  title = spaceId ? 'Files' : 'Vault',
  description = spaceId
    ? 'Files connected to this Space.'
    : 'Keep references and managed copies organized across your Spaces.',
}: VaultViewProps) {
  const [search, setSearch] = useState('')
  const [mode, setMode] = useState<VaultStorageMode | 'all'>('all')
  const [selectedSpace, setSelectedSpace] = useState(spaceId ?? 'all')
  const [importPath, setImportPath] = useState<string | null>(null)
  const [editItem, setEditItem] = useState<VaultItem | null>(null)
  const [removeItem, setRemoveItem] = useState<VaultItem | null>(null)
  const [actionError, setActionError] = useState<string | null>(null)

  const filter = useMemo(
    () => ({
      ...(spaceId
        ? { spaceId }
        : selectedSpace === 'unassigned'
          ? { unassignedOnly: true }
          : selectedSpace !== 'all'
            ? { spaceId: selectedSpace }
            : {}),
      ...(mode !== 'all' ? { storageMode: mode } : {}),
      ...(search.trim() ? { search: search.trim() } : {}),
    }),
    [mode, search, selectedSpace, spaceId],
  )
  const vault = useVault(filter)
  const spacesById = useMemo(
    () => new Map(spaces.map((space) => [space.id, space])),
    [spaces],
  )

  async function chooseFile() {
    setActionError(null)
    if (!vault.isTauri) {
      setActionError('File import is available in the Aether desktop app.')
      return
    }
    try {
      const selected = await selectFile({ multiple: false, directory: false })
      if (typeof selected === 'string') setImportPath(selected)
    } catch (cause) {
      setActionError(
        cause instanceof Error ? cause.message : 'Could not open the file picker.',
      )
    }
  }

  function runNative(operation: Promise<void>) {
    setActionError(null)
    void operation.catch((cause) => {
      setActionError(
        cause instanceof Error ? cause.message : 'The native file action failed.',
      )
    })
  }

  const hasFilters =
    !!search.trim() || mode !== 'all' || (!spaceId && selectedSpace !== 'all')
  return (
    <div className="mx-auto max-w-[940px] px-8 py-7">
      <div className="mb-6 flex items-start gap-4">
        <div className="flex-1">
          <h1 className="text-xl font-semibold text-[var(--color-text-primary)]">
            {title}
          </h1>
          <p className="mt-1 text-sm text-[var(--color-text-secondary)]">{description}</p>
        </div>
        <button
          onClick={() => void chooseFile()}
          className="inline-flex items-center gap-2 rounded-md bg-[var(--color-accent)] px-3 py-2 text-sm font-medium text-[var(--color-accent-text)] focus-ring"
        >
          <Plus size={15} /> Add file
        </button>
      </div>

      <div className="mb-4 flex flex-wrap items-center gap-2">
        <label className="flex h-9 min-w-56 flex-1 items-center gap-2 rounded-md border border-[var(--color-border)] bg-[var(--color-bg)] px-3">
          <Search size={14} className="text-[var(--color-text-tertiary)]" />
          <span className="sr-only">Search Vault</span>
          <input
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder="Search titles, filenames, and tags"
            className="w-full bg-transparent text-sm outline-none"
          />
        </label>
        <select
          aria-label="Filter by storage"
          value={mode}
          onChange={(event) => setMode(event.target.value as VaultStorageMode | 'all')}
          className="h-9 rounded-md border border-[var(--color-border)] bg-[var(--color-bg)] px-2.5 text-sm outline-none"
        >
          <option value="all">All storage</option>
          <option value="linked">Linked</option>
          <option value="managed">Managed</option>
        </select>
        {!spaceId && (
          <select
            aria-label="Filter by Space"
            value={selectedSpace}
            onChange={(event) => setSelectedSpace(event.target.value)}
            className="h-9 rounded-md border border-[var(--color-border)] bg-[var(--color-bg)] px-2.5 text-sm outline-none"
          >
            <option value="all">All Spaces</option>
            <option value="unassigned">No Space</option>
            {spaces.map((space) => (
              <option key={space.id} value={space.id}>
                {space.name}
              </option>
            ))}
          </select>
        )}
      </div>

      {(actionError || vault.error) && (
        <div
          role="alert"
          className="mb-4 rounded-lg border border-[var(--color-danger)]/30 bg-[var(--color-bg-secondary)] px-3 py-2 text-sm text-[var(--color-danger)]"
        >
          {actionError ?? vault.error}
        </div>
      )}

      {vault.loading ? (
        <p className="py-16 text-center text-sm text-[var(--color-text-tertiary)]">
          Loading files…
        </p>
      ) : vault.items.length === 0 ? (
        <div className="flex flex-col items-center justify-center rounded-xl border border-dashed border-[var(--color-border)] px-8 py-16 text-center">
          <div className="mb-4 flex h-11 w-11 items-center justify-center rounded-xl bg-[var(--color-accent-muted)] text-[var(--color-accent)]">
            {hasFilters ? <Search size={20} /> : <FolderOpen size={20} />}
          </div>
          <h2 className="text-base font-semibold text-[var(--color-text-primary)]">
            {hasFilters
              ? 'No matching files'
              : spaceId
                ? 'No files in this Space'
                : 'Your Vault is empty'}
          </h2>
          <p className="mt-1 max-w-sm text-sm leading-relaxed text-[var(--color-text-secondary)]">
            {hasFilters
              ? 'Try a different search or filter.'
              : 'Link an existing file or let Aether keep a managed copy.'}
          </p>
          {!hasFilters && (
            <button
              onClick={() => void chooseFile()}
              className="mt-5 inline-flex items-center gap-2 rounded-md border border-[var(--color-border)] px-3 py-2 text-sm font-medium hover:bg-[var(--color-bg-tertiary)] focus-ring"
            >
              <FileInput size={15} /> Choose a file
            </button>
          )}
        </div>
      ) : (
        <div className="overflow-visible rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)]">
          <div className="grid grid-cols-[minmax(0,1fr)_110px_110px_40px] gap-4 border-b border-[var(--color-border)] px-3 py-2 text-xs font-medium text-[var(--color-text-tertiary)]">
            <span>File</span>
            <span>Space</span>
            <span>Details</span>
            <span />
          </div>
          {vault.items.map((item) => (
            <VaultRow
              key={item.id}
              item={item}
              space={item.space_id ? spacesById.get(item.space_id) : undefined}
              onOpen={() => runNative(vault.openItem(item.id))}
              onReveal={() => runNative(vault.reveal(item.id))}
              onEdit={() => setEditItem(item)}
              onRemove={() => setRemoveItem(item)}
            />
          ))}
        </div>
      )}

      {importPath && (
        <VaultEditor
          path={importPath}
          spaces={spaces}
          fixedSpaceId={spaceId}
          onImport={vault.importItem}
          onUpdate={vault.update}
          onClose={() => setImportPath(null)}
        />
      )}
      {editItem && (
        <VaultEditor
          item={editItem}
          spaces={spaces}
          fixedSpaceId={spaceId}
          onImport={vault.importItem}
          onUpdate={vault.update}
          onClose={() => setEditItem(null)}
        />
      )}
      {removeItem && (
        <ConfirmDialog
          title={`Remove “${removeItem.display_title}”?`}
          message={
            removeItem.storage_mode === 'linked'
              ? 'This removes the Vault reference. The original file stays exactly where it is.'
              : 'This permanently deletes Aether’s managed Vault copy. The original file you imported from is not affected.'
          }
          confirmLabel={
            removeItem.storage_mode === 'linked'
              ? 'Remove reference'
              : 'Delete managed copy'
          }
          danger
          onConfirm={() => {
            const id = removeItem.id
            setRemoveItem(null)
            void vault.remove(id).catch(() => undefined)
          }}
          onCancel={() => setRemoveItem(null)}
        />
      )}
    </div>
  )
}
