import { useState } from 'react'
import { Check, Clipboard, FileWarning, RefreshCw } from 'lucide-react'
import { Button, SectionLabel, Surface } from '@/components/ui/AetherUI'
import type { BetaDiagnosticReport } from '@/lib/db/types'
import * as native from '@/lib/db/tauri'

export function BetaDiagnostics() {
  const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  const [report, setReport] = useState<BetaDiagnosticReport | null>(null)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)

  async function generate() {
    if (!isTauri) return
    setBusy(true)
    setError(null)
    setNotice(null)
    try {
      setReport(await native.getBetaDiagnostics())
    } catch (cause) {
      setReport(null)
      setError(
        cause instanceof Error
          ? cause.message
          : 'Could not create the diagnostic report.',
      )
    } finally {
      setBusy(false)
    }
  }

  async function copy() {
    if (!report) return
    setError(null)
    setNotice(null)
    try {
      if (!navigator.clipboard?.writeText)
        throw new Error('Clipboard access is unavailable.')
      await navigator.clipboard.writeText(formatReport(report))
      setNotice('Diagnostic report copied. Review it before sharing.')
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Could not copy the report.')
    }
  }

  return (
    <section>
      <SectionLabel>Beta support</SectionLabel>
      <Surface className="space-y-4 p-5">
        <div className="flex items-start gap-4">
          <FileWarning
            size={19}
            className="mt-0.5 shrink-0 text-[var(--color-text-secondary)]"
            aria-hidden="true"
          />
          <div className="min-w-0 flex-1">
            <h2 className="text-sm font-semibold">Sanitized diagnostics</h2>
            <p className="mt-1 text-xs leading-5 text-[var(--color-text-tertiary)]">
              Create a local technical summary for a beta report. It contains no notes,
              prompts, filenames, paths, logs, identifiers, record counts or API keys.
              Aether never sends it automatically.
            </p>
          </div>
          <Button
            variant="quiet"
            icon={RefreshCw}
            onClick={() => void generate()}
            disabled={!isTauri || busy}
          >
            {busy ? 'Checking…' : report ? 'Refresh' : 'Create report'}
          </Button>
        </div>

        {!isTauri && (
          <p className="text-xs text-[var(--color-text-tertiary)]" role="status">
            Diagnostics are available only in the installed Aether desktop app.
          </p>
        )}

        {report && (
          <div className="space-y-3">
            <pre
              aria-label="Sanitized diagnostic report"
              className="max-h-64 overflow-auto rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-primary)] p-4 text-xs leading-5 text-[var(--color-text-secondary)]"
            >
              {formatReport(report)}
            </pre>
            <div className="flex flex-wrap items-center justify-between gap-3">
              <p className="text-xs text-[var(--color-text-tertiary)]">
                Inspect this exact text before pasting it into a report.
              </p>
              <Button variant="secondary" icon={Clipboard} onClick={() => void copy()}>
                Copy report
              </Button>
            </div>
          </div>
        )}

        {(error || notice) && (
          <p
            role={error ? 'alert' : 'status'}
            className={`flex items-center gap-2 text-xs ${error ? 'text-[var(--color-danger)]' : 'text-[var(--color-success)]'}`}
          >
            {!error && <Check size={14} aria-hidden="true" />}
            {error ?? notice}
          </p>
        )}
      </Surface>
    </section>
  )
}

function formatReport(report: BetaDiagnosticReport): string {
  return JSON.stringify(report, null, 2)
}
