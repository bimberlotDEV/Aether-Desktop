import {
  Cable,
  CircleAlert,
  LoaderCircle,
  RefreshCw,
  SlidersHorizontal,
} from 'lucide-react'
import { useEffect, useRef } from 'react'
import { useConnections } from '@/hooks/useConnections'
import {
  connectionStatusDetails,
  displayCapability,
  displayProviderId,
  formatConnectionTime,
  providerCatalog,
  safeErrorSummary,
  type ConnectionStatusTone,
} from '@/lib/integrations/presentation'
import type { Integration } from '@/lib/db/types'
import { EmptyState, SectionLabel, Surface } from '@/components/ui/AetherUI'

export function ConnectionsSettings() {
  const connections = useConnections()
  const sectionRef = useRef<HTMLElement>(null)

  useEffect(() => {
    if (window.location.hash === '#connections') sectionRef.current?.focus()
  }, [])

  return (
    <section
      ref={sectionRef}
      id="connections"
      aria-labelledby="connections-heading"
      tabIndex={-1}
    >
      <SectionLabel meta={connections.connections.length || undefined}>
        Connections
      </SectionLabel>
      <div className="mb-4 flex items-start gap-3">
        <Cable
          size={18}
          className="mt-0.5 shrink-0 text-[var(--color-text-secondary)]"
          aria-hidden="true"
        />
        <div>
          <h2
            id="connections-heading"
            className="text-sm font-semibold text-[var(--color-text-primary)]"
          >
            Connected services
          </h2>
          <p className="mt-1 text-xs leading-5 text-[var(--color-text-tertiary)]">
            Connection status comes from this device. Provider setup and sync controls
            appear only when Aether supports them.
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
        <UnavailableState onRetry={() => void connections.load()} />
      ) : connections.connections.length === 0 ? (
        <EmptyConnections isTauri={connections.isTauri} />
      ) : (
        <div className="space-y-3">
          {connections.error && <ConnectionAlert message={connections.error} />}
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

      <ProviderCatalog />
    </section>
  )
}

function UnavailableState({ onRetry }: { onRetry: () => void }) {
  return (
    <Surface className="p-5" role="alert">
      <p className="text-sm font-medium text-[var(--color-danger)]">
        Connections are unavailable
      </p>
      <p className="mt-1 text-xs leading-5 text-[var(--color-text-secondary)]">
        Aether could not read local connection status. No connection state is being shown.
      </p>
      <button
        type="button"
        className="aether-button aether-button--secondary focus-ring mt-4"
        onClick={onRetry}
      >
        <RefreshCw size={14} aria-hidden="true" /> Try again
      </button>
    </Surface>
  )
}

function EmptyConnections({ isTauri }: { isTauri: boolean }) {
  return (
    <Surface className="p-1">
      <EmptyState
        compact
        icon={SlidersHorizontal}
        eyebrow="Local connections"
        title="No connections configured"
        description={
          isTauri
            ? 'No service has been connected in this workspace. Aether will show only real connections when provider setup is available.'
            : 'Connection status is available in the Aether desktop app.'
        }
      />
    </Surface>
  )
}

function ConnectionAlert({ message }: { message: string }) {
  return (
    <div
      className="flex gap-2 rounded-md border border-[var(--color-danger)]/40 bg-[var(--color-bg)] px-3 py-2 text-sm text-[var(--color-danger)]"
      role="alert"
    >
      <CircleAlert size={16} className="mt-0.5 shrink-0" aria-hidden="true" />
      <p>Some connection details may be outdated. {message}</p>
    </div>
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
  const statusId = `connection-status-${connection.id}`

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
          <p
            id={statusId}
            className="mt-1 text-xs leading-5 text-[var(--color-text-tertiary)]"
          >
            {status.description}
          </p>
        </div>
        <label className="flex cursor-pointer items-center gap-2 text-sm font-medium text-[var(--color-text-primary)]">
          <input
            type="checkbox"
            checked={connection.enabled}
            disabled={updating}
            aria-describedby={statusId}
            onChange={(event) => onEnabledChange(event.target.checked)}
            className="h-4 w-4 accent-[var(--color-accent)] focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[var(--color-accent)]"
          />
          {updating ? 'Saving…' : 'Enabled'}
        </label>
      </div>

      <dl className="mt-5 grid gap-x-6 gap-y-4 text-sm sm:grid-cols-2 xl:grid-cols-3">
        <Detail label="Sync state" value={displayCapability(connection.sync_status)} />
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
        {connection.rate_limit_remaining !== null && (
          <Detail
            label="Requests remaining"
            value={String(connection.rate_limit_remaining)}
          />
        )}
      </dl>

      <div className="mt-5 grid gap-4 border-t border-[var(--color-border)] pt-4 sm:grid-cols-2">
        <CapabilityList
          label="Advertised capabilities"
          capabilities={connection.advertised_capabilities}
          description="Provider-declared capabilities."
        />
        <CapabilityList
          label="Effective capabilities"
          capabilities={connection.effective_capabilities}
          description="Capabilities currently available to Aether."
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
  description,
}: {
  label: string
  capabilities: string[]
  description: string
}) {
  return (
    <div>
      <p className="text-xs font-medium text-[var(--color-text-secondary)]">{label}</p>
      <p className="mt-1 text-xs text-[var(--color-text-tertiary)]">{description}</p>
      {capabilities.length > 0 ? (
        <div className="mt-2 flex flex-wrap gap-2" aria-label={label}>
          {capabilities.map((capability) => (
            <span key={capability} className="aether-badge">
              {displayCapability(capability)}
            </span>
          ))}
        </div>
      ) : (
        <p className="mt-2 text-xs text-[var(--color-text-tertiary)]">None confirmed.</p>
      )}
    </div>
  )
}

function StatusBadge({ label, tone }: { label: string; tone: ConnectionStatusTone }) {
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

function ProviderCatalog() {
  return (
    <section className="mt-7" aria-labelledby="provider-catalog-heading">
      <p className="text-xs font-medium text-[var(--color-text-secondary)]">
        Provider availability
      </p>
      <h3
        id="provider-catalog-heading"
        className="mt-1 text-sm font-semibold text-[var(--color-text-primary)]"
      >
        Not available to set up yet
      </h3>
      <p className="mt-1 text-xs leading-5 text-[var(--color-text-tertiary)]">
        These future provider categories are not connectors in this version of Aether.
        They cannot be connected or used from here.
      </p>
      <div className="mt-3 grid gap-2 sm:grid-cols-2 xl:grid-cols-4">
        {providerCatalog.map((provider) => (
          <Surface key={provider.id} className="p-3">
            <p className="text-sm font-medium text-[var(--color-text-primary)]">
              {provider.name}
            </p>
            <p className="mt-1 text-xs text-[var(--color-text-tertiary)]">
              {provider.category}
            </p>
            <p className="mt-3 text-xs font-medium text-[var(--color-text-tertiary)]">
              Setup unavailable
            </p>
          </Surface>
        ))}
      </div>
    </section>
  )
}
