import { useState } from 'react'
import { Check, FileText, ListChecks, ShieldCheck, X } from 'lucide-react'
import type { ActionPreview, AiActionDraft } from '@/lib/db/types'
import * as db from '@/lib/db/tauri'

export function AiActionProposals({
  actions,
  conversationId,
  messageId,
  onClose,
  onExecuted,
}: {
  actions: AiActionDraft[]
  conversationId: string
  messageId: string
  onClose: () => void
  onExecuted: () => void
}) {
  const [index, setIndex] = useState(0)
  const [preview, setPreview] = useState<ActionPreview | null>(null)
  const [completed, setCompleted] = useState(0)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const action = actions[index]

  async function review() {
    if (!action) return
    setBusy(true)
    setError(null)
    try {
      setPreview(
        await db.previewAiActionProposal(conversationId, messageId, action.index),
      )
    } catch (cause) {
      setError(message(cause, 'Aether could not validate this Action.'))
    } finally {
      setBusy(false)
    }
  }

  async function approve() {
    if (!preview) return
    setBusy(true)
    setError(null)
    try {
      await db.executeAction(preview.token)
      setCompleted((value) => value + 1)
      setPreview(null)
      setIndex((value) => value + 1)
      onExecuted()
    } catch (cause) {
      setPreview(null)
      setError(message(cause, 'The approved Action could not be completed.'))
    } finally {
      setBusy(false)
    }
  }

  async function reject() {
    const token = preview?.token
    setPreview(null)
    if (token) await db.cancelAction(token).catch(() => undefined)
    setIndex((value) => value + 1)
  }

  async function close() {
    if (preview) await db.cancelAction(preview.token).catch(() => undefined)
    onClose()
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4"
      onClick={() => void close()}
    >
      <div className="absolute inset-0 bg-[var(--color-bg-overlay)]" />
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby="ai-actions-title"
        onClick={(event) => event.stopPropagation()}
        className="relative w-full max-w-[560px] rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-elevated)] p-5 shadow-xl"
      >
        <div className="flex items-start gap-3">
          <ShieldCheck size={20} className="mt-0.5 text-[var(--color-accent)]" />
          <div className="flex-1">
            <h2 id="ai-actions-title" className="text-base font-semibold">
              Review AI Actions
            </h2>
            <p className="mt-1 text-xs text-[var(--color-text-tertiary)]">
              Every item needs its own preview and approval. The AI never receives
              approval tokens.
            </p>
          </div>
          <button
            onClick={() => void close()}
            aria-label="Close proposals"
            className="rounded-md p-1.5 hover:bg-[var(--color-bg-tertiary)] focus-ring"
          >
            <X size={16} />
          </button>
        </div>

        {action ? (
          <div className="mt-5 rounded-lg border border-[var(--color-border)] p-4">
            <div className="flex items-start gap-3">
              {action.actionType === 'createTask' ? (
                <ListChecks size={18} />
              ) : (
                <FileText size={18} />
              )}
              <div className="min-w-0 flex-1">
                <p className="text-xs text-[var(--color-text-tertiary)]">
                  Proposal {index + 1} of {actions.length}
                </p>
                <p className="mt-1 font-medium">{action.title}</p>
                <p className="mt-1 whitespace-pre-wrap text-sm text-[var(--color-text-secondary)]">
                  {action.detail || 'No additional details'}
                </p>
              </div>
            </div>
            {preview ? (
              <div className="mt-4 border-t border-[var(--color-border)] pt-4">
                <p className="text-sm font-medium">{preview.summary}</p>
                <p className="mt-1 text-xs text-[var(--color-text-secondary)]">
                  {preview.consequence}
                </p>
                <div className="mt-4 flex justify-end gap-2">
                  <button
                    disabled={busy}
                    onClick={() => void reject()}
                    className="aether-button aether-button--secondary focus-ring"
                  >
                    Reject
                  </button>
                  <button
                    disabled={busy}
                    onClick={() => void approve()}
                    className="aether-button aether-button--primary focus-ring"
                  >
                    <Check size={14} /> {busy ? 'Creating…' : 'Approve and create'}
                  </button>
                </div>
              </div>
            ) : (
              <div className="mt-4 flex justify-end gap-2">
                <button
                  disabled={busy}
                  onClick={() => void reject()}
                  className="aether-button aether-button--secondary focus-ring"
                >
                  Skip
                </button>
                <button
                  disabled={busy}
                  onClick={() => void review()}
                  className="aether-button aether-button--primary focus-ring"
                >
                  {busy ? 'Validating…' : 'Preview Action'}
                </button>
              </div>
            )}
          </div>
        ) : (
          <div className="mt-5 rounded-lg border border-[var(--color-border)] p-6 text-center">
            <Check className="mx-auto text-[var(--color-success)]" />
            <p className="mt-2 font-medium">Review complete</p>
            <p className="mt-1 text-sm text-[var(--color-text-secondary)]">
              {completed} of {actions.length} Actions approved and created.
            </p>
            <button
              onClick={onClose}
              className="aether-button aether-button--primary mt-4 focus-ring"
            >
              Done
            </button>
          </div>
        )}
        {error && (
          <p role="alert" className="mt-3 text-sm text-[var(--color-danger)]">
            {error}
          </p>
        )}
      </div>
    </div>
  )
}

function message(cause: unknown, fallback: string) {
  return cause instanceof Error && cause.message ? cause.message : fallback
}
