import { useCallback, useEffect, useState } from 'react'
import * as db from '@/lib/db/tauri'
import type { ActivityItem, SpaceContinuity } from '@/lib/db/types'

const desktopAvailable = () =>
  typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

export function useActivityFeed(spaceId?: string) {
  const isDesktop = desktopAvailable()
  const [items, setItems] = useState<ActivityItem[]>([])
  const [loading, setLoading] = useState(isDesktop)
  const [error, setError] = useState<string | null>(null)

  const reload = useCallback(async () => {
    if (!isDesktop) return
    setLoading(true)
    setError(null)
    try {
      setItems(await db.listActivity({ spaceId, limit: 50 }))
    } catch (cause) {
      setError(message(cause, 'Could not load Activity.'))
    } finally {
      setLoading(false)
    }
  }, [isDesktop, spaceId])

  useEffect(() => {
    void reload()
  }, [reload])

  return { items, loading, error, isDesktop, reload }
}

export function useSpaceContinuity(spaceId: string) {
  const isDesktop = desktopAvailable()
  const [data, setData] = useState<SpaceContinuity | null>(null)
  const [loading, setLoading] = useState(isDesktop)
  const [error, setError] = useState<string | null>(null)

  const reload = useCallback(async () => {
    if (!isDesktop || !spaceId) return
    setLoading(true)
    setError(null)
    try {
      setData(await db.getSpaceContinuity(spaceId))
    } catch (cause) {
      setError(message(cause, 'Could not build this Space overview.'))
    } finally {
      setLoading(false)
    }
  }, [isDesktop, spaceId])

  useEffect(() => {
    let active = true
    if (!isDesktop || !spaceId) return
    setLoading(true)
    setError(null)
    void db
      .getSpaceContinuity(spaceId)
      .then((result) => {
        if (active) setData(result)
      })
      .catch((cause) => {
        if (active) setError(message(cause, 'Could not build this Space overview.'))
      })
      .finally(() => {
        if (active) setLoading(false)
      })
    return () => {
      active = false
    }
  }, [isDesktop, spaceId])

  return { data, loading, error, isDesktop, reload }
}

function message(cause: unknown, fallback: string) {
  return cause instanceof Error && cause.message ? cause.message : fallback
}
