import { useState, useEffect, useCallback, useRef } from 'react'
import type { Note, NoteListItem, NoteSearchResult } from '@/lib/db/types'
import * as db from '@/lib/db/tauri'

const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

export function useNotes(spaceId: string | undefined) {
  const [notes, setNotes] = useState<NoteListItem[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(async () => {
    if (!spaceId || !isTauri) { setLoading(false); return }
    setLoading(true)
    setError(null)
    try {
      const result = await db.listNotesBySpace(spaceId)
      setNotes(result)
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load notes')
    } finally {
      setLoading(false)
    }
  }, [spaceId])

  useEffect(() => { load() }, [load])

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

  const remove = useCallback(async (id: string) => {
    if (!isTauri) return
    await db.deleteNote(id)
    await load()
  }, [load])

  const pin = useCallback(async (id: string, pinned: boolean) => {
    if (!isTauri) return
    await db.pinNote(id, pinned)
    await load()
  }, [load])

  const archive = useCallback(async (id: string) => {
    if (!isTauri) return
    await db.archiveNote(id)
    await load()
  }, [load])

  const duplicate = useCallback(async (id: string) => {
    if (!isTauri) return
    await db.duplicateNote(id)
    await load()
  }, [load])

  const move = useCallback(async (id: string, newSpaceId: string) => {
    if (!isTauri) return
    await db.moveNote(id, newSpaceId)
    await load()
  }, [load])

  return { notes, loading, error, load, create, remove, pin, archive, duplicate, move }
}

export function useNote(id: string | undefined) {
  const [note, setNote] = useState<Note | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const saveTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const [saveState, setSaveState] = useState<'saved' | 'saving' | 'unsaved' | 'failed'>('saved')
  const [draft, setDraft] = useState<{ title: string; content: string } | null>(null)

  const load = useCallback(async () => {
    if (!id || !isTauri) { setLoading(false); return }
    setLoading(true)
    setError(null)
    try {
      const result = await db.getNote(id)
      setNote(result)
      setDraft(null)
      setSaveState('saved')
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load note')
    } finally {
      setLoading(false)
    }
  }, [id])

  useEffect(() => { load() }, [load])

  const save = useCallback(async (title: string, content: string) => {
    if (!id || !isTauri || !note) return
    setSaveState('saving')
    const excerpt = content.replace(/[#*`>[\]_~-]/g, '').trim().slice(0, 160)
    try {
      const result = await db.updateNote(id, {
        title: title || 'Untitled note',
        content,
        excerpt,
        expectedRevision: note.revision,
      })
      if (result) {
        setNote(result)
        setDraft(null)
        setSaveState('saved')
      } else {
        setSaveState('failed')
        setError('Note may have been deleted')
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : 'Save failed'
      if (msg.includes('Stale update')) {
        setError('Modified elsewhere. Reloading...')
        await load()
      }
      setSaveState('failed')
    }
  }, [id, note, load])

  const debouncedSave = useCallback((title: string, content: string) => {
    setDraft({ title, content })
    setSaveState('unsaved')
    if (saveTimeoutRef.current) clearTimeout(saveTimeoutRef.current)
    saveTimeoutRef.current = setTimeout(() => {
      save(title, content)
    }, 800)
  }, [save])

  useEffect(() => {
    return () => {
      if (saveTimeoutRef.current) clearTimeout(saveTimeoutRef.current)
    }
  }, [])

  const forceSave = useCallback((title: string, content: string) => {
    if (saveTimeoutRef.current) {
      clearTimeout(saveTimeoutRef.current)
      saveTimeoutRef.current = null
    }
    return save(title, content)
  }, [save])

  return { note, loading, error, saveState, draft, load, debouncedSave, forceSave, setError }
}

export function useGlobalNotes() {
  const [recent, setRecent] = useState<NoteListItem[]>([])
  const [pinned, setPinned] = useState<NoteListItem[]>([])
  const [loading, setLoading] = useState(true)

  const load = useCallback(async () => {
    if (!isTauri) { setLoading(false); return }
    setLoading(true)
    try {
      const [r, p] = await Promise.all([
        db.listRecentNotes(undefined, 8),
        db.listPinnedNotes(),
      ])
      setRecent(r)
      setPinned(p)
    } catch { /* silent */ }
    finally { setLoading(false) }
  }, [])

  useEffect(() => { load() }, [load])

  return { recent, pinned, loading, refresh: load }
}

export function useNoteSearch() {
  const [results, setResults] = useState<NoteSearchResult[]>([])
  const [searching, setSearching] = useState(false)

  const search = useCallback(async (query: string, spaceId?: string) => {
    if (!isTauri || !query.trim()) { setResults([]); return }
    setSearching(true)
    try {
      const r = await db.searchNotes(query, spaceId, 20)
      setResults(r)
    } catch { /* silent */ }
    finally { setSearching(false) }
  }, [])

  return { results, searching, search }
}
