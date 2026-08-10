import { useEffect, useState } from 'react'
import { Bell, CheckCircle2, Keyboard, PanelTopClose } from 'lucide-react'
import type { NativeStatus } from '@/lib/db/types'
import * as native from '@/lib/db/tauri'

export function NativeSettings() {
  const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  const [status, setStatus] = useState<NativeStatus | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)

  useEffect(() => {
    if (!isTauri) return
    native
      .getNativeStatus()
      .then(setStatus)
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

  return (
    <section>
      <h2 className="mb-4 text-sm font-semibold">Windows desktop</h2>
      <div className="space-y-4 rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-5">
        {!isTauri ? (
          <p className="text-sm text-[var(--color-text-secondary)]">
            Native desktop features are available in the installed Aether app.
          </p>
        ) : !status ? (
          <p className="text-sm text-[var(--color-text-tertiary)]">
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
              <Bell size={16} className="text-[var(--color-text-tertiary)]" />
              <div className="min-w-0 flex-1">
                <p className="text-sm font-medium">Native notifications</p>
                <p className="text-xs text-[var(--color-text-tertiary)]">
                  Windows controls notification visibility and focus-assist behavior.
                </p>
              </div>
              <button
                onClick={() => void testNotification()}
                disabled={!status.notificationsAvailable}
                className="rounded-md border border-[var(--color-border)] px-3 py-2 text-sm font-medium disabled:opacity-50 focus-ring"
              >
                Send test
              </button>
            </div>
            <p className="border-t border-[var(--color-border)] pt-3 text-xs text-[var(--color-text-tertiary)]">
              Updates are intentionally disabled until a signed release key and trusted
              endpoint are configured.
            </p>
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
      </div>
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
      <Icon size={16} className="mt-0.5 text-[var(--color-text-tertiary)]" />
      <div className="min-w-0 flex-1">
        <p className="text-sm font-medium">{title}</p>
        <p className="text-xs text-[var(--color-text-tertiary)]">{detail}</p>
      </div>
      {ready && <CheckCircle2 size={16} className="text-[var(--color-success)]" />}
    </div>
  )
}
