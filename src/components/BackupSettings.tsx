import { useState } from 'react'
import { save } from '@tauri-apps/plugin-dialog'
import { Archive, CheckCircle2 } from 'lucide-react'
import * as db from '@/lib/db/tauri'

export function BackupSettings() {
  const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)

  async function exportBackup() {
    setError(null)
    setNotice(null)
    const date = new Date().toISOString().slice(0, 10)
    const destination = await save({
      defaultPath: `Aether-${date}.aether-backup.db`,
      filters: [{ name: 'Aether workspace backup', extensions: ['db'] }],
    })
    if (!destination) return

    setBusy(true)
    try {
      const result = await db.exportWorkspaceBackup(destination)
      setNotice(`Backup saved (${formatBytes(result.sizeBytes)}).`)
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Could not export the backup.')
    } finally {
      setBusy(false)
    }
  }

  return (
    <section>
      <h2 className="mb-4 text-sm font-semibold">Backup</h2>
      <div className="space-y-4 rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-5">
        <div className="flex items-start gap-3">
          <Archive size={17} className="mt-0.5 text-[var(--color-text-tertiary)]" />
          <div className="min-w-0 flex-1">
            <p className="text-sm font-medium">Export workspace database</p>
            <p className="mt-1 text-xs leading-5 text-[var(--color-text-tertiary)]">
              Includes Spaces, Notes, Tasks, Memory, conversations, and Vault metadata.
              API credentials and the contents of managed or linked Vault files are not
              included.
            </p>
          </div>
        </div>
        <button
          type="button"
          onClick={() => void exportBackup()}
          disabled={!isTauri || busy}
          className="rounded-md border border-[var(--color-border)] px-3 py-2 text-sm font-medium disabled:opacity-50 focus-ring"
        >
          {busy ? 'Exporting…' : 'Choose location and export'}
        </button>
        {!isTauri && (
          <p className="text-xs text-[var(--color-text-tertiary)]">
            Backup export is available in the installed Aether app.
          </p>
        )}
        {(error || notice) && (
          <p
            role={error ? 'alert' : 'status'}
            className={`flex items-center gap-2 text-sm ${error ? 'text-[var(--color-danger)]' : 'text-[var(--color-success)]'}`}
          >
            {!error && <CheckCircle2 size={15} />}
            {error ?? notice}
          </p>
        )}
      </div>
    </section>
  )
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}
