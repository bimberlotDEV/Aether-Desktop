import { useState } from 'react'
import { CheckCircle2, KeyRound, LoaderCircle, Trash2 } from 'lucide-react'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { useAiSettings } from '@/hooks/useAi'

export function AiSettings() {
  const ai = useAiSettings()
  const [apiKey, setApiKey] = useState('')
  const [busy, setBusy] = useState<'save' | 'test' | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  const [confirmRemove, setConfirmRemove] = useState(false)

  async function save(event: React.FormEvent) {
    event.preventDefault()
    if (!apiKey.trim()) return
    setBusy('save')
    setSuccess(null)
    try {
      await ai.save(apiKey)
      setApiKey('')
      setSuccess('API key stored securely for your Windows account.')
    } catch {
      // The hook exposes the actionable error below the form.
    } finally {
      setBusy(null)
    }
  }

  async function test() {
    setBusy('test')
    setSuccess(null)
    try {
      setSuccess(await ai.test())
    } catch {
      // The hook exposes the actionable error below the controls.
    } finally {
      setBusy(null)
    }
  }

  return (
    <section>
      <div className="mb-4 flex items-center gap-2">
        <KeyRound size={16} className="text-[var(--color-text-secondary)]" />
        <h2 className="text-sm font-semibold text-[var(--color-text-primary)]">
          DeepSeek
        </h2>
      </div>
      <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-5">
        <div className="mb-4 flex items-start gap-3">
          <div className="flex-1">
            <p className="text-sm font-medium text-[var(--color-text-primary)]">
              {ai.loading
                ? 'Checking configuration…'
                : ai.status === 'configured'
                  ? 'API key configured'
                  : 'API key required'}
            </p>
            <p className="mt-1 text-xs leading-relaxed text-[var(--color-text-tertiary)]">
              The key is protected with Windows DPAPI and never exposed to the frontend
              after saving. Messages and explicitly attached context are sent to DeepSeek.
            </p>
          </div>
          {ai.status === 'configured' && (
            <CheckCircle2 size={18} className="text-[var(--color-success)]" />
          )}
        </div>

        {!ai.isTauri ? (
          <p className="rounded-md border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2 text-sm text-[var(--color-text-secondary)]">
            AI configuration is available in the Aether desktop app.
          </p>
        ) : (
          <form onSubmit={save} className="flex gap-2">
            <label className="sr-only" htmlFor="deepseek-api-key">
              DeepSeek API key
            </label>
            <input
              id="deepseek-api-key"
              type="password"
              value={apiKey}
              onChange={(event) => setApiKey(event.target.value)}
              autoComplete="off"
              placeholder={
                ai.status === 'configured' ? 'Enter a replacement key' : 'sk-…'
              }
              className="min-w-0 flex-1 rounded-md border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2 text-sm outline-none focus:border-[var(--color-accent)]"
            />
            <button
              disabled={!apiKey.trim() || busy !== null}
              className="rounded-md bg-[var(--color-accent)] px-3 py-2 text-sm font-medium text-[var(--color-accent-text)] disabled:opacity-50 focus-ring"
            >
              {busy === 'save'
                ? 'Saving…'
                : ai.status === 'configured'
                  ? 'Replace'
                  : 'Save key'}
            </button>
          </form>
        )}

        {ai.status === 'configured' && ai.isTauri && (
          <div className="mt-3 flex items-center gap-2">
            <button
              onClick={() => void test()}
              disabled={busy !== null}
              className="inline-flex items-center gap-2 rounded-md border border-[var(--color-border)] px-3 py-2 text-sm font-medium hover:bg-[var(--color-bg-tertiary)] disabled:opacity-50 focus-ring"
            >
              {busy === 'test' && <LoaderCircle size={14} className="animate-spin" />}{' '}
              Test connection
            </button>
            <button
              onClick={() => setConfirmRemove(true)}
              className="inline-flex items-center gap-2 rounded-md px-3 py-2 text-sm text-[var(--color-danger)] hover:bg-[var(--color-bg-tertiary)] focus-ring"
            >
              <Trash2 size={14} /> Remove key
            </button>
          </div>
        )}
        {(ai.error || success) && (
          <p
            role="status"
            className={`mt-3 text-sm ${ai.error ? 'text-[var(--color-danger)]' : 'text-[var(--color-success)]'}`}
          >
            {ai.error ?? success}
          </p>
        )}
      </div>
      {confirmRemove && (
        <ConfirmDialog
          title="Remove DeepSeek API key?"
          message="Aether will no longer be able to send AI requests until you add a new key."
          confirmLabel="Remove key"
          danger
          onConfirm={() => {
            setConfirmRemove(false)
            void ai.remove()
          }}
          onCancel={() => setConfirmRemove(false)}
        />
      )}
    </section>
  )
}
