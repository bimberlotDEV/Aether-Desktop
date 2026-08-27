import { useState } from 'react'
import { CheckCircle2, KeyRound, LoaderCircle, Trash2 } from 'lucide-react'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { useAiSettings } from '@/hooks/useAi'
import type { AiProvider } from '@/lib/db/types'

const PROVIDERS: Array<{ id: AiProvider['id']; name: string; placeholder: string }> = [
  { id: 'deepseek', name: 'DeepSeek', placeholder: 'sk-…' },
  { id: 'openai', name: 'OpenAI', placeholder: 'sk-proj-…' },
]

export function AiSettings() {
  const ai = useAiSettings()

  return (
    <section>
      <div className="mb-4 flex items-center gap-2">
        <KeyRound size={16} className="text-[var(--color-text-secondary)]" />
        <h2 className="text-sm font-semibold text-[var(--color-text-primary)]">
          AI providers
        </h2>
      </div>
      <p className="mb-4 text-xs leading-relaxed text-[var(--color-text-tertiary)]">
        Auto routing uses only providers configured here and shows the chosen route on
        every response. It never retries through another provider without asking.
      </p>
      <div className="grid gap-3 lg:grid-cols-2">
        {PROVIDERS.map((provider) => (
          <ProviderCard
            key={provider.id}
            provider={provider}
            status={
              ai.statuses.find((item) => item.provider === provider.id)?.status ??
              (ai.loading ? 'unavailable' : 'missing')
            }
            loading={ai.loading}
            error={ai.error}
            isTauri={ai.isTauri}
            onSave={(key) => ai.save(provider.id, key)}
            onTest={() => ai.test(provider.id)}
            onRemove={() => ai.remove(provider.id)}
          />
        ))}
      </div>
      <p className="mt-4 text-xs leading-relaxed text-[var(--color-text-tertiary)]">
        Keys are protected with Windows DPAPI and are never returned to the frontend.
        Conversation history and only explicitly attached context are sent to the provider
        shown on the response.
      </p>
    </section>
  )
}

function ProviderCard({
  provider,
  status,
  loading,
  error,
  isTauri,
  onSave,
  onTest,
  onRemove,
}: {
  provider: (typeof PROVIDERS)[number]
  status: 'configured' | 'missing' | 'unavailable'
  loading: boolean
  error: string | null
  isTauri: boolean
  onSave: (key: string) => Promise<void>
  onTest: () => Promise<string>
  onRemove: () => Promise<void>
}) {
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
      await onSave(apiKey)
      setApiKey('')
      setSuccess('API key stored securely for your Windows account.')
    } catch {
      // The shared hook exposes the actionable error.
    } finally {
      setBusy(null)
    }
  }

  async function test() {
    setBusy('test')
    setSuccess(null)
    try {
      setSuccess(await onTest())
    } catch {
      // The shared hook exposes the actionable error.
    } finally {
      setBusy(null)
    }
  }

  return (
    <div className="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-5">
      <div className="mb-4 flex items-start gap-3">
        <div className="flex-1">
          <p className="text-sm font-semibold text-[var(--color-text-primary)]">
            {provider.name}
          </p>
          <p className="mt-1 text-xs text-[var(--color-text-tertiary)]">
            {loading
              ? 'Checking configuration…'
              : status === 'configured'
                ? 'API key configured'
                : status === 'unavailable'
                  ? 'Credential store unavailable'
                  : 'API key not configured'}
          </p>
        </div>
        {status === 'configured' && (
          <CheckCircle2 size={18} className="text-[var(--color-success)]" />
        )}
      </div>

      {!isTauri ? (
        <p className="rounded-md border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2 text-sm text-[var(--color-text-secondary)]">
          Configuration is available in the desktop app.
        </p>
      ) : (
        <form onSubmit={save} className="flex gap-2">
          <label className="sr-only" htmlFor={`${provider.id}-api-key`}>
            {provider.name} API key
          </label>
          <input
            id={`${provider.id}-api-key`}
            type="password"
            value={apiKey}
            onChange={(event) => setApiKey(event.target.value)}
            autoComplete="off"
            placeholder={
              status === 'configured' ? 'Enter a replacement key' : provider.placeholder
            }
            className="min-w-0 flex-1 rounded-md border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2 text-sm outline-none focus:border-[var(--color-accent)]"
          />
          <button
            disabled={!apiKey.trim() || busy !== null}
            className="aether-button aether-button--primary focus-ring"
          >
            {busy === 'save'
              ? 'Saving…'
              : status === 'configured'
                ? 'Replace'
                : 'Save key'}
          </button>
        </form>
      )}

      {status === 'configured' && isTauri && (
        <div className="mt-3 flex flex-wrap items-center gap-2">
          <button
            onClick={() => void test()}
            disabled={busy !== null}
            className="aether-button aether-button--secondary focus-ring"
          >
            {busy === 'test' && <LoaderCircle size={14} className="animate-spin" />} Test{' '}
            {provider.name}
          </button>
          <button
            onClick={() => setConfirmRemove(true)}
            className="aether-button text-[var(--color-danger)] hover:bg-[var(--color-bg-tertiary)] focus-ring"
          >
            <Trash2 size={14} /> Remove {provider.name} key
          </button>
        </div>
      )}
      {(error || success) && (
        <p
          role="status"
          className={`mt-3 text-sm ${error ? 'text-[var(--color-danger)]' : 'text-[var(--color-success)]'}`}
        >
          {error ?? success}
        </p>
      )}
      {confirmRemove && (
        <ConfirmDialog
          title={`Remove ${provider.name} API key?`}
          message={`Aether will stop routing requests to ${provider.name}. Other configured providers are unaffected.`}
          confirmLabel={`Remove ${provider.name} key`}
          danger
          onConfirm={() => {
            setConfirmRemove(false)
            void onRemove()
          }}
          onCancel={() => setConfirmRemove(false)}
        />
      )}
    </div>
  )
}
