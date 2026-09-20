import { Cable, LoaderCircle, RefreshCw, SlidersHorizontal } from 'lucide-react'
import { useConnections } from '@/hooks/useConnections'
import {
  connectionStatusDetails,
  displayProviderId,
  formatConnectionTime,
  safeErrorSummary,
} from '@/lib/integrations/presentation'
import type { Integration } from '@/lib/db/types'
import { EmptyState, SectionLabel, Surface } from '@/components/ui/AetherUI'

export function ConnectionsSettings() {
  const connections = useConnections()

  return (
    <section aria-labelledby="connections-heading">
      <SectionLabel meta={connections.connections.length || undefined}>
        Connections
      </SectionLabel>
      <div className="mb-4 flex items-start gap-3">
        <Cable size={18} className="mt-0.5 shrink-0 text-[var(--color-text-secondary)]" />
        <div>
          <h2
            id="connections-heading"
            className="text-sm font-semibold text-[var(--color-text-primary)]"
          >
            Connected services
          </h2>
          <p className="mt-1 text-xs leading-5 text-[var(--color-text-tertiary)]">
            Aether lists only connection types that are actually configured on this
            device. Setup and sync become available when a supported provider is added.
          </p>
        </div>
      </div>

      {connections.loading ? (
        <Surface
          className="flex items-center gap-3 p-5 text-sm text-[var(--color-text-secondary)]"
          aria-busy="true"
        >
          <LoaderCircle
            size={17}
            className="animate-spin text-[var(--color-accent)]"
            aria-hidden="true"
          />
          Reading local connection status…
        </Surface>
      ) : connections.error && connections.connections.length === 0 ? (
        <Surface className="p-5" role="alert">
          <p className="text-sm font-medium text-[var(--color-danger)]">
            Connections are unavailable
          </p>
          <p className="mt-1 text-xs leading-5 text-[var(--color-text-secondary)]">
            Aether could not read local connection status. No connection state is being
            shown.
          </p>
          <button
            type="button"
            className="aether-button aether-button--secondary focus-ring mt-4"
            onClick={() => void connections.load()}
          >
            <RefreshCw size={14} aria-hidden="true" /> Try again
          </button>
        </Surface>
      ) : connections.connections.length === 0 ? (
        <Surface className="p-1">
          <EmptyState
            compact
            icon={SlidersHorizontal}
            eyebrow="Local connections"
            title="No connections configured"
            description={
              connections.isTauri
                ? 'No service has been connected in this workspace. Aether will show only real connections when provider setup is available.'
                : 'Connection status is available in the Aether desktop app.'
            }
          />
        </Surface>
      ) : (
        <div className="space-y-3">
          {connections.error && (
            <p role="alert" className="text-sm text-[var(--color-danger)]">
              Some connection details may be outdated. {connections.error}
            </p>
          )}
          {connections.connections.map((connection) => (
            <ConnectionCard
              key={connection.id}
              connection={connection}
              updating={connections.updatingId === connection.id}
              onEnabledChange={(enabled) =>
                void connections.setEnabled(connection, enabled)
              }
            />
          ))}
        </div>
      )}
    </section>
  )
}

function ConnectionCard({
  connection,
  updating,
  onEnabledChange,
}: {
  connection: Integration
  updating: boolean
  onEnabledChange: (enabled: boolean) => void
}) {
  const status = connectionStatusDetails[connection.connection_status]
  const error = safeErrorSummary(connection.last_sync_error_message)

  return (
    <Surface className="p-5">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-2">
            <h3 className="text-sm font-semibold text-[var(--color-text-primary)]">
              {displayProviderId(connection.provider_id)}
            </h3>
            <StatusBadge label={status.label} tone={status.tone} />
            <StatusBadge
              label={connection.enabled ? 'Enabled' : 'Disabled'}
              tone={connection.enabled ? 'success' : 'quiet'}
            />
          </div>
          <p className="mt-1 text-xs leading-5 text-[var(--color-text-tertiary)]">
            {status.description}
          </p>
        </div>
        <label className="flex cursor-pointer items-center gap-2 text-sm font-medium text-[var(--color-text-primary)]">
          <input
            type="checkbox"
            checked={connection.enabled}
            disabled={updating}
            onChange={(event) => onEnabledChange(event.target.checked)}
            className="h-4 w-4 accent-[var(--color-accent)] focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[var(--color-accent)]"
          />
          {updating ? 'Saving…' : 'Enabled'}
        </label>
      </div>

      <dl className="mt-5 grid gap-x-6 gap-y-4 text-sm sm:grid-cols-2">
        <Detail label="Sync state" value={connection.sync_status.replaceAll('_', ' ')} />
        <Detail
          label="Last attempted sync"
          value={formatConnectionTime(connection.last_attempted_at)}
        />
        <Detail
          label="Last successful sync"
          value={formatConnectionTime(connection.last_successful_sync_at)}
        />
        {connection.next_allowed_sync_at && (
          <Detail
            label="Next allowed sync"
            value={formatConnectionTime(connection.next_allowed_sync_at)}
          />
        )}
        {connection.retry_after_at && (
          <Detail
            label="Retry after"
            value={formatConnectionTime(connection.retry_after_at)}
          />
        )}
      </dl>

      <div className="mt-5 grid gap-4 border-t border-[var(--color-border)] pt-4 sm:grid-cols-2">
        <CapabilityList
          label="Advertised capabilities"
          capabilities={connection.advertised_capabilities}
        />
        <CapabilityList
          label="Effective capabilities"
          capabilities={connection.effective_capabilities}
        />
      </div>

      {error && (
        <div
          className="mt-4 rounded-md border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2"
          role="status"
        >
          <p className="text-xs font-medium text-[var(--color-text-secondary)]">
            Last connection error
          </p>
          <p className="mt-1 text-sm text-[var(--color-text-primary)]">{error}</p>
        </div>
      )}
      <p className="mt-4 text-xs leading-5 text-[var(--color-text-tertiary)]">
        Connection setup, reconnect, disconnect, and sync actions are not available until
        their provider support is installed.
      </p>
    </Surface>
  )
}

function Detail({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <dt className="text-xs text-[var(--color-text-tertiary)]">{label}</dt>
      <dd className="mt-1 font-medium text-[var(--color-text-primary)]">{value}</dd>
    </div>
  )
}

function CapabilityList({
  label,
  capabilities,
}: {
  label: string
  capabilities: string[]
}) {
  return (
    <div>
      <p className="text-xs font-medium text-[var(--color-text-secondary)]">{label}</p>
      {capabilities.length > 0 ? (
        <div className="mt-2 flex flex-wrap gap-2" aria-label={label}>
          {capabilities.map((capability) => (
            <span key={capability} className="aether-badge">
              {capability.replaceAll('_', ' ')}
            </span>
          ))}
        </div>
      ) : (
        <p className="mt-1 text-xs text-[var(--color-text-tertiary)]">None confirmed.</p>
      )}
    </div>
  )
}

function StatusBadge({
  label,
  tone,
}: {
  label: string
  tone: 'success' | 'accent' | 'warning' | 'danger' | 'quiet'
}) {
  const color = {
    success: 'var(--color-success)',
    accent: 'var(--color-accent)',
    warning: 'var(--color-warning)',
    danger: 'var(--color-danger)',
    quiet: 'var(--color-text-tertiary)',
  }[tone]
  return (
    <span className="aether-badge" style={{ color, borderColor: color }}>
      {label}
    </span>
  )
}
