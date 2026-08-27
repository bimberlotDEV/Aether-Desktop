import type { LucideIcon } from 'lucide-react'
import {
  Activity as ActivityIcon,
  Brain,
  CheckCircle2,
  FileText,
  FolderOpen,
  MessageSquare,
  RefreshCw,
  Sparkles,
} from 'lucide-react'
import { useNavigate } from 'react-router-dom'
import { useActivityFeed } from '@/hooks/useContinuity'
import type { ActivityItem } from '@/lib/db/types'
import { Button, EmptyState, Page, PageHeader, Surface } from '@/components/ui/AetherUI'

const eventIcons: Record<string, LucideIcon> = {
  space_opened: FolderOpen,
  note_created: FileText,
  note_edited: FileText,
  task_created: CheckCircle2,
  task_created_from_ai_proposal: Sparkles,
  task_completed: CheckCircle2,
  task_archived: CheckCircle2,
  vault_imported: FolderOpen,
  vault_updated: FolderOpen,
  vault_removed: FolderOpen,
  memory_created: Brain,
  memory_updated: Brain,
  memory_deleted: Brain,
  source_scanned: RefreshCw,
  ai_conversation_used: MessageSquare,
}

export function Activity() {
  const navigate = useNavigate()
  const { items, loading, error, isDesktop, reload } = useActivityFeed()

  return (
    <Page>
      <PageHeader
        eyebrow="Timeline"
        icon={ActivityIcon}
        title="Activity"
        description="A quiet, local history of meaningful changes across your workspace."
        actions={
          isDesktop ? (
            <Button icon={RefreshCw} variant="quiet" onClick={() => void reload()}>
              Refresh
            </Button>
          ) : undefined
        }
      />

      {!isDesktop ? (
        <EmptyState
          icon={ActivityIcon}
          eyebrow="Installed app"
          title="Your local timeline stays on this device"
          description="Open Aether Desktop to review real Activity from your local workspace. Browser preview never fabricates persisted events."
        />
      ) : loading ? (
        <Surface className="p-8" role="status" aria-live="polite">
          <p className="text-sm text-[var(--color-text-secondary)]">
            Loading meaningful Activity…
          </p>
        </Surface>
      ) : error ? (
        <EmptyState
          icon={ActivityIcon}
          eyebrow="Timeline unavailable"
          title="Activity could not be loaded"
          description={error}
          action={{ label: 'Try again', onClick: () => void reload() }}
        />
      ) : items.length === 0 ? (
        <EmptyState
          icon={ActivityIcon}
          eyebrow="Nothing to review"
          title="Your timeline is quiet"
          description="Open a Space or create, edit, complete, index, or use something. Only meaningful local actions appear here."
        />
      ) : (
        <ol className="space-y-2" aria-label="Meaningful workspace Activity">
          {items.map((item) => (
            <ActivityRow
              key={item.id}
              item={item}
              onOpen={() => navigate(item.destination)}
            />
          ))}
        </ol>
      )}
    </Page>
  )
}

function ActivityRow({ item, onOpen }: { item: ActivityItem; onOpen: () => void }) {
  const Icon = eventIcons[item.eventType] ?? ActivityIcon
  return (
    <li>
      <button className="aether-activity-row" onClick={onOpen}>
        <span className="aether-activity-icon" aria-hidden="true">
          <Icon size={16} strokeWidth={1.8} />
        </span>
        <span className="min-w-0 flex-1 text-left">
          <span className="block truncate text-sm font-medium text-[var(--color-text-primary)]">
            {item.title}
          </span>
          <span className="mt-0.5 block truncate text-xs text-[var(--color-text-tertiary)]">
            {[item.spaceName, item.detail].filter(Boolean).join(' · ') || 'Workspace'}
          </span>
        </span>
        <time
          className="shrink-0 text-xs text-[var(--color-text-tertiary)]"
          dateTime={item.createdAt}
          title={formatDate(item.createdAt)}
        >
          {relativeTime(item.createdAt)}
        </time>
      </button>
    </li>
  )
}

function asDate(value: string) {
  return new Date(/[zZ]|[+-]\d\d:\d\d$/.test(value) ? value : `${value}Z`)
}

function formatDate(value: string) {
  return asDate(value).toLocaleString()
}

function relativeTime(value: string) {
  const seconds = Math.max(0, Math.floor((Date.now() - asDate(value).getTime()) / 1000))
  if (seconds < 60) return 'just now'
  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) return `${minutes}m ago`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours}h ago`
  const days = Math.floor(hours / 24)
  if (days < 7) return `${days}d ago`
  return asDate(value).toLocaleDateString()
}
