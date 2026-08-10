import { useCallback, useEffect, useMemo, useState } from 'react'
import type { MemoryFilter, MemoryInput, MemoryItem } from '@/lib/db/types'
import * as db from '@/lib/db/tauri'

const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
const listeners = new Set<() => void>()

function notifyChanged() {
  listeners.forEach((listener) => listener())
}

export function useMemory(filter: MemoryFilter = {}) {
  const filterKey = JSON.stringify(filter)
  const stableFilter = useMemo<MemoryFilter>(() => JSON.parse(filterKey), [filterKey])
  const [items, setItems] = useState<MemoryItem[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      setItems(isTauri ? await db.listMemory(stableFilter) : [])
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Failed to load Memory')
    } finally {
      setLoading(false)
    }
  }, [stableFilter])

  useEffect(() => void load(), [load])
  useEffect(() => {
    const listener = () => void load()
    listeners.add(listener)
    return () => {
      listeners.delete(listener)
    }
  }, [load])

  const mutate = useCallback(async <T>(operation: () => Promise<T>) => {
    setError(null)
    try {
      const result = await operation()
      notifyChanged()
      return result
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Memory operation failed')
      throw cause
    }
  }, [])

  const create = useCallback(
    (input: MemoryInput) => mutate(() => db.createMemory(input)),
    [mutate],
  )
  const update = useCallback(
    (id: string, input: MemoryInput) =>
      mutate(async () => {
        const item = await db.updateMemory(id, input)
        if (!item) throw new Error('This Memory item no longer exists.')
        return item
      }),
    [mutate],
  )
  const remove = useCallback(
    (id: string) =>
      mutate(async () => {
        if (!(await db.deleteMemory(id)))
          throw new Error('This Memory item no longer exists.')
      }),
    [mutate],
  )

  return { items, loading, error, isTauri, load, create, update, remove }
}
