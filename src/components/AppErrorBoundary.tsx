import { Component, type ErrorInfo, type ReactNode } from 'react'
import { AlertTriangle, Home, RotateCcw } from 'lucide-react'

interface Props {
  children: ReactNode
}

interface State {
  failed: boolean
}

export class AppErrorBoundary extends Component<Props, State> {
  state: State = { failed: false }

  static getDerivedStateFromError(): State {
    return { failed: true }
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error('Aether rendering failed', error, info)
  }

  private reload = () => {
    window.location.reload()
  }

  private goHome = () => {
    window.history.replaceState({}, '', '/')
    window.location.reload()
  }

  render() {
    if (!this.state.failed) return this.props.children

    return (
      <main className="flex h-screen w-screen items-center justify-center bg-[var(--color-bg)] px-6">
        <section
          role="alert"
          className="w-full max-w-[460px] rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-elevated)] p-8 text-center shadow-lg"
        >
          <div className="mx-auto mb-5 flex h-12 w-12 items-center justify-center rounded-xl bg-[rgb(220_38_38_/_0.1)]">
            <AlertTriangle size={22} className="text-[var(--color-danger)]" />
          </div>
          <h1 className="text-xl font-semibold text-[var(--color-text-primary)]">
            Aether hit a problem
          </h1>
          <p className="mx-auto mt-2 max-w-sm text-sm leading-relaxed text-[var(--color-text-secondary)]">
            Your local data is safe. Reload the current screen, or return to Pulse if the
            problem keeps happening.
          </p>
          <div className="mt-6 flex flex-wrap justify-center gap-3">
            <button
              type="button"
              onClick={this.reload}
              className="inline-flex items-center gap-2 rounded-md bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-[var(--color-accent-text)] focus-ring"
            >
              <RotateCcw size={14} /> Reload Aether
            </button>
            <button
              type="button"
              onClick={this.goHome}
              className="inline-flex items-center gap-2 rounded-md border border-[var(--color-border)] px-4 py-2 text-sm font-medium text-[var(--color-text-primary)] focus-ring"
            >
              <Home size={14} /> Go to Pulse
            </button>
          </div>
        </section>
      </main>
    )
  }
}
