import { useCallback, useEffect, useState } from 'react'
import * as db from '@/lib/db/tauri'
import { safeErrorSummary } from '@/lib/integrations/presentation'
import type { Integration } from '@/lib/db/types'

const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

function errorMessage(cause: unknown) {
  const message =
    cause instanceof Error && cause.message
      ? cause.message
      : 'Could not load connections from the local workspace.'
  return (
    safeErrorSummary(message) ?? 'Could not load connections from the local workspace.'
  )
}

export function useConnections() {
  const [connections, setConnections] = useState<Integration[]>([])
  const [loading, setLoading] = useState(isTauri)
  const [error, setError] = useState<string | null>(null)
  const [updatingId, setUpdatingId] = useState<string | null>(null)

  const load = useCallback(async () => {
    if (!isTauri) {
      setConnections([])
      setLoading(false)
      return
    }
    setLoading(true)
    setError(null)
    try {
      setConnections(await db.listIntegrations())
    } catch (cause) {
      setError(errorMessage(cause))
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => void load(), [load])

  const setEnabled = useCallback(async (integration: Integration, enabled: boolean) => {
    setUpdatingId(integration.id)
    setError(null)
    try {
      const updated = await db.setIntegrationEnabled(integration.id, enabled)
      if (!updated) throw new Error('This connection is no longer available.')
      setConnections((current) =>
        current.map((item) => (item.id === updated.id ? updated : item)),
      )
    } catch (cause) {
      setError(errorMessage(cause))
    } finally {
      setUpdatingId(null)
    }
  }, [])

  const refresh = useCallback(
    async (integration: Integration) => {
      setUpdatingId(integration.id)
      setError(null)
      try {
        await db.requestIntegrationSync(integration.id)
        await load()
      } catch (cause) {
        setError(errorMessage(cause))
      } finally {
        setUpdatingId(null)
      }
    },
    [load],
  )

  const disconnectCalendar = useCallback(
    async (integration: Integration) => {
      setUpdatingId(integration.id)
      setError(null)
      try {
        if (integration.provider_id === 'my_timetable') {
          await db.disconnectMyTimetable(integration.id)
        } else {
          throw new Error('This calendar connection is not supported.')
        }
        await load()
      } catch (cause) {
        setError(errorMessage(cause))
      } finally {
        setUpdatingId(null)
      }
    },
    [load],
  )

  const removeUnsupportedCalendar = useCallback(
    async (integration: Integration) => {
      setUpdatingId(integration.id)
      setError(null)
      try {
        await db.removeUnsupportedCalendarConnection(integration.id)
        await load()
      } catch (cause) {
        setError(errorMessage(cause))
      } finally {
        setUpdatingId(null)
      }
    },
    [load],
  )

  return {
    connections,
    loading,
    error,
    updatingId,
    isTauri,
    load,
    setEnabled,
    refresh,
    disconnectCalendar,
    removeUnsupportedCalendar,
  }
}
