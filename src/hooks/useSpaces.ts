import { useState, useEffect, useCallback } from 'react'
import type { ModuleInstance, Space, SpaceWithDetails } from '@/lib/db/types'
import * as db from '@/lib/db/tauri'

const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

let mockSpaces: Space[] = []
let mockModulesBySpace = new Map<string, ModuleInstance[]>()
let mockIdCounter = 0
const spaceChangeListeners = new Set<() => void>()

function mockId(): string {
  mockIdCounter++
  return `mock-${mockIdCounter}-${Date.now()}`
}

function mockModules(spaceId: string, moduleTypes: string[]): ModuleInstance[] {
  const now = new Date().toISOString()
  return moduleTypes.map((moduleType) => ({
    id: mockId(),
    space_id: spaceId,
    module_type: moduleType,
    title: null,
    config_json: null,
    layout_json: null,
    created_at: now,
    updated_at: now,
  }))
}

function notifySpacesChanged() {
  spaceChangeListeners.forEach((listener) => listener())
}

function useSpaceChangeSubscription(load: () => Promise<void>) {
  useEffect(() => {
    const listener = () => {
      void load()
    }
    spaceChangeListeners.add(listener)
    return () => {
      spaceChangeListeners.delete(listener)
    }
  }, [load])
}

export function useSpaces() {
  const [spaces, setSpaces] = useState<Space[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      setSpaces(isTauri ? await db.listTopLevelSpaces() : [...mockSpaces])
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load spaces')
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    void load()
  }, [load])
  useSpaceChangeSubscription(load)

  const create = useCallback(
    async (params: {
      name: string
      description?: string
      icon?: string
      accent?: string
      templateType?: string
      moduleTypes: string[]
      parentSpaceId?: string
      subjects?: { name: string; icon?: string; accent?: string }[]
    }) => {
      if (isTauri) {
        if (params.templateType === 'school' && params.subjects?.length) {
          await db.createSchoolSpace({
            name: params.name,
            description: params.description,
            icon: params.icon,
            accent: params.accent,
            moduleTypes: params.moduleTypes,
            subjects: params.subjects,
          })
        } else {
          await db.createSpaceWithModules({
            name: params.name,
            description: params.description,
            icon: params.icon,
            accent: params.accent,
            templateType: params.templateType,
            moduleTypes: params.moduleTypes,
            parentSpaceId: params.parentSpaceId,
          })
        }
      } else {
        const now = new Date().toISOString()
        const newSpace = {
          id: mockId(),
          name: params.name,
          description: params.description ?? null,
          icon: params.icon ?? null,
          accent: params.accent ?? null,
          template_type: params.templateType ?? 'blank',
          favourite: false,
          archived_at: null,
          sort_order: mockSpaces.length,
          settings_json: null,
          parent_space_id: params.parentSpaceId ?? null,
          last_opened_at: null,
          created_at: now,
          updated_at: now,
        } as Space
        mockSpaces = [...mockSpaces, newSpace]
        mockModulesBySpace.set(newSpace.id, mockModules(newSpace.id, params.moduleTypes))

        if (params.templateType === 'school' && params.subjects?.length) {
          const subjects = params.subjects.map((subject) => {
            const childId = mockId()
            mockModulesBySpace.set(childId, mockModules(childId, params.moduleTypes))
            return {
              ...newSpace,
              id: childId,
              name: subject.name,
              icon: subject.icon ?? null,
              accent: subject.accent ?? null,
              template_type: 'subject',
              parent_space_id: newSpace.id,
              sort_order: mockSpaces.length,
            }
          })
          mockSpaces = [...mockSpaces, ...subjects]
        }
      }
      notifySpacesChanged()
    },
    [],
  )

  const update = useCallback(
    async (
      id: string,
      params: {
        name: string
        description: string | null
        icon: string | null
        accent: string | null
        moduleTypes: string[]
      },
    ) => {
      if (isTauri) {
        await db.updateSpace(id, params)
        await db.setSpaceModules(id, params.moduleTypes)
      } else {
        mockSpaces = mockSpaces.map((space) =>
          space.id === id
            ? {
                ...space,
                name: params.name,
                description: params.description,
                icon: params.icon,
                accent: params.accent,
                updated_at: new Date().toISOString(),
              }
            : space,
        )
        mockModulesBySpace.set(id, mockModules(id, params.moduleTypes))
      }
      notifySpacesChanged()
    },
    [],
  )

  const archive = useCallback(async (id: string) => {
    if (isTauri) await db.archiveSpace(id)
    else
      mockSpaces = mockSpaces.map((space) =>
        space.id === id ? { ...space, archived_at: new Date().toISOString() } : space,
      )
    notifySpacesChanged()
  }, [])

  const restore = useCallback(async (id: string) => {
    if (isTauri) await db.restoreSpace(id)
    else
      mockSpaces = mockSpaces.map((space) =>
        space.id === id ? { ...space, archived_at: null } : space,
      )
    notifySpacesChanged()
  }, [])

  const remove = useCallback(async (id: string) => {
    if (isTauri) await db.deleteSpace(id)
    else {
      const removedIds = new Set([
        id,
        ...mockSpaces
          .filter((space) => space.parent_space_id === id)
          .map((space) => space.id),
      ])
      mockSpaces = mockSpaces.filter((space) => !removedIds.has(space.id))
      removedIds.forEach((removedId) => mockModulesBySpace.delete(removedId))
    }
    notifySpacesChanged()
  }, [])

  const toggleFavourite = useCallback(async (id: string, favourite: boolean) => {
    if (isTauri) await db.favouriteSpace(id, favourite)
    else
      mockSpaces = mockSpaces.map((space) =>
        space.id === id ? { ...space, favourite } : space,
      )
    notifySpacesChanged()
  }, [])

  const duplicate = useCallback(async (id: string) => {
    if (isTauri) {
      await db.duplicateSpace(id)
    } else {
      const original = mockSpaces.find((space) => space.id === id)
      if (original) {
        const duplicateId = mockId()
        mockSpaces = [
          ...mockSpaces,
          {
            ...original,
            id: duplicateId,
            name: `${original.name} (copy)`,
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
          },
        ]
        mockModulesBySpace.set(
          duplicateId,
          mockModules(
            duplicateId,
            (mockModulesBySpace.get(original.id) ?? []).map(
              (module) => module.module_type,
            ),
          ),
        )
      }
    }
    notifySpacesChanged()
  }, [])

  const reorder = useCallback(async (ids: string[]) => {
    if (isTauri) await db.reorderSpaces(ids)
    else {
      const byId = new Map(mockSpaces.map((space) => [space.id, space]))
      const reordered = ids.flatMap((id, index) => {
        const space = byId.get(id)
        return space ? [{ ...space, sort_order: index }] : []
      })
      const untouched = mockSpaces.filter((space) => !ids.includes(space.id))
      mockSpaces = [...reordered, ...untouched]
    }
    notifySpacesChanged()
  }, [])

  return {
    spaces,
    loading,
    error,
    load,
    create,
    update,
    archive,
    restore,
    remove,
    toggleFavourite,
    duplicate,
    reorder,
    isTauri,
  }
}

export function useSpaceDetail(spaceId: string | undefined) {
  const [data, setData] = useState<SpaceWithDetails | null>(null)
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
      if (isTauri) {
        setData(await db.getSpace(spaceId))
      } else {
        const space = mockSpaces.find((candidate) => candidate.id === spaceId)
        if (space) {
          setData({
            space,
            modules: mockModulesBySpace.get(space.id) ?? [],
            children: mockSpaces.filter(
              (candidate) => candidate.parent_space_id === spaceId,
            ),
          })
        } else {
          setData(null)
          setError('Space not found')
        }
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load Space')
    } finally {
      setLoading(false)
    }
  }, [spaceId])

  useEffect(() => {
    void load()
  }, [load])
  useSpaceChangeSubscription(load)

  return { data, loading, error, load }
}

export function useArchivedSpaces() {
  const [spaces, setSpaces] = useState<Space[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      if (isTauri) {
        const allSpaces = await db.listSpaces(true)
        setSpaces(allSpaces.filter((space) => space.archived_at))
      } else {
        setSpaces(mockSpaces.filter((space) => space.archived_at))
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load archived Spaces')
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    void load()
  }, [load])
  useSpaceChangeSubscription(load)

  return { spaces, loading, error, load }
}
