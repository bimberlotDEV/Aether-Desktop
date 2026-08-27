import { useCallback, useEffect, useState } from 'react'
import * as db from '@/lib/db/tauri'
import type {
  ActionPreview,
  ActionRequest,
  ActionResult,
  Source,
  Space,
} from '@/lib/db/types'

const desktopAvailable = () =>
  typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

export function useActions() {
  const isDesktop = desktopAvailable()
  const [spaces, setSpaces] = useState<Space[]>([])
  const [sources, setSources] = useState<Source[]>([])
  const [preview, setPreview] = useState<ActionPreview | null>(null)
  const [result, setResult] = useState<ActionResult | null>(null)
  const [loading, setLoading] = useState(isDesktop)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let active = true
    if (!isDesktop) return
    Promise.all([db.listSpaces(false), db.listSources()])
      .then(([nextSpaces, nextSources]) => {
        if (!active) return
        setSpaces(nextSpaces.filter((space) => !space.archived_at))
        setSources(nextSources)
      })
      .catch((cause) => {
        if (active) setError(message(cause, 'Could not load Action options.'))
      })
      .finally(() => {
        if (active) setLoading(false)
      })
    return () => {
      active = false
    }
  }, [isDesktop])

  const propose = useCallback(async (request: ActionRequest) => {
    setBusy(true)
    setError(null)
    setResult(null)
    try {
      setPreview(await db.previewAction(request))
    } catch (cause) {
      setError(message(cause, 'Aether could not validate this Action.'))
    } finally {
      setBusy(false)
    }
  }, [])

  const approve = useCallback(async () => {
    if (!preview) return
    setBusy(true)
    setError(null)
    try {
      const completed = await db.executeAction(preview.token)
      setResult(completed)
      setPreview(null)
    } catch (cause) {
      setPreview(null)
      setError(message(cause, 'The approved Action could not be completed.'))
    } finally {
      setBusy(false)
    }
  }, [preview])

  const cancel = useCallback(async () => {
    const token = preview?.token
    setPreview(null)
    if (!token) return
    try {
      await db.cancelAction(token)
    } catch {
      // A missing/expired token is already safely non-executable.
    }
  }, [preview])

  const reset = useCallback(() => {
    setPreview(null)
    setResult(null)
    setError(null)
  }, [])

  return {
    isDesktop,
    spaces,
    sources,
    preview,
    result,
    loading,
    busy,
    error,
    propose,
    approve,
    cancel,
    reset,
  }
}

function message(cause: unknown, fallback: string) {
  return cause instanceof Error && cause.message ? cause.message : fallback
}
