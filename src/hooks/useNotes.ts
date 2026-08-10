import { useState, useEffect, useCallback, useRef } from 'react'
import type { Note, NoteListItem, NoteSearchResult } from '@/lib/db/types'
import * as db from '@/lib/db/tauri'
import { createAutosaveCoordinator, type AutosaveCoordinator } from '@/lib/autosave'

const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

export function useNotes(spaceId: string | undefined) {
  const [notes, setNotes] = useState<NoteListItem[]>([])
  const [archivedNotes, setArchivedNotes] = useState<NoteListItem[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(async () => {
    if (!spaceId || !isTauri) {
      setLoading(false)
      return
    }
    setLoading(true)
    setError(null)
    try {
      const [active, archived] = await Promise.all([
        db.listNotesBySpace(spaceId),
        db.listArchivedNotes(spaceId),
      ])
      setNotes(active)
      setArchivedNotes(archived)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load notes')
    } finally {
      setLoading(false)
    }
  }, [spaceId])

  useEffect(() => {
    load()
  }, [load])

  const create = useCallback(async () => {
    if (!spaceId || !isTauri) return null
    try {
      const note = await db.createNote(spaceId)
      await load()
      return note
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to create note')
      return null
    }
  }, [spaceId, load])

  const remove = useCallback(
    async (id: string) => {
      if (!isTauri) return
      await db.deleteNote(id)
      await load()
    },
    [load],
  )

  const pin = useCallback(
    async (id: string, pinned: boolean) => {
      if (!isTauri) return
      await db.pinNote(id, pinned)
      await load()
    },
    [load],
  )

  const archive = useCallback(
    async (id: string) => {
      if (!isTauri) return
      await db.archiveNote(id)
      await load()
    },
    [load],
  )

  const restore = useCallback(
    async (id: string) => {
      if (!isTauri) return
      await db.restoreNote(id)
      await load()
    },
    [load],
  )

  const duplicate = useCallback(
    async (id: string) => {
      if (!isTauri) return
      await db.duplicateNote(id)
      await load()
    },
    [load],
  )

  const move = useCallback(
    async (id: string, newSpaceId: string) => {
      if (!isTauri) return
      await db.moveNote(id, newSpaceId)
      await load()
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
    if (!id || !isTauri) {
      setLoading(false)
      return
    }
    setLoading(true)
    setError(null)
    try {
      const result = await db.getNote(id)
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
      if (!id || !isTauri || !currentNote) return
      setSaveState('saving')
      const excerpt = content
        .replace(/[#*`>[\]_~-]/g, '')
        .trim()
        .slice(0, 160)
      try {
        const result = await db.updateNote(id, {
          title: title || 'Untitled note',
          content,
          excerpt,
          expectedRevision: currentNote.revision,
        })
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
          const latest = await db.getNote(id).catch(() => null)
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
    if (!isTauri) {
      setLoading(false)
      return
    }
    setLoading(true)
    try {
      const [r, p] = await Promise.all([
        db.listRecentNotes(undefined, 8),
        db.listPinnedNotes(),
      ])
      setRecent(r)
      setPinned(p)
    } catch {
      /* silent */
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    load()
  }, [load])

  return { recent, pinned, loading, refresh: load }
}

export function useNoteSearch() {
  const [results, setResults] = useState<NoteSearchResult[]>([])
  const [searching, setSearching] = useState(false)
  const requestRef = useRef(0)

  const search = useCallback(async (query: string, spaceId?: string) => {
    const request = ++requestRef.current
    if (!isTauri || !query.trim()) {
      setResults([])
      setSearching(false)
      return
    }
    setResults([])
    setSearching(true)
    try {
      const r = await db.searchNotes(query, spaceId, 20)
      if (request === requestRef.current) setResults(r)
    } catch {
      /* silent */
    } finally {
      if (request === requestRef.current) setSearching(false)
    }
  }, [])

  return { results, searching, search }
}
