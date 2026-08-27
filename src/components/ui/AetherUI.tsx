import type { ButtonHTMLAttributes, HTMLAttributes, ReactNode } from 'react'
import type { LucideIcon } from 'lucide-react'
import { ArrowRight } from 'lucide-react'
import { cn } from '@/lib/utils'

type PageWidth = 'compact' | 'default' | 'wide' | 'full'

const pageWidths: Record<PageWidth, string> = {
  compact: 'max-w-[760px]',
  default: 'max-w-[1040px]',
  wide: 'max-w-[1240px]',
  full: 'max-w-none',
}

export function Page({
  children,
  width = 'default',
  className,
}: {
  children: ReactNode
  width?: PageWidth
  className?: string
}) {
  return (
    <div className="aether-page-scroll">
      <div className={cn('aether-page', pageWidths[width], className)}>{children}</div>
    </div>
  )
}

export function PageHeader({
  eyebrow,
  title,
  description,
  icon: Icon,
  actions,
}: {
  eyebrow?: string
  title: string
  description?: string
  icon?: LucideIcon
  actions?: ReactNode
}) {
  return (
    <header className="aether-page-header">
      <div className="aether-page-heading">
        {Icon && (
          <div className="aether-icon-frame aether-icon-frame--accent" aria-hidden="true">
            <Icon size={18} strokeWidth={1.8} />
          </div>
        )}
        <div className="min-w-0">
          {eyebrow && <p className="aether-eyebrow">{eyebrow}</p>}
          <h1 className="aether-page-title">{title}</h1>
          {description && <p className="aether-page-description">{description}</p>}
        </div>
      </div>
      {actions && <div className="aether-page-actions">{actions}</div>}
    </header>
  )
}

type ButtonVariant = 'primary' | 'secondary' | 'quiet' | 'danger'

export function Button({
  variant = 'secondary',
  icon: Icon,
  children,
  className,
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: ButtonVariant
  icon?: LucideIcon
}) {
  return (
    <button
      className={cn('aether-button', `aether-button--${variant}`, className)}
      {...props}
    >
      {Icon && <Icon size={15} strokeWidth={2} aria-hidden="true" />}
      {children}
    </button>
  )
}

export function Surface({
  children,
  className,
  interactive = false,
  ...props
}: HTMLAttributes<HTMLDivElement> & {
  interactive?: boolean
}) {
  return (
    <div
      className={cn(
        'aether-surface',
        interactive && 'aether-surface--interactive',
        className,
      )}
      {...props}
    >
      {children}
    </div>
  )
}

export function EmptyState({
  icon: Icon,
  eyebrow,
  title,
  description,
  action,
  secondary,
  compact = false,
}: {
  icon: LucideIcon
  eyebrow?: string
  title: string
  description: string
  action?: { label: string; onClick: () => void }
  secondary?: ReactNode
  compact?: boolean
}) {
  return (
    <div className={cn('aether-empty', compact && 'aether-empty--compact')}>
      <div className="aether-empty-orbit" aria-hidden="true">
        <div className="aether-icon-frame aether-icon-frame--large">
          <Icon size={22} strokeWidth={1.7} />
        </div>
      </div>
      <div className="aether-empty-copy">
        {eyebrow && <p className="aether-eyebrow">{eyebrow}</p>}
        <h2>{title}</h2>
        <p>{description}</p>
      </div>
      {(action || secondary) && (
        <div className="aether-empty-actions">
          {action && (
            <Button variant="primary" onClick={action.onClick}>
              {action.label}
              <ArrowRight size={14} aria-hidden="true" />
            </Button>
          )}
          {secondary}
        </div>
      )}
    </div>
  )
}

export function SectionLabel({
  children,
  meta,
}: {
  children: ReactNode
  meta?: ReactNode
}) {
  return (
    <div className="aether-section-label">
      <span>{children}</span>
      {meta && <span className="aether-section-meta">{meta}</span>}
    </div>
  )
}

export function StatusDot({ tone = 'online' }: { tone?: 'online' | 'quiet' }) {
  return <span className={cn('aether-status-dot', `aether-status-dot--${tone}`)} />
}
