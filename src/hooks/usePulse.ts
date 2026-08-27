import { useCallback, useEffect, useState } from 'react'
import * as db from '@/lib/db/tauri'
import type { PulseSnapshot } from '@/lib/db/types'

const desktopAvailable = () =>
  typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

export function usePulse() {
  const isDesktop = desktopAvailable()
  const [data, setData] = useState<PulseSnapshot | null>(null)
  const [loading, setLoading] = useState(isDesktop)
  const [error, setError] = useState<string | null>(null)

  const reload = useCallback(async () => {
    if (!isDesktop) return
    setLoading(true)
    setError(null)
    try {
      setData(await db.getPulse())
    } catch (cause) {
      setError(
        cause instanceof Error && cause.message ? cause.message : 'Could not load Pulse.',
      )
    } finally {
      setLoading(false)
    }
  }, [isDesktop])

  useEffect(() => {
    let active = true
    if (!isDesktop) return
    void db
      .getPulse()
      .then((snapshot) => {
        if (active) setData(snapshot)
      })
      .catch((cause) => {
        if (active)
          setError(
            cause instanceof Error && cause.message
              ? cause.message
              : 'Could not load Pulse.',
          )
      })
      .finally(() => {
        if (active) setLoading(false)
      })
    return () => {
      active = false
    }
  }, [isDesktop])

  return { data, loading, error, isDesktop, reload }
}
