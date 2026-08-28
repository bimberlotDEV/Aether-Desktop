import { useState } from 'react'
import { open, save } from '@tauri-apps/plugin-dialog'
import { Archive, CheckCircle2, RotateCcw, ShieldCheck } from 'lucide-react'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { Button, Surface } from '@/components/ui/AetherUI'
import * as db from '@/lib/db/tauri'
import type { RestorePreview } from '@/lib/db/types'

type BusyState = 'export' | 'preview' | 'restore' | null

export function BackupSettings() {
  const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  const [busy, setBusy] = useState<BusyState>(null)
  const [error, setError] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)
  const [preview, setPreview] = useState<RestorePreview | null>(null)

  async function exportBackup() {
    setError(null)
    setNotice(null)
    const date = new Date().toISOString().slice(0, 10)
    const destination = await save({
      defaultPath: `Aether-${date}.aether-backup`,
      filters: [{ name: 'Complete Aether backup', extensions: ['aether-backup'] }],
    })
    if (!destination) return

    setBusy('export')
    try {
      const result = await db.exportWorkspaceArchive(destination)
      setNotice(
        `Complete backup saved (${formatBytes(result.sizeBytes)}, ${result.managedFileCount} managed ${pluralize(result.managedFileCount, 'file')}).`,
      )
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Could not create the backup.')
    } finally {
      setBusy(null)
    }
  }

  async function chooseRestore() {
    setError(null)
    setNotice(null)
    const source = await open({
      multiple: false,
      directory: false,
      filters: [{ name: 'Complete Aether backup', extensions: ['aether-backup'] }],
    })
    if (!source) return

    setBusy('preview')
    try {
      setPreview(await db.previewWorkspaceRestore(source))
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Could not verify the backup.')
    } finally {
      setBusy(null)
    }
  }

  async function cancelRestore() {
    const token = preview?.token
    setPreview(null)
    if (token) {
      try {
        await db.cancelWorkspaceRestore(token)
      } catch {
        // Tokens expire automatically; cancellation is best-effort and never mutates data.
      }
    }
  }

  async function approveRestore() {
    if (!preview) return
    const token = preview.token
    setPreview(null)
    setError(null)
    setNotice('Preparing a recovery backup and verified restore…')
    setBusy('restore')
    try {
      await db.approveWorkspaceRestore(token)
      setNotice('Restore staged. Aether is restarting…')
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Could not prepare the restore.')
      setNotice(null)
      setBusy(null)
    }
  }

  return (
    <section>
      <h2 className="mb-4 text-sm font-semibold">Backup & restore</h2>
      <Surface className="space-y-5 p-5">
        <div className="flex items-start gap-3">
          <Archive
            size={17}
            className="mt-0.5 shrink-0 text-[var(--color-text-tertiary)]"
            aria-hidden="true"
          />
          <div className="min-w-0 flex-1">
            <p className="text-sm font-medium">Complete portable backup</p>
            <p className="mt-1 text-xs leading-5 text-[var(--color-text-tertiary)]">
              Includes your workspace database and every file managed by Aether Vault.
              API credentials and linked files outside Aether remain on this PC and are
              never copied.
            </p>
          </div>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button
            type="button"
            variant="secondary"
            icon={ShieldCheck}
            onClick={() => void exportBackup()}
            disabled={!isTauri || busy !== null}
          >
            {busy === 'export' ? 'Creating backup…' : 'Create complete backup'}
          </Button>
          <Button
            type="button"
            variant="quiet"
            icon={RotateCcw}
            onClick={() => void chooseRestore()}
            disabled={!isTauri || busy !== null}
          >
            {busy === 'preview' ? 'Verifying backup…' : 'Restore from backup'}
          </Button>
        </div>
        {!isTauri && (
          <p className="text-xs text-[var(--color-text-tertiary)]">
            Backup and restore are available in the installed Aether app.
          </p>
        )}
        {(error || notice) && (
          <p
            role={error ? 'alert' : 'status'}
            className={`flex items-center gap-2 text-sm ${error ? 'text-[var(--color-danger)]' : 'text-[var(--color-success)]'}`}
          >
            {!error && <CheckCircle2 size={15} aria-hidden="true" />}
            {error ?? notice}
          </p>
        )}
      </Surface>

      {preview && (
        <ConfirmDialog
          title="Replace this workspace from backup?"
          message={restoreMessage(preview)}
          confirmLabel="Create safety backup & restore"
          danger
          onCancel={() => void cancelRestore()}
          onConfirm={() => void approveRestore()}
        />
      )}
    </section>
  )
}

function restoreMessage(preview: RestorePreview): string {
  const { counts } = preview
  const linked = preview.linkedFileCount
    ? ` ${preview.linkedFileCount} linked ${pluralize(preview.linkedFileCount, 'file')} will remain external and may be unavailable on another PC.`
    : ''
  return `This verified Aether ${preview.appVersion} backup contains ${counts.spaces} Spaces, ${counts.notes} Notes, ${counts.tasks} Tasks, ${counts.memories} Memory items, ${counts.conversations} AI conversations, and ${preview.managedFileCount} managed Vault ${pluralize(preview.managedFileCount, 'file')}. It will replace—not merge with—the current workspace. Aether first creates a complete local safety backup, then restarts.${linked}`
}

function pluralize(count: number, noun: string): string {
  return count === 1 ? noun : `${noun}s`
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}
