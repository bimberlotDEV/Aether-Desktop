import { useEffect, useState } from 'react'
import {
  Bell,
  CheckCircle2,
  Download,
  Keyboard,
  PanelTopClose,
  RefreshCw,
  ShieldCheck,
  X,
} from 'lucide-react'
import { Button, Surface } from '@/components/ui/AetherUI'
import type {
  NativeStatus,
  UpdatePreview,
  UpdateProgressEvent,
  UpdateStatus,
} from '@/lib/db/types'
import * as native from '@/lib/db/tauri'

type BusyState = 'checking' | 'installing' | null

type DownloadProgress = {
  downloaded: number
  total: number | null
  stage: 'downloading' | 'verifying' | 'installing'
}

export function NativeSettings() {
  const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  const [status, setStatus] = useState<NativeStatus | null>(null)
  const [updateStatus, setUpdateStatus] = useState<UpdateStatus | null>(null)
  const [preview, setPreview] = useState<UpdatePreview | null>(null)
  const [busy, setBusy] = useState<BusyState>(null)
  const [progress, setProgress] = useState<DownloadProgress | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)

  useEffect(() => {
    if (!isTauri) return
    Promise.all([native.getNativeStatus(), native.getUpdateStatus()])
      .then(([desktop, updates]) => {
        setStatus(desktop)
        setUpdateStatus(updates)
      })
      .catch((cause) => {
        setError(
          cause instanceof Error ? cause.message : 'Could not load desktop status.',
        )
      })
  }, [isTauri])

  async function testNotification() {
    setError(null)
    setNotice(null)
    try {
      await native.sendTestNotification()
      setNotice('Test notification sent.')
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Could not send notification.')
    }
  }

  async function checkForUpdate() {
    setError(null)
    setNotice(null)
    setPreview(null)
    setBusy('checking')
    try {
      const available = await native.checkForUpdate()
      if (available) {
        setPreview(available)
        setNotice(`Aether ${available.version} is ready for your review.`)
      } else {
        setNotice('You already have the newest Stable version of Aether.')
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Could not check for updates.')
    } finally {
      setBusy(null)
    }
  }

  async function dismissUpdate() {
    const token = preview?.token
    setPreview(null)
    setNotice(null)
    if (!token) return
    try {
      await native.cancelUpdate(token)
    } catch {
      // Pending tokens expire automatically; cancellation never mutates app data.
    }
  }

  async function installUpdate() {
    if (!preview) return
    const token = preview.token
    setBusy('installing')
    setError(null)
    setNotice('Downloading the signed update…')
    setProgress({ downloaded: 0, total: null, stage: 'downloading' })
    try {
      await native.installUpdate(token, handleProgress)
      setNotice('Update verified and handed to Windows. Aether will restart.')
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Could not install the update.')
      setNotice(null)
      setProgress(null)
      setPreview(null)
      setBusy(null)
    }
  }

  function handleProgress(event: UpdateProgressEvent) {
    switch (event.event) {
      case 'started':
        setProgress({
          downloaded: 0,
          total: event.data.contentLength,
          stage: 'downloading',
        })
        break
      case 'progress':
        setProgress((current) => ({
          downloaded: event.data.downloaded,
          total: current?.total ?? null,
          stage: 'downloading',
        }))
        break
      case 'downloaded':
        setProgress((current) => ({
          downloaded: current?.downloaded ?? 0,
          total: current?.total ?? null,
          stage: 'verifying',
        }))
        setNotice('Download complete. Verifying its release signature…')
        break
      case 'verified':
        setNotice('Signature verified. Preparing the Windows installer…')
        break
      case 'installing':
        setProgress((current) => ({
          downloaded: current?.downloaded ?? 0,
          total: current?.total ?? null,
          stage: 'installing',
        }))
        setNotice('Windows is installing the update. Aether will restart automatically.')
        break
    }
  }

  return (
    <section>
      <h2 className="mb-4 text-sm font-semibold">Windows desktop</h2>
      <Surface className="space-y-4 p-5">
        {!isTauri ? (
          <p className="text-sm text-[var(--color-text-secondary)]">
            Native desktop features are available in the installed Aether app.
          </p>
        ) : !status || !updateStatus ? (
          <p className="text-sm text-[var(--color-text-tertiary)]" role="status">
            Checking desktop integration…
          </p>
        ) : (
          <>
            <StatusRow
              icon={PanelTopClose}
              title="System tray"
              detail="Closing the window keeps Aether available. Use Quit Aether in the tray to exit."
              ready={status.trayAvailable}
            />
            <StatusRow
              icon={Keyboard}
              title="Global shortcut"
              detail={`${status.shortcut} shows and focuses Aether${status.shortcutRegistered ? '.' : ' — unavailable because another app may be using it.'}`}
              ready={status.shortcutRegistered}
            />
            <div className="flex items-center gap-3">
              <Bell
                size={16}
                className="text-[var(--color-text-tertiary)]"
                aria-hidden="true"
              />
              <div className="min-w-0 flex-1">
                <p className="text-sm font-medium">Native notifications</p>
                <p className="text-xs text-[var(--color-text-tertiary)]">
                  Windows controls notification visibility and focus-assist behavior.
                </p>
              </div>
              <Button
                variant="quiet"
                onClick={() => void testNotification()}
                disabled={!status.notificationsAvailable || busy !== null}
              >
                Send test
              </Button>
            </div>

            <div className="border-t border-[var(--color-border)] pt-4">
              <div className="flex items-start gap-3">
                <ShieldCheck
                  size={16}
                  className="mt-0.5 text-[var(--color-text-tertiary)]"
                  aria-hidden="true"
                />
                <div className="min-w-0 flex-1">
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    <div>
                      <p className="text-sm font-medium">Signed Stable updates</p>
                      <p className="text-xs text-[var(--color-text-tertiary)]">
                        {updateStatus.channel} · installed {updateStatus.currentVersion}
                      </p>
                    </div>
                    {updateStatus.configured && (
                      <Button
                        variant="quiet"
                        icon={RefreshCw}
                        onClick={() => void checkForUpdate()}
                        disabled={busy !== null}
                      >
                        {busy === 'checking' ? 'Checking…' : 'Check for updates'}
                      </Button>
                    )}
                  </div>
                  {!updateStatus.configured && (
                    <p className="mt-2 text-xs leading-5 text-[var(--color-text-tertiary)]">
                      This development or unsigned build has no release trust key and will
                      not contact an update service. Existing 0.4.0 installations need the
                      signed 0.5.0 installer once before in-app updates become available.
                    </p>
                  )}
                </div>
              </div>

              {preview && (
                <div className="mt-4 rounded-lg border border-[var(--color-border-strong)] bg-[var(--color-bg-tertiary)] p-4">
                  <div className="flex flex-wrap items-start justify-between gap-3">
                    <div>
                      <p className="text-sm font-semibold">
                        Aether {preview.version} is available
                      </p>
                      <p className="mt-1 text-xs text-[var(--color-text-tertiary)]">
                        From {preview.currentVersion}
                        {preview.publishedAt
                          ? ` · published ${formatReleaseDate(preview.publishedAt)}`
                          : ''}
                      </p>
                    </div>
                    <Button
                      variant="quiet"
                      icon={X}
                      onClick={() => void dismissUpdate()}
                      disabled={busy !== null}
                    >
                      Dismiss
                    </Button>
                  </div>
                  <p className="mt-3 whitespace-pre-wrap text-xs leading-5 text-[var(--color-text-secondary)]">
                    {preview.notes || 'This release does not include additional notes.'}
                  </p>
                  <p className="mt-3 text-xs leading-5 text-[var(--color-text-tertiary)]">
                    Aether downloads and verifies the signed installer before Windows runs
                    it. The app closes during installation and restarts afterward.
                  </p>
                  <Button
                    className="mt-3"
                    icon={Download}
                    onClick={() => void installUpdate()}
                    disabled={busy !== null}
                  >
                    Download, verify & install
                  </Button>
                </div>
              )}

              {progress && busy === 'installing' && (
                <div className="mt-4" aria-live="polite">
                  <div className="mb-2 flex justify-between text-xs text-[var(--color-text-tertiary)]">
                    <span>{progressLabel(progress.stage)}</span>
                    <span>{formatProgress(progress)}</span>
                  </div>
                  <div
                    className="h-1.5 overflow-hidden rounded-full bg-[var(--color-bg-primary)]"
                    role="progressbar"
                    aria-label="Update download"
                    aria-valuemin={0}
                    aria-valuemax={progress.total ?? undefined}
                    aria-valuenow={progress.total ? progress.downloaded : undefined}
                  >
                    <div
                      className="h-full bg-[var(--color-accent)] transition-[width]"
                      style={{ width: progressWidth(progress) }}
                    />
                  </div>
                </div>
              )}
            </div>
          </>
        )}
        {(error || notice) && (
          <p
            role={error ? 'alert' : 'status'}
            className={`text-sm ${error ? 'text-[var(--color-danger)]' : 'text-[var(--color-success)]'}`}
          >
            {error ?? notice}
          </p>
        )}
      </Surface>
    </section>
  )
}

