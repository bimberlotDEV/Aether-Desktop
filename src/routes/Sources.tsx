import { useCallback, useEffect, useState } from 'react'
import { open } from '@tauri-apps/plugin-dialog'
import {
  Database,
  FolderPlus,
  RefreshCw,
  ShieldCheck,
  Trash2,
  FileText,
  AlertTriangle,
} from 'lucide-react'
import * as db from '@/lib/db/tauri'
import type { IndexedFile, Source, SourceScanResult } from '@/lib/db/types'
import { useSpaces } from '@/hooks/useSpaces'
import {
  Button,
  EmptyState,
  Page,
  PageHeader,
  SectionLabel,
  Surface,
} from '@/components/ui/AetherUI'

export function Sources() {
  const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  const { spaces } = useSpaces()
  const [sources, setSources] = useState<Source[]>([])
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [files, setFiles] = useState<IndexedFile[]>([])
  const [busyId, setBusyId] = useState<string | null>(null)
  const [loading, setLoading] = useState(isTauri)
  const [error, setError] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)

  const loadSources = useCallback(async () => {
    if (!isTauri) return
    try {
      setSources(await db.listSources())
    } catch (cause) {
      setError(message(cause, 'Could not load Sources.'))
    } finally {
      setLoading(false)
    }
  }, [isTauri])

  useEffect(() => {
    void loadSources()
  }, [loadSources])
  useEffect(() => {
    if (!selectedId || !isTauri) {
      setFiles([])
      return
    }
    void db
      .listIndexedFiles(selectedId)
      .then(setFiles)
      .catch((cause) => setError(message(cause, 'Could not load indexed files.')))
  }, [selectedId, isTauri])

  async function addSource() {
    setError(null)
    setNotice(null)
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Authorize a folder for Aether',
    })
    if (!selected) return
    const rootPath = selected
    const displayName =
      rootPath
        .replace(/[\\/]+$/, '')
        .split(/[\\/]/)
        .pop() || 'Local source'
    setBusyId('new')
    try {
      const source = await db.createSource({ rootPath, displayName, spaceId: null })
      setSelectedId(source.id)
      await runScan(source.id, false)
    } catch (cause) {
      setError(message(cause, 'Could not authorize this folder.'))
    } finally {
      setBusyId(null)
    }
  }

  async function runScan(id: string, manageBusy = true) {
    if (manageBusy) setBusyId(id)
    setError(null)
    setNotice(null)
    try {
      const result = await db.scanSource(id)
      setNotice(scanSummary(result))
      await loadSources()
      if (selectedId === id || !selectedId) setFiles(await db.listIndexedFiles(id))
    } catch (cause) {
      setError(message(cause, 'Could not scan this Source.'))
      await loadSources()
    } finally {
      if (manageBusy) setBusyId(null)
    }
  }

  async function associate(id: string, spaceId: string | null) {
    setError(null)
    try {
      await db.updateSourceSpace(id, spaceId)
      await loadSources()
    } catch (cause) {
      setError(message(cause, 'Could not update the Space association.'))
    }
  }

  async function revoke(source: Source) {
    if (
      !window.confirm(
        `Revoke “${source.displayName}”? Aether will delete only its local index. Files in the folder will not be changed.`,
      )
    )
      return
    setBusyId(source.id)
    setError(null)
    setNotice(null)
    try {
      await db.revokeSource(source.id)
      if (selectedId === source.id) setSelectedId(null)
      await loadSources()
      setNotice('Source access revoked. No files were changed.')
    } catch (cause) {
      setError(message(cause, 'Could not revoke this Source.'))
    } finally {
      setBusyId(null)
    }
  }

  return (
    <Page width="wide">
      <PageHeader
        eyebrow="Context engine"
        icon={Database}
        title="Sources"
        description="Authorize only the folders Aether may observe. Indexing stays local and never changes your files."
        actions={
          <Button
            variant="primary"
            icon={FolderPlus}
            onClick={() => void addSource()}
            disabled={!isTauri || busyId !== null}
          >
            Add folder
          </Button>
        }
      />
      <Surface className="mb-6 flex items-start gap-3 p-4">
        <ShieldCheck
          size={18}
          className="mt-0.5 shrink-0 text-[var(--color-success)]"
          aria-hidden="true"
        />
        <p className="text-sm leading-6 text-[var(--color-text-secondary)]">
          <strong className="text-[var(--color-text-primary)]">Private by design.</strong>{' '}
          Aether stores filenames and file metadata only. It does not read file contents,
          follow shortcuts, send Source data to AI, or reorganize anything.
        </p>
      </Surface>
      {(error || notice) && (
        <p
          role={error ? 'alert' : 'status'}
          className={`mb-5 flex items-center gap-2 text-sm ${error ? 'text-[var(--color-danger)]' : 'text-[var(--color-success)]'}`}
        >
          {error ? <AlertTriangle size={15} /> : <ShieldCheck size={15} />}
          {error ?? notice}
        </p>
      )}
      {!isTauri ? (
        <EmptyState
          icon={Database}
          eyebrow="Desktop capability"
          title="Sources are available in the installed app"
          description="Directory authorization and local indexing require Aether's protected Windows desktop boundary."
        />
      ) : loading ? (
        <p className="text-sm text-[var(--color-text-tertiary)]">Loading Sources…</p>
      ) : sources.length === 0 ? (
        <EmptyState
          icon={FolderPlus}
          eyebrow="Explicit access"
          title="Choose the first folder Aether may understand"
          description="You stay in control: add one folder, inspect what was indexed, and revoke access at any time."
          action={{ label: 'Choose a folder', onClick: () => void addSource() }}
        />
      ) : (
        <div className="grid gap-6 lg:grid-cols-[minmax(0,0.9fr)_minmax(0,1.35fr)]">
          <section>
            <SectionLabel meta={`${sources.length} authorized`}>
              Authorized folders
            </SectionLabel>
            <div className="space-y-3">
              {sources.map((source) => (
                <Surface
                  key={source.id}
                  interactive
                  className={`p-4 ${selectedId === source.id ? 'ring-1 ring-[var(--color-accent)]' : ''}`}
                >
                  <button
                    type="button"
                    className="w-full text-left focus-ring"
                    onClick={() => setSelectedId(source.id)}
                  >
                    <div className="flex items-start justify-between gap-3">
                      <div className="min-w-0">
                        <h2 className="truncate text-sm font-semibold">
                          {source.displayName}
                        </h2>
                        <p
                          className="mt-1 truncate text-xs text-[var(--color-text-tertiary)]"
                          title={source.rootPath}
                        >
                          {source.rootPath}
                        </p>
                      </div>
                      <span className="aether-badge">{source.scanStatus}</span>
                    </div>
                  </button>
                  <div className="mt-4 flex flex-wrap items-center gap-2">
                    <select
                      aria-label={`Space for ${source.displayName}`}
                      value={source.spaceId ?? ''}
                      onChange={(e) => void associate(source.id, e.target.value || null)}
                      className="aether-field min-w-0 flex-1 text-xs"
                    >
                      <option value="">No Space</option>
                      {spaces
                        .filter((space) => !space.archived_at)
                        .map((space) => (
                          <option key={space.id} value={space.id}>
                            {space.name}
                          </option>
                        ))}
                    </select>
                    <Button
                      variant="quiet"
                      icon={RefreshCw}
                      onClick={() => void runScan(source.id)}
                      disabled={busyId !== null}
                    >
                      {busyId === source.id ? 'Scanning…' : 'Rescan'}
                    </Button>
                    <Button
                      variant="quiet"
                      icon={Trash2}
                      aria-label={`Revoke ${source.displayName}`}
                      onClick={() => void revoke(source)}
                      disabled={busyId !== null}
                    />
                  </div>
                  {source.lastError && (
                    <p className="mt-3 text-xs text-[var(--color-warning)]">
                      {source.lastError}
                    </p>
                  )}
                </Surface>
              ))}
            </div>
          </section>
          <section>
            <SectionLabel
              meta={
                selectedId
                  ? `${files.length}${files.length === 2000 ? '+' : ''} current files loaded`
                  : undefined
              }
            >
              Indexed metadata
            </SectionLabel>
            {!selectedId ? (
              <Surface className="p-8 text-center text-sm text-[var(--color-text-tertiary)]">
                Select a Source to inspect its local index.
              </Surface>
            ) : files.length === 0 ? (
              <Surface className="p-8 text-center text-sm text-[var(--color-text-tertiary)]">
                No regular files indexed yet. Run a rescan to refresh this Source.
              </Surface>
            ) : (
              <Surface className="overflow-hidden">
                <ul className="divide-y divide-[var(--color-border)]">
                  {files.slice(0, 250).map((file) => (
                    <li key={file.id} className="flex items-center gap-3 px-4 py-3">
                      <FileText
                        size={15}
                        className="shrink-0 text-[var(--color-text-tertiary)]"
                      />
                      <div className="min-w-0 flex-1">
                        <p className="truncate text-sm">{file.relativePath}</p>
                        <p className="mt-0.5 text-xs text-[var(--color-text-tertiary)]">
                          {formatBytes(file.sizeBytes)}
                          {file.extension ? ` · ${file.extension.toUpperCase()}` : ''}
                        </p>
                      </div>
                    </li>
                  ))}
                </ul>
                {files.length > 250 && (
                  <p className="border-t border-[var(--color-border)] px-4 py-3 text-xs text-[var(--color-text-tertiary)]">
                    Showing the first 250 of{' '}
                    {files.length === 2000 ? 'at least 2,000' : files.length} loaded
                    files.
                  </p>
                )}
              </Surface>
            )}
          </section>
        </div>
      )}
    </Page>
  )
}

function message(cause: unknown, fallback: string) {
  return cause instanceof Error
    ? cause.message
    : typeof cause === 'string'
      ? cause
      : fallback
}
function formatBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`
}
function scanSummary(result: SourceScanResult) {
  return `Scan complete: ${result.scanned} files · ${result.added} new · ${result.changed} changed · ${result.renamed} renamed · ${result.removed} removed${result.truncated ? ' · limit reached' : ''}.`
}
