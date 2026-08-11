import { useState, useEffect, useCallback, useRef } from 'react'
import type { Note, NoteListItem, NoteSearchResult } from '@/lib/db/types'
import * as db from '@/lib/db/tauri'
import { createAutosaveCoordinator, type AutosaveCoordinator } from '@/lib/autosave'

const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
let mockNotes: Note[] = []
let mockNoteCounter = 0
const noteChangeListeners = new Set<() => void>()

function notifyNotesChanged() {
  noteChangeListeners.forEach((listener) => listener())
}

function useNoteChangeSubscription(load: () => Promise<void>) {
  useEffect(() => {
    const listener = () => void load()
    noteChangeListeners.add(listener)
    return () => {
      noteChangeListeners.delete(listener)
    }
  }, [load])
}

function createMockNote(spaceId: string): Note {
  const now = new Date().toISOString()
  mockNoteCounter += 1
  return {
    id: `mock-note-${mockNoteCounter}`,
    space_id: spaceId,
    title: 'Untitled note',
    content: '',
    content_format: 'markdown',
    excerpt: '',
    pinned: false,
    revision: 1,
    archived_at: null,
    created_at: now,
    updated_at: now,
    last_opened_at: now,
  }
}

export function useNotes(spaceId: string | undefined) {
  const [notes, setNotes] = useState<NoteListItem[]>([])
  const [archivedNotes, setArchivedNotes] = useState<NoteListItem[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(async () => {
    if (!spaceId) {
      setLoading(false)
      return
    }
    setLoading(true)
    setError(null)
    try {
      const [active, archived] = isTauri
        ? await Promise.all([db.listNotesBySpace(spaceId), db.listArchivedNotes(spaceId)])
        : [
            mockNotes.filter((note) => note.space_id === spaceId && !note.archived_at),
            mockNotes.filter((note) => note.space_id === spaceId && note.archived_at),
          ]
      setNotes(active)
      setArchivedNotes(archived)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load notes')
    } finally {
      setLoading(false)
    }
  }, [spaceId])

  useEffect(() => {
    void load()
  }, [load])
  useNoteChangeSubscription(load)

  const create = useCallback(async () => {
    if (!spaceId) return null
    try {
      const note = isTauri ? await db.createNote(spaceId) : createMockNote(spaceId)
      if (!isTauri) mockNotes = [note, ...mockNotes]
      await load()
      notifyNotesChanged()
      return note
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to create note')
      return null
    }
  }, [spaceId, load])

  const remove = useCallback(
    async (id: string) => {
      if (isTauri) await db.deleteNote(id)
      else mockNotes = mockNotes.filter((note) => note.id !== id)
      await load()
      notifyNotesChanged()
    },
    [load],
  )

  const pin = useCallback(
    async (id: string, pinned: boolean) => {
      if (isTauri) await db.pinNote(id, pinned)
      else
        mockNotes = mockNotes.map((note) =>
          note.id === id
            ? { ...note, pinned, updated_at: new Date().toISOString() }
            : note,
        )
      await load()
      notifyNotesChanged()
    },
    [load],
  )

  const archive = useCallback(
    async (id: string) => {
      if (isTauri) await db.archiveNote(id)
      else
        mockNotes = mockNotes.map((note) =>
          note.id === id
            ? { ...note, archived_at: new Date().toISOString(), pinned: false }
            : note,
        )
      await load()
      notifyNotesChanged()
    },
    [load],
  )

  const restore = useCallback(
    async (id: string) => {
      if (isTauri) await db.restoreNote(id)
      else
        mockNotes = mockNotes.map((note) =>
          note.id === id
            ? { ...note, archived_at: null, updated_at: new Date().toISOString() }
            : note,
        )
      await load()
      notifyNotesChanged()
    },
    [load],
  )

  const duplicate = useCallback(
    async (id: string) => {
      if (isTauri) await db.duplicateNote(id)
      else {
        const original = mockNotes.find((note) => note.id === id)
        if (original) {
          const duplicate = createMockNote(original.space_id)
          mockNotes = [
            {
              ...duplicate,
              title: `${original.title} (copy)`,
              content: original.content,
              excerpt: original.excerpt,
            },
            ...mockNotes,
          ]
        }
      }
      await load()
      notifyNotesChanged()
    },
    [load],
  )

  const move = useCallback(
    async (id: string, newSpaceId: string) => {
      if (isTauri) await db.moveNote(id, newSpaceId)
      else
        mockNotes = mockNotes.map((note) =>
          note.id === id
            ? { ...note, space_id: newSpaceId, updated_at: new Date().toISOString() }
            : note,
        )
      await load()
      notifyNotesChanged()
    },
    [load],
  )

  return {
    notes,
    archivedNotes,
    loading,
    error,
    load,
    create,
    remove,
    pin,
    archive,
    restore,
    duplicate,
    move,
  }
}

export function useNote(id: string | undefined) {
  const [note, setNote] = useState<Note | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const noteRef = useRef<Note | null>(null)
  const draftRef = useRef<{ title: string; content: string } | null>(null)
  const autosaveRef = useRef<AutosaveCoordinator<{
    title: string
    content: string
  }> | null>(null)
  const [saveState, setSaveState] = useState<'saved' | 'saving' | 'unsaved' | 'failed'>(
    'saved',
  )
  const [draft, setDraft] = useState<{ title: string; content: string } | null>(null)

  const load = useCallback(async () => {
    if (!id) {
      setLoading(false)
      return
    }
    setLoading(true)
    setError(null)
    try {
      const result = isTauri
        ? await db.getNote(id)
        : (mockNotes.find((note) => note.id === id) ?? null)
      setNote(result)
      noteRef.current = result
      setDraft(null)
      draftRef.current = null
      setSaveState('saved')
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load note')
    } finally {
      setLoading(false)
    }
  }, [id])

  useEffect(() => {
    load()
  }, [load])

  const save = useCallback(
    async ({ title, content }: { title: string; content: string }) => {
      const currentNote = noteRef.current
      if (!id || !currentNote) return
      setSaveState('saving')
      const excerpt = content
        .replace(/[#*`>[\]_~-]/g, '')
        .trim()
        .slice(0, 160)
      try {
        const result = isTauri
          ? await db.updateNote(id, {
              title: title || 'Untitled note',
              content,
              excerpt,
              expectedRevision: currentNote.revision,
            })
          : (() => {
              const updated: Note = {
                ...currentNote,
                title: title || 'Untitled note',
                content,
                excerpt,
                revision: currentNote.revision + 1,
                updated_at: new Date().toISOString(),
              }
              mockNotes = mockNotes.map((note) => (note.id === id ? updated : note))
              notifyNotesChanged()
              return updated
            })()
        if (result) {
          noteRef.current = result
          setNote(result)
          const currentDraft = draftRef.current
          if (currentDraft?.title === title && currentDraft.content === content) {
            draftRef.current = null
            setDraft(null)
            setSaveState('saved')
          }
        } else {
          setSaveState('failed')
          setError('Note may have been deleted')
        }
      } catch (e) {
        const msg = e instanceof Error ? e.message : 'Save failed'
        if (msg.includes('Stale update')) {
          const latest = isTauri
            ? await db.getNote(id).catch(() => null)
            : (mockNotes.find((note) => note.id === id) ?? null)
          if (latest) {
            noteRef.current = latest
            setNote(latest)
          }
          setError(
            'This Note changed elsewhere. Your draft is preserved; review it and save again.',
          )
        } else setError(msg)
        setSaveState('failed')
      }
    },
    [id],
  )

  useEffect(() => {
    const coordinator = createAutosaveCoordinator(save)
    autosaveRef.current = coordinator
    return () => {
      if (draftRef.current) coordinator.schedule(draftRef.current)
      if (autosaveRef.current === coordinator) autosaveRef.current = null
      void coordinator.flush()
    }
  }, [save])

  const debouncedSave = useCallback((title: string, content: string) => {
    const nextDraft = { title, content }
    draftRef.current = nextDraft
    setDraft(nextDraft)
    setSaveState('unsaved')
    autosaveRef.current?.schedule(nextDraft)
  }, [])

  const forceSave = useCallback((title: string, content: string) => {
    const nextDraft = { title, content }
    draftRef.current = nextDraft
    setDraft(nextDraft)
    autosaveRef.current?.schedule(nextDraft)
    return autosaveRef.current?.flush() ?? Promise.resolve()
  }, [])

  return {
    note,
    loading,
    error,
    saveState,
    draft,
    load,
    debouncedSave,
    forceSave,
    setError,
  }
}

export function useGlobalNotes() {
  const [recent, setRecent] = useState<NoteListItem[]>([])
  const [pinned, setPinned] = useState<NoteListItem[]>([])
  const [loading, setLoading] = useState(true)

  const load = useCallback(async () => {
    setLoading(true)
    try {
      const [r, p] = isTauri
        ? await Promise.all([db.listRecentNotes(undefined, 8), db.listPinnedNotes()])
        : [
            [...mockNotes]
              .filter((note) => !note.archived_at)
              .sort((a, b) => b.updated_at.localeCompare(a.updated_at))
              .slice(0, 8),
            mockNotes.filter((note) => note.pinned && !note.archived_at),
          ]
      setRecent(r)
      setPinned(p)
    } catch {
      /* silent */
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    void load()
  }, [load])
  useNoteChangeSubscription(load)

  return { recent, pinned, loading, refresh: load }
}

export function useNoteSearch() {
  const [results, setResults] = useState<NoteSearchResult[]>([])
  const [searching, setSearching] = useState(false)
  const requestRef = useRef(0)

  const search = useCallback(async (query: string, spaceId?: string) => {
    const request = ++requestRef.current
    if (!query.trim()) {
      setResults([])
      setSearching(false)
      return
    }
    setResults([])
    setSearching(true)
    try {
      const normalized = query.trim().toLocaleLowerCase()
      const r = isTauri
        ? await db.searchNotes(query, spaceId, 20)
        : mockNotes
            .filter((note) => !note.archived_at)
            .filter((note) => (spaceId ? note.space_id === spaceId : true))
            .filter((note) =>
              `${note.title} ${note.content}`.toLocaleLowerCase().includes(normalized),
            )
            .slice(0, 20)
      if (request === requestRef.current) setResults(r)
    } catch {
      /* silent */
    } finally {
      if (request === requestRef.current) setSearching(false)
    }
  }, [])

  return { results, searching, search }
}
