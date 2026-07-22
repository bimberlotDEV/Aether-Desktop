import { useEffect, useRef } from 'react'
import { AlertTriangle } from 'lucide-react'

interface Props {
  title: string
  message: string
  confirmLabel?: string
  danger?: boolean
  onConfirm: () => void
  onCancel: () => void
}

export function ConfirmDialog({ title, message, confirmLabel = 'Confirm', danger, onConfirm, onCancel }: Props) {
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onCancel()
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [onCancel])

  useEffect(() => {
    ref.current?.focus()
  }, [])

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center" onClick={onCancel}>
      <div className="absolute inset-0" style={{ backgroundColor: 'var(--color-bg-overlay)' }} />
      <div
        ref={ref}
        tabIndex={-1}
        className="relative w-full max-w-[380px] rounded-xl p-6 shadow-2xl"
        style={{ backgroundColor: 'var(--color-bg-elevated)', border: '1px solid var(--color-border)' }}
        onClick={e => e.stopPropagation()}
      >
        {danger && (
          <div className="w-10 h-10 rounded-full flex items-center justify-center mb-4" style={{ backgroundColor: 'rgb(220 38 38 / 0.1)' }}>
            <AlertTriangle size={20} style={{ color: 'var(--color-danger)' }} />
          </div>
        )}
        <h3 className="text-base font-semibold mb-2" style={{ color: 'var(--color-text-primary)' }}>{title}</h3>
        <p className="text-sm leading-relaxed mb-6" style={{ color: 'var(--color-text-secondary)' }}>{message}</p>
        <div className="flex gap-3 justify-end">
          <button
            onClick={onCancel}
            className="px-4 py-2 rounded-lg text-sm font-medium transition-colors"
            style={{ backgroundColor: 'var(--color-bg-tertiary)', color: 'var(--color-text-primary)' }}
          >
            Cancel
          </button>
          <button
            onClick={onConfirm}
            className="px-4 py-2 rounded-lg text-sm font-medium transition-colors"
            style={{
              backgroundColor: danger ? 'var(--color-danger)' : 'var(--color-accent)',
              color: '#fff',
            }}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  )
}
