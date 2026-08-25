import { useEffect, useRef } from 'react'
import { AlertTriangle } from 'lucide-react'
import { Button } from '@/components/ui/AetherUI'

interface Props {
  title: string
  message: string
  confirmLabel?: string
  danger?: boolean
  onConfirm: () => void
  onCancel: () => void
}

export function ConfirmDialog({
  title,
  message,
  confirmLabel = 'Confirm',
  danger,
  onConfirm,
  onCancel,
}: Props) {
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onCancel()
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [onCancel])

  useEffect(() => {
    ref.current?.focus()
  }, [])

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      onClick={onCancel}
    >
      <div
        className="absolute inset-0"
        style={{ backgroundColor: 'var(--color-bg-overlay)' }}
      />
      <div
        ref={ref}
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="confirm-dialog-title"
        aria-describedby="confirm-dialog-message"
        tabIndex={-1}
        className="aether-dialog relative w-full max-w-[400px] p-6"
        style={{
          backgroundColor: 'var(--color-bg-elevated)',
          border: '1px solid var(--color-border)',
        }}
        onClick={(event) => event.stopPropagation()}
      >
        {danger && (
          <div
            className="mb-4 flex h-10 w-10 items-center justify-center rounded-full"
            style={{ backgroundColor: 'rgb(220 38 38 / 0.1)' }}
          >
            <AlertTriangle size={20} style={{ color: 'var(--color-danger)' }} />
          </div>
        )}
        <h3
          id="confirm-dialog-title"
          className="mb-2 text-base font-semibold"
          style={{ color: 'var(--color-text-primary)' }}
        >
          {title}
        </h3>
        <p
          id="confirm-dialog-message"
          className="mb-6 text-sm leading-relaxed"
          style={{ color: 'var(--color-text-secondary)' }}
        >
          {message}
        </p>
        <div className="flex justify-end gap-3">
          <Button type="button" variant="secondary" onClick={onCancel}>
            Cancel
          </Button>
          <Button
            type="button"
            variant={danger ? 'danger' : 'primary'}
            onClick={onConfirm}
          >
            {confirmLabel}
          </Button>
        </div>
      </div>
    </div>
  )
}
