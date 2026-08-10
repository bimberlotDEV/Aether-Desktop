import { useState, useEffect, useRef } from 'react'
import { useParams } from 'react-router-dom'
import {
  FileText,
  Plus,
  Search,
  Pin,
  PinOff,
  Archive,
  Copy,
  Trash2,
  MoreHorizontal,
  ChevronDown,
  ChevronRight,
  RotateCcw,
} from 'lucide-react'
import { useNotes, useNote, useNoteSearch } from '@/hooks/useNotes'
import type { NoteListItem } from '@/lib/db/types'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { cn } from '@/lib/utils'

const SAVE_STATUS: Record<string, { label: string; color: string }> = {
  saved: { label: 'Saved', color: 'var(--color-text-tertiary)' },
  saving: { label: 'Saving…', color: 'var(--color-warning)' },
  unsaved: { label: 'Unsaved', color: 'var(--color-warning)' },
  failed: { label: 'Save failed', color: 'var(--color-danger)' },
}

export function NotesView() {
  const { spaceId } = useParams<{ spaceId: string }>()
  const {
    notes,
    archivedNotes,
    loading,
    create,
    remove,
    pin,
    archive,
    restore,
    duplicate,
  } = useNotes(spaceId)
  const { results: searchResults, searching, search } = useNoteSearch()
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [searchQuery, setSearchQuery] = useState('')
  const [menuOpen, setMenuOpen] = useState<string | null>(null)
  const [deleteTarget, setDeleteTarget] = useState<NoteListItem | null>(null)
  const [showArchived, setShowArchived] = useState(false)

  useEffect(() => {
    void search(searchQuery, spaceId)
  }, [searchQuery, search, spaceId])

  const normalizedQuery = searchQuery.trim().toLowerCase()
  const backendMatches = new Set(searchResults.map((result) => result.id))
  const filtered = normalizedQuery
    ? notes.filter(
        (note) =>
          backendMatches.has(note.id) ||
          note.title.toLowerCase().includes(normalizedQuery) ||
          note.excerpt.toLowerCase().includes(normalizedQuery),
      )
    : notes

  const pinnedNotes = filtered.filter((n) => n.pinned)
  const otherNotes = filtered.filter((n) => !n.pinned)

  const handleCreate = async () => {
    const note = await create()
    if (note) setSelectedId(note.id)
  }

  return (
    <div className="flex h-full min-h-0">
      {/* List panel */}
      <div
        className="w-[280px] shrink-0 flex flex-col h-full border-r"
        style={{ borderColor: 'var(--color-border)' }}
      >
        {/* Header */}
        <div
          className="px-4 py-3 flex items-center gap-2 shrink-0"
          style={{ borderBottom: '1px solid var(--color-border)' }}
        >
          <div className="flex-1 relative">
            <Search
              size={14}
              className="absolute left-2.5 top-1/2 -translate-y-1/2"
              style={{ color: 'var(--color-text-tertiary)' }}
            />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search notes…"
              className="w-full pl-8 pr-3 py-1.5 rounded-md text-sm outline-none"
              style={{
                backgroundColor: 'var(--color-bg-tertiary)',
                color: 'var(--color-text-primary)',
              }}
            />
          </div>
          <button
            onClick={handleCreate}
            className="p-1.5 rounded-md shrink-0 transition-colors hover:bg-[var(--color-bg-tertiary)]"
            style={{ color: 'var(--color-accent)' }}
            title="New note"
          >
            <Plus size={18} strokeWidth={2} />
          </button>
        </div>

        {/* Note list */}
        <div className="flex-1 overflow-y-auto py-1">
          {loading ? (
            <p
              className="px-4 py-8 text-sm text-center"
              style={{ color: 'var(--color-text-tertiary)' }}
            >
              Loading…
            </p>
          ) : filtered.length === 0 ? (
            <div className="px-4 py-10 text-center">
              <p className="text-sm" style={{ color: 'var(--color-text-tertiary)' }}>
                {searchQuery
                  ? searching
                    ? 'Searching…'
                    : 'No notes match your search'
                  : 'No notes yet'}
              </p>
              {!searchQuery && (
                <button
                  onClick={handleCreate}
                  className="mt-3 text-sm font-medium"
                  style={{ color: 'var(--color-accent)' }}
                >
                  Create your first note
                </button>
              )}
            </div>
          ) : (
            <>
              {pinnedNotes.length > 0 && (
                <div className="mb-1">
                  <div
                    className="px-4 py-1 text-[10px] font-semibold uppercase tracking-wider"
                    style={{ color: 'var(--color-text-tertiary)' }}
                  >
                    Pinned
                  </div>
                  {pinnedNotes.map((n) => (
                    <NoteRow
                      key={n.id}
                      note={n}
                      isActive={selectedId === n.id}
                      onSelect={() => setSelectedId(n.id)}
                      onPin={() => pin(n.id, false)}
                      onArchive={() => {
                        archive(n.id)
                        if (selectedId === n.id) setSelectedId(null)
                      }}
                      onDuplicate={() => duplicate(n.id)}
                      onDelete={() => setDeleteTarget(n)}
                      menuOpen={menuOpen}
                      setMenuOpen={setMenuOpen}
                    />
                  ))}
                </div>
              )}
              {otherNotes.length > 0 && (
                <div>
                  {pinnedNotes.length > 0 && (
                    <div
                      className="px-4 py-1 text-[10px] font-semibold uppercase tracking-wider"
                      style={{ color: 'var(--color-text-tertiary)' }}
                    >
                      Notes
                    </div>
                  )}
                  {otherNotes.map((n) => (
                    <NoteRow
                      key={n.id}
                      note={n}
                      isActive={selectedId === n.id}
                      onSelect={() => setSelectedId(n.id)}
                      onPin={() => pin(n.id, true)}
                      onArchive={() => {
                        archive(n.id)
                        if (selectedId === n.id) setSelectedId(null)
                      }}
                      onDuplicate={() => duplicate(n.id)}
                      onDelete={() => setDeleteTarget(n)}
                      menuOpen={menuOpen}
                      setMenuOpen={setMenuOpen}
                    />
                  ))}
                </div>
              )}
            </>
          )}
          {!loading && archivedNotes.length > 0 && (
            <div
              className="mt-2 pt-2"
              style={{ borderTop: '1px solid var(--color-border)' }}
            >
              <button
                onClick={() => setShowArchived((current) => !current)}
                className="w-full flex items-center gap-1.5 px-4 py-1 text-[10px] font-semibold uppercase tracking-wider"
                style={{ color: 'var(--color-text-tertiary)' }}
              >
                {showArchived ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
                Archived ({archivedNotes.length})
              </button>
              {showArchived &&
                archivedNotes.map((note) => (
                  <div
                    key={note.id}
                    className="flex items-center gap-2 px-4 py-2 opacity-70 hover:opacity-100"
                  >
                    <Archive size={14} style={{ color: 'var(--color-text-tertiary)' }} />
                    <span
                      className="flex-1 min-w-0 truncate text-sm"
                      style={{ color: 'var(--color-text-secondary)' }}
                    >
                      {note.title || 'Untitled note'}
                    </span>
                    <button
                      onClick={() => restore(note.id)}
                      className="p-1 rounded-md hover:bg-[var(--color-bg-tertiary)]"
                      title="Restore Note"
                      aria-label={`Restore ${note.title || 'Untitled note'}`}
                      style={{ color: 'var(--color-text-tertiary)' }}
                    >
                      <RotateCcw size={13} />
                    </button>
                    <button
                      onClick={() => setDeleteTarget(note)}
                      className="p-1 rounded-md hover:bg-[var(--color-bg-tertiary)]"
                      title="Delete permanently"
                      aria-label={`Delete ${note.title || 'Untitled note'} permanently`}
                      style={{ color: 'var(--color-danger)' }}
                    >
                      <Trash2 size={13} />
                    </button>
                  </div>
                ))}
            </div>
          )}
        </div>
      </div>

      {/* Editor panel */}
      <div className="flex-1 flex flex-col min-w-0 h-full">
        {selectedId ? (
          <NoteEditor key={selectedId} noteId={selectedId} />
        ) : (
          <div className="flex-1 flex items-center justify-center">
            <div className="text-center">
              <div
                className="w-12 h-12 rounded-xl flex items-center justify-center mx-auto mb-4"
                style={{ backgroundColor: 'var(--color-accent-muted)' }}
              >
                <FileText
                  size={22}
                  strokeWidth={1.75}
                  style={{ color: 'var(--color-accent)' }}
                />
              </div>
              <p className="text-sm" style={{ color: 'var(--color-text-tertiary)' }}>
                Select a note or create a new one
              </p>
            </div>
          </div>
        )}
      </div>

      {/* Delete confirmation */}
      {deleteTarget && (
        <ConfirmDialog
          title="Delete Note"
          message={`Permanently delete "${deleteTarget.title}"? This cannot be undone.`}
          confirmLabel="Delete"
          danger
          onConfirm={async () => {
            await remove(deleteTarget.id)
            if (selectedId === deleteTarget.id) setSelectedId(null)
            setDeleteTarget(null)
          }}
          onCancel={() => setDeleteTarget(null)}
        />
      )}
    </div>
  )
}

function NoteRow({
  note,
  isActive,
  onSelect,
  onPin,
  onArchive,
  onDuplicate,
  onDelete,
  menuOpen,
  setMenuOpen,
}: {
  note: NoteListItem
  isActive: boolean
  onSelect: () => void
  onPin: () => void
  onArchive: () => void
  onDuplicate: () => void
  onDelete: () => void
  menuOpen: string | null
  setMenuOpen: (id: string | null) => void
}) {
  const isMenuOpen = menuOpen === note.id
  return (
    <div
      className={cn(
        'group flex items-center gap-2.5 px-4 py-2 cursor-pointer transition-colors duration-75',
        isActive
          ? 'bg-[var(--color-accent-muted)]'
          : 'hover:bg-[var(--color-bg-tertiary)]',
      )}
      onClick={onSelect}
    >
      <FileText
        size={15}
        strokeWidth={1.75}
        style={{
          color: isActive ? 'var(--color-accent)' : 'var(--color-text-tertiary)',
          flexShrink: 0,
        }}
      />
      <div className="flex-1 min-w-0">
        <div
          className="text-sm font-medium truncate"
          style={{ color: 'var(--color-text-primary)' }}
        >
          {note.title || 'Untitled note'}
        </div>
        {note.excerpt && (
          <p
            className="text-xs truncate mt-0.5"
            style={{ color: 'var(--color-text-tertiary)' }}
          >
            {note.excerpt}
          </p>
        )}
      </div>
      {note.pinned && (
        <Pin
          size={10}
          fill="var(--color-warning)"
          style={{ color: 'var(--color-warning)', flexShrink: 0 }}
        />
      )}
      <div className="relative shrink-0" onClick={(e) => e.stopPropagation()}>
        <button
          onClick={() => setMenuOpen(isMenuOpen ? null : note.id)}
          className="p-1 rounded-md opacity-0 group-hover:opacity-100 transition-opacity hover:bg-[var(--color-bg-tertiary)]"
          style={{ color: 'var(--color-text-tertiary)' }}
        >
          <MoreHorizontal size={14} />
        </button>
        {isMenuOpen && (
          <>
            <div className="fixed inset-0 z-10" onClick={() => setMenuOpen(null)} />
            <div
              className="absolute right-0 top-7 z-20 w-44 py-1 rounded-lg shadow-lg border"
              style={{
                backgroundColor: 'var(--color-bg-elevated)',
                borderColor: 'var(--color-border)',
              }}
            >
              <RowMenuItem
                icon={note.pinned ? PinOff : Pin}
                label={note.pinned ? 'Unpin' : 'Pin'}
                onClick={() => {
                  setMenuOpen(null)
                  onPin()
                }}
              />
              <RowMenuItem
                icon={Archive}
                label="Archive"
                onClick={() => {
                  setMenuOpen(null)
                  onArchive()
                }}
              />
              <RowMenuItem
                icon={Copy}
                label="Duplicate"
                onClick={() => {
                  setMenuOpen(null)
                  onDuplicate()
                }}
              />
              <div style={{ borderTop: '1px solid var(--color-border)' }} />
              <RowMenuItem
                icon={Trash2}
                label="Delete"
                danger
                onClick={() => {
                  setMenuOpen(null)
                  onDelete()
                }}
              />
            </div>
          </>
        )}
      </div>
    </div>
  )
}

function RowMenuItem({
  icon: Icon,
  label,
  danger,
  onClick,
}: {
  icon: React.ComponentType<{ size?: number }>
  label: string
  danger?: boolean
  onClick: () => void
}) {
  return (
    <button
      onClick={onClick}
      className="flex items-center gap-2.5 w-full px-3 py-2 text-sm text-left transition-colors hover:bg-[var(--color-bg-tertiary)]"
      style={{ color: danger ? 'var(--color-danger)' : 'var(--color-text-primary)' }}
    >
      <Icon size={14} /> {label}
    </button>
  )
}

function NoteEditor({ noteId }: { noteId: string }) {
  const { note, loading, error, saveState, debouncedSave, forceSave, setError } =
    useNote(noteId)
  const [title, setTitle] = useState('')
  const [content, setContent] = useState('')
  const titleRef = useRef<HTMLInputElement>(null)
  const contentRef = useRef<HTMLTextAreaElement>(null)
  const initialized = useRef(false)

  // Initialize from loaded note
  useEffect(() => {
    if (note && !initialized.current) {
      setTitle(note.title === 'Untitled note' ? '' : note.title)
      setContent(note.content)
      initialized.current = true
      // Focus title if empty, else content
      setTimeout(() => {
        if (note.title === 'Untitled note') titleRef.current?.focus()
        else contentRef.current?.focus()
      }, 50)
    }
  }, [note])

  // Reset when note ID changes
  useEffect(() => {
    initialized.current = false
    setTitle('')
    setContent('')
  }, [noteId])

  // Keyboard shortcut: Ctrl+S to force save
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 's') {
        e.preventDefault()
        forceSave(title, content)
      }
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [title, content, forceSave])

  const handleTitleChange = (value: string) => {
    setTitle(value)
    debouncedSave(value, content)
  }

  const handleContentChange = (value: string) => {
    setContent(value)
    debouncedSave(title, value)
  }

  if (loading) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <p className="text-sm" style={{ color: 'var(--color-text-tertiary)' }}>
          Loading…
        </p>
      </div>
    )
  }

  if (error && !note) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <div className="text-center">
          <p className="text-sm mb-4" style={{ color: 'var(--color-danger)' }}>
            {error}
          </p>
          <button
            onClick={() => window.location.reload()}
            className="text-sm font-medium"
            style={{ color: 'var(--color-accent)' }}
          >
            Reload
          </button>
        </div>
      </div>
    )
  }

  const statusInfo = SAVE_STATUS[saveState]

  return (
    <div className="flex-1 flex flex-col min-h-0">
      {/* Editor header */}
      <div
        className="flex items-center gap-3 px-6 py-2 shrink-0"
        style={{ borderBottom: '1px solid var(--color-border)' }}
      >
        <div className="flex-1" />
        <span className="text-xs" style={{ color: statusInfo.color }}>
          {statusInfo.label}
        </span>
      </div>

      {/* Editor body */}
      <div className="flex-1 overflow-y-auto">
        <div className="max-w-[720px] mx-auto px-8 py-6">
          {/* Title */}
          <input
            ref={titleRef}
            type="text"
            value={title}
            onChange={(e) => handleTitleChange(e.target.value)}
            placeholder="Untitled note"
            className="w-full text-2xl font-semibold bg-transparent outline-none mb-4 tracking-tight"
            style={{ color: 'var(--color-text-primary)' }}
            spellCheck={false}
          />

          {/* Content */}
          <textarea
            ref={contentRef}
            value={content}
            onChange={(e) => handleContentChange(e.target.value)}
            placeholder="Start writing…"
            className="w-full flex-1 min-h-[400px] bg-transparent outline-none resize-none text-sm leading-relaxed"
            style={{ color: 'var(--color-text-primary)' }}
            spellCheck
          />
        </div>
      </div>

      {/* Error banner */}
      {error && (
        <div
          className="px-6 py-2 text-xs shrink-0 flex items-center gap-2"
          style={{
            backgroundColor: 'rgb(220 38 38 / 0.08)',
            color: 'var(--color-danger)',
          }}
        >
          <span>{error}</span>
          <button
            onClick={() => setError(null)}
            className="ml-auto font-medium"
            style={{ color: 'var(--color-danger)' }}
          >
            Dismiss
          </button>
        </div>
      )}
    </div>
  )
}
