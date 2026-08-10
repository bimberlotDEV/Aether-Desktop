import { useCallback, useEffect, useMemo, useState } from 'react'
import type {
  Space,
  VaultFilter,
  VaultImportInput,
  VaultItem,
  VaultUpdateInput,
} from '@/lib/db/types'
import * as db from '@/lib/db/tauri'

const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
const vaultChangeListeners = new Set<() => void>()

function notifyVaultChanged() {
  vaultChangeListeners.forEach((listener) => listener())
}

function useVaultSubscription(load: () => Promise<void>) {
  useEffect(() => {
    const listener = () => void load()
    vaultChangeListeners.add(listener)
    return () => {
      vaultChangeListeners.delete(listener)
    }
  }, [load])
}

export function useVault(filter: VaultFilter = {}) {
  const filterKey = JSON.stringify(filter)
  const stableFilter = useMemo<VaultFilter>(() => JSON.parse(filterKey), [filterKey])
  const [items, setItems] = useState<VaultItem[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      setItems(isTauri ? await db.listVaultItems(stableFilter) : [])
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Failed to load Vault items')
    } finally {
      setLoading(false)
    }
  }, [stableFilter])

  useEffect(() => void load(), [load])
  useVaultSubscription(load)

  const mutate = useCallback(async <T>(operation: () => Promise<T>): Promise<T> => {
    setError(null)
    try {
      const result = await operation()
      notifyVaultChanged()
      return result
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Vault operation failed')
      throw cause
    }
  }, [])

  const importItem = useCallback(
    (input: VaultImportInput) => {
      if (!isTauri)
        return Promise.reject(new Error('File import requires the desktop app.'))
      return mutate(() => db.importVaultItem(input))
    },
    [mutate],
  )

  const update = useCallback(
    (id: string, input: VaultUpdateInput) =>
      mutate(async () => {
        const item = await db.updateVaultItem(id, input)
        if (!item) throw new Error('This Vault item no longer exists.')
        return item
      }),
    [mutate],
  )

  const remove = useCallback(
    (id: string) =>
      mutate(async () => {
        const removed = await db.removeVaultItem(id)
        if (!removed) throw new Error('This Vault item no longer exists.')
        return removed
      }),
    [mutate],
  )

  const openItem = useCallback((id: string) => db.openVaultItem(id), [])
  const reveal = useCallback((id: string) => db.revealVaultItem(id), [])

  return {
    items,
    loading,
    error,
    load,
    importItem,
    update,
    remove,
    openItem,
    reveal,
    isTauri,
  }
}

export function useVaultSpaces() {
  const [spaces, setSpaces] = useState<Space[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    let active = true
    async function load() {
      try {
        const result = isTauri ? await db.listSpaces(false) : []
        if (active) setSpaces(result.filter((space) => !space.archived_at))
      } finally {
        if (active) setLoading(false)
      }
    }
    void load()
    return () => {
      active = false
    }
  }, [])

  return { spaces, loading }
}