function StatusRow({
  icon: Icon,
  title,
  detail,
  ready,
}: {
  icon: typeof Keyboard
  title: string
  detail: string
  ready: boolean
}) {
  return (
    <div className="flex items-start gap-3">
      <Icon
        size={16}
        className="mt-0.5 text-[var(--color-text-tertiary)]"
        aria-hidden="true"
      />
      <div className="min-w-0 flex-1">
        <p className="text-sm font-medium">{title}</p>
        <p className="text-xs text-[var(--color-text-tertiary)]">{detail}</p>
      </div>
      {ready && (
        <CheckCircle2
          size={16}
          className="text-[var(--color-success)]"
          aria-label="Ready"
        />
      )}
    </div>
  )
}

function formatReleaseDate(value: string): string {
  const date = new Date(value)
  return Number.isNaN(date.valueOf()) ? 'date unavailable' : date.toLocaleDateString()
}

function progressLabel(stage: DownloadProgress['stage']): string {
  if (stage === 'verifying') return 'Verifying signature'
  if (stage === 'installing') return 'Starting Windows installer'
  return 'Downloading signed update'
}

function formatProgress(progress: DownloadProgress): string {
  if (progress.stage !== 'downloading') return ''
  if (!progress.total) return formatBytes(progress.downloaded)
  return `${Math.min(100, Math.round((progress.downloaded / progress.total) * 100))}%`
}

function progressWidth(progress: DownloadProgress): string {
  if (progress.stage !== 'downloading') return '100%'
  if (!progress.total) return '35%'
  return `${Math.min(100, (progress.downloaded / progress.total) * 100)}%`
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}
