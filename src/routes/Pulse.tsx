import { useMemo } from 'react'
import { Sparkles, ArrowRight } from 'lucide-react'

function getGreeting(): string {
  const hour = new Date().getHours()
  if (hour < 12) return 'Good morning'
  if (hour < 17) return 'Good afternoon'
  return 'Good evening'
}

function formatDate(): string {
  const now = new Date()
  return now.toLocaleDateString('en-US', {
    weekday: 'long',
    month: 'long',
    day: 'numeric',
  })
}

export function Pulse() {
  const greeting = useMemo(() => getGreeting(), [])
  const date = useMemo(() => formatDate(), [])

  return (
    <div className="flex-1 flex flex-col items-center justify-center px-8 py-12 min-h-0">
      <div className="w-full max-w-[560px] mx-auto text-center">
        {/* Greeting */}
        <h1
          className="text-3xl font-semibold tracking-tight mb-2"
          style={{ color: 'var(--color-text-primary)' }}
        >
          {greeting}
        </h1>
        <p
          className="text-sm mb-12"
          style={{ color: 'var(--color-text-tertiary)' }}
        >
          {date}
        </p>

        {/* Empty state — calm and intentional */}
        <div
          className="rounded-2xl p-10 text-center"
          style={{
            backgroundColor: 'var(--color-bg-secondary)',
            border: `1px solid var(--color-border)`,
          }}
        >
          <div
            className="w-12 h-12 rounded-xl flex items-center justify-center mx-auto mb-5"
            style={{ backgroundColor: 'var(--color-accent-muted)' }}
          >
            <Sparkles size={22} strokeWidth={1.75} style={{ color: 'var(--color-accent)' }} />
          </div>
          <h2
            className="text-lg font-semibold mb-2"
            style={{ color: 'var(--color-text-primary)' }}
          >
            Welcome to Aether
          </h2>
          <p
            className="text-sm max-w-[360px] mx-auto mb-6 leading-relaxed"
            style={{ color: 'var(--color-text-secondary)' }}
          >
            Your personal workspace is ready. Create your first Space to
            begin organizing notes, tasks, files, and AI-assisted thinking.
          </p>
          <button
            className="inline-flex items-center gap-2 px-5 py-2.5 rounded-lg text-sm font-medium transition-colors duration-100"
            style={{
              backgroundColor: 'var(--color-accent)',
              color: 'var(--color-accent-text)',
            }}
          >
            <span>Create your first Space</span>
            <ArrowRight size={14} strokeWidth={2} />
          </button>
        </div>

        {/* Subtle hint about Command palette */}
        <p
          className="mt-8 text-xs"
          style={{ color: 'var(--color-text-tertiary)' }}
        >
          Press{' '}
          <kbd
            className="text-[10px] font-medium px-1.5 py-0.5 rounded mx-0.5"
            style={{
              backgroundColor: 'var(--color-bg-tertiary)',
              color: 'var(--color-text-secondary)',
              border: `1px solid var(--color-border)`,
            }}
          >
            Ctrl+K
          </kbd>{' '}
          to open the Command palette
        </p>
      </div>
    </div>
  )
}
