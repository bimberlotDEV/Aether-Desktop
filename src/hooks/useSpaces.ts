import { useState, useEffect, useCallback } from 'react'
import type { Space, SpaceWithDetails } from '@/lib/db/types'
import * as db from '@/lib/db/tauri'

// Detect if we're running in Tauri
const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

// Mock data for browser dev mode
let mockSpaces: Space[] = []
let mockIdCounter = 0

function mockId(): string {
  mockIdCounter++
  return `mock-${mockIdCounter}-${Date.now()}`
}

export function useSpaces() {
  const [spaces, setSpaces] = useState<Space[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      if (isTauri) {
        const result = await db.listTopLevelSpaces()
        setSpaces(result)
      } else {
        setSpaces([...mockSpaces])
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load spaces')
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => { load() }, [load])

  const create = useCallback(async (params: {
    name: string; description?: string; icon?: string; accent?: string;
    templateType?: string; moduleTypes: string[];
    parentSpaceId?: string;
    subjects?: { name: string; icon?: string; accent?: string }[];
  }) => {
    if (isTauri) {
      if (params.templateType === 'school' && params.subjects?.length) {
        await db.createSchoolSpace({
          name: params.name, description: params.description,
          icon: params.icon, accent: params.accent,
          moduleTypes: params.moduleTypes,
          subjects: params.subjects,
        })
      } else {
        await db.createSpaceWithModules({
          name: params.name, description: params.description,
          icon: params.icon, accent: params.accent,
          templateType: params.templateType,
          moduleTypes: params.moduleTypes,
          parentSpaceId: params.parentSpaceId,
        })
      }
    } else {
      const newSpace = {
        id: mockId(), name: params.name,
        description: params.description ?? null,
        icon: params.icon ?? null, accent: params.accent ?? null,
        template_type: params.templateType ?? 'blank',
        favourite: false, archived_at: null, sort_order: mockSpaces.length,
        settings_json: null, parent_space_id: params.parentSpaceId ?? null,
        last_opened_at: null,
        created_at: new Date().toISOString(), updated_at: new Date().toISOString(),
      } as Space
      mockSpaces = [...mockSpaces, newSpace]
    }
    await load()
  }, [load])

  const archive = useCallback(async (id: string) => {
    if (isTauri) await db.archiveSpace(id)
    else {
      mockSpaces = mockSpaces.map(s => s.id === id ? { ...s, archived_at: new Date().toISOString() } : s)
    }
    await load()
  }, [load])

  const restore = useCallback(async (id: string) => {
    if (isTauri) await db.restoreSpace(id)
    else {
      mockSpaces = mockSpaces.map(s => s.id === id ? { ...s, archived_at: null } : s)
    }
    await load()
  }, [load])

  const remove = useCallback(async (id: string) => {
    if (isTauri) await db.deleteSpace(id)
    else { mockSpaces = mockSpaces.filter(s => s.id !== id) }
    await load()
  }, [load])

  const toggleFavourite = useCallback(async (id: string, fav: boolean) => {
    if (isTauri) await db.favouriteSpace(id, fav)
    else {
      mockSpaces = mockSpaces.map(s => s.id === id ? { ...s, favourite: fav } : s)
    }
    await load()
  }, [load])

  const duplicate = useCallback(async (id: string) => {
    if (isTauri) { await db.duplicateSpace(id) }
    else {
      const orig = mockSpaces.find(s => s.id === id)
      if (orig) {
        mockSpaces = [...mockSpaces, { ...orig, id: mockId(), name: `${orig.name} (copy)`, created_at: new Date().toISOString(), updated_at: new Date().toISOString() } as Space]
      }
    }
    await load()
  }, [load])

  const reorder = useCallback(async (ids: string[]) => {
    if (isTauri) await db.reorderSpaces(ids)
    else {
      const map = new Map(mockSpaces.map(s => [s.id, s]))
      mockSpaces = ids.map((id, i) => ({ ...map.get(id)!, sort_order: i }))
    }
    await load()
  }, [load])

  return {
    spaces, loading, error, load,
    create, archive, restore, remove, toggleFavourite, duplicate, reorder,
    isTauri,
  }
}

export function useSpaceDetail(spaceId: string | undefined) {
  const [data, setData] = useState<SpaceWithDetails | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (!spaceId) { setLoading(false); return }
    setLoading(true)
    if (isTauri) {
      db.getSpace(spaceId).then(setData).catch(e => setError(String(e))).finally(() => setLoading(false))
    } else {
      const space = mockSpaces.find(s => s.id === spaceId)
      if (space) {
        setData({ space, modules: [], children: mockSpaces.filter(s => s.parent_space_id === spaceId) })
      } else {
        setError('Space not found')
      }
      setLoading(false)
    }
  }, [spaceId])

  return { data, loading, error }
}

export function useArchivedSpaces() {
  const [spaces, setSpaces] = useState<Space[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    if (isTauri) {
      db.listSpaces(true).then(all => setSpaces(all.filter(s => s.archived_at))).finally(() => setLoading(false))
    } else {
      setSpaces(mockSpaces.filter(s => s.archived_at))
      setLoading(false)
    }
  }, [])

  return { spaces, loading }
}

// Allow external refresh
export function useRefresh() {
  const [, setTick] = useState(0)
  return () => setTick(t => t + 1)
}
