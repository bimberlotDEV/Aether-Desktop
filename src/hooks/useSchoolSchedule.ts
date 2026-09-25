import { useCallback, useEffect, useMemo, useState } from 'react'
import {
  getSchoolSchedule,
  setSchoolSourceAssociation,
  setSchoolSourceGroups,
} from '@/lib/db/tauri'
import type { SchoolSchedule } from '@/lib/db/types'
import { schoolScheduleRequest } from '@/lib/school/schedule'

const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

export function useSchoolSchedule(spaceId: string, now: Date) {
  const [data, setData] = useState<SchoolSchedule | null>(null)
  const [loading, setLoading] = useState(isTauri)
  const [error, setError] = useState<string | null>(null)
  const request = useMemo(() => schoolScheduleRequest(spaceId, now), [spaceId, now])

  const reload = useCallback(async () => {
    if (!isTauri) {
      setLoading(false)
      return
    }
    setLoading(true)
    setError(null)
    try {
      setData(await getSchoolSchedule(request))
    } catch (cause) {
      setError(
        cause instanceof Error && cause.message
          ? cause.message
          : 'The local school schedule could not be loaded.',
      )
    } finally {
      setLoading(false)
    }
  }, [request])

  useEffect(() => void reload(), [reload])

  const update = useCallback(
    async (mutation: () => Promise<void>) => {
      if (!isTauri) return
      setLoading(true)
      setError(null)
      try {
        await mutation()
        setData(await getSchoolSchedule(request))
      } catch (cause) {
        setError(
          cause instanceof Error && cause.message
            ? cause.message
            : 'The School source configuration could not be saved.',
        )
      } finally {
        setLoading(false)
      }
    },
    [request],
  )

  const setSourceAssociated = useCallback(
    (connectionId: string, associated: boolean) =>
      update(() => setSchoolSourceAssociation(spaceId, connectionId, associated)),
    [spaceId, update],
  )

  const selectSourceGroup = useCallback(
    (connectionId: string, selectedGroup: string | null) =>
      update(() =>
        setSchoolSourceGroups(
          spaceId,
          connectionId,
          selectedGroup ? [selectedGroup] : [],
        ),
      ),
    [spaceId, update],
  )

  return {
    data,
    loading,
    error,
    isTauri,
    reload,
    setSourceAssociated,
    selectSourceGroup,
  }
}
