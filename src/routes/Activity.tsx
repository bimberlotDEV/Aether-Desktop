import { Activity as ActivityIcon } from 'lucide-react'

export function Activity() {
  return (
    <div className="flex-1 flex flex-col items-center justify-center px-8 py-12 min-h-0">
      <div className="w-full max-w-[420px] mx-auto text-center">
        <div
          className="w-12 h-12 rounded-xl flex items-center justify-center mx-auto mb-5"
          style={{ backgroundColor: 'var(--color-accent-muted)' }}
        >
          <ActivityIcon size={22} strokeWidth={1.75} style={{ color: 'var(--color-accent)' }} />
        </div>
        <h2
          className="text-lg font-semibold mb-2"
          style={{ color: 'var(--color-text-primary)' }}
        >
          No activity yet
        </h2>
        <p
          className="text-sm leading-relaxed"
          style={{ color: 'var(--color-text-secondary)' }}
        >
          As you use Aether, meaningful actions will appear here.
          A timeline of what matters — without the noise.
        </p>
      </div>
    </div>
  )
}
