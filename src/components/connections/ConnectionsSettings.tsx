import {
  Cable,
  CircleAlert,
  LoaderCircle,
  RefreshCw,
  SlidersHorizontal,
} from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import type { FormEvent } from 'react'
import * as db from '@/lib/db/tauri'
import { useConnections } from '@/hooks/useConnections'
import {
  connectionStatusDetails,
  displayCapability,
  displayProviderId,
  formatConnectionTime,
  calendarProviderDetails,
  isCalendarProvider,
  providerCatalog,
  safeErrorSummary,
  type CalendarProviderId,
  type ConnectionStatusTone,
} from '@/lib/integrations/presentation'
import type { IcsValidation, Integration } from '@/lib/db/types'
import { EmptyState, SectionLabel, Surface } from '@/components/ui/AetherUI'

export function ConnectionsSettings() {
  const connections = useConnections()
  const [setup, setSetup] = useState<{
    providerId: CalendarProviderId
    connectionId: string | null
  } | null>(null)
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
              onRefresh={() => void connections.refresh(connection)}
              onDisconnect={() => {
                if (!isCalendarProvider(connection.provider_id)) return
                const provider = calendarProviderDetails(connection.provider_id)
                if (
                  window.confirm(
                    `Disconnect ${provider.name}? Cached calendar items follow the standard connection removal policy.`,
                  )
                )
                  void connections.disconnectCalendar(connection)
              }}
              onReplace={() => {
                if (!isCalendarProvider(connection.provider_id)) return
                setSetup({
                  providerId: connection.provider_id,
                  connectionId: connection.id,
                })
              }}
            />
          ))}
        </div>
      )}

      <ProviderCatalog
        onSetup={(providerId) => {
          setSetup({ providerId, connectionId: null })
        }}
      />
      {setup && (
        <CalendarProviderSetup
          providerId={setup.providerId}
          connectionId={setup.connectionId}
          onClose={() => {
            setSetup(null)
          }}
          onConnected={() => {
            setSetup(null)
            void connections.load()
          }}
        />
      )}
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
  onRefresh,
  onDisconnect,
  onReplace,
}: {
  connection: Integration
  updating: boolean
  onEnabledChange: (enabled: boolean) => void
  onRefresh: () => void
  onDisconnect: () => void
  onReplace: () => void
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
      {isCalendarProvider(connection.provider_id) && (
        <div className="mt-4 border-t border-[var(--color-border)] pt-4">
          {connection.provider_id === 'brightspace' && (
            <p className="mb-3 text-xs leading-5 text-[var(--color-text-tertiary)]">
              Calendar-only connection. Aether does not read Brightspace courses,
              assignments, grades, or materials.
            </p>
          )}
          <div className="flex flex-wrap gap-2">
            <button
              type="button"
              className="aether-button aether-button--secondary focus-ring"
              disabled={updating}
              onClick={onRefresh}
            >
              <RefreshCw size={14} aria-hidden="true" /> Refresh now
            </button>
            <button
              type="button"
              className="aether-button aether-button--secondary focus-ring"
              disabled={updating}
              onClick={onReplace}
            >
              Replace subscription link
            </button>
            <button
              type="button"
              className="aether-button aether-button--secondary focus-ring"
              disabled={updating}
              onClick={onDisconnect}
            >
              Disconnect
            </button>
          </div>
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

function ProviderCatalog({
  onSetup,
}: {
  onSetup: (providerId: CalendarProviderId) => void
}) {
  return (
    <section className="mt-7" aria-labelledby="provider-catalog-heading">
      <p className="text-xs font-medium text-[var(--color-text-secondary)]">
        Provider availability
      </p>
      <h3
        id="provider-catalog-heading"
        className="mt-1 text-sm font-semibold text-[var(--color-text-primary)]"
      >
        Available connections
      </h3>
      <p className="mt-1 text-xs leading-5 text-[var(--color-text-tertiary)]">
        Calendar connectors are read-only and store renewable subscription links as
        encrypted credentials on this device.
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
            <p className="mt-1 text-xs leading-5 text-[var(--color-text-tertiary)]">
              {provider.description}
            </p>
            {provider.setupSupported ? (
              <button
                type="button"
                className="aether-button aether-button--secondary focus-ring mt-3"
                onClick={() => onSetup(provider.id)}
              >
                Connect calendar
              </button>
            ) : (
              <p className="mt-3 text-xs font-medium text-[var(--color-text-tertiary)]">
                Setup unavailable
              </p>
            )}
          </Surface>
        ))}
      </div>
    </section>
  )
}

function calendarProviderApi(providerId: CalendarProviderId) {
  return providerId === 'brightspace'
    ? {
        validate: db.validateBrightspace,
        connect: db.connectBrightspace,
        replace: db.replaceBrightspaceLink,
      }
    : {
        validate: db.validateMyTimetable,
        connect: db.connectMyTimetable,
        replace: db.replaceMyTimetableLink,
      }
}

function CalendarProviderSetup({
  providerId,
  connectionId,
  onClose,
  onConnected,
}: {
  providerId: CalendarProviderId
  connectionId: string | null
  onClose: () => void
  onConnected: () => void
}) {
  const provider = calendarProviderDetails(providerId)
  const api = calendarProviderApi(providerId)
  const [url, setUrl] = useState('')
  const [message, setMessage] = useState<string | null>(null)
  const [working, setWorking] = useState(false)
  const [validation, setValidation] = useState<IcsValidation | null>(null)
  const [validatedUrl, setValidatedUrl] = useState<string | null>(null)
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    inputRef.current?.focus()
  }, [])

  const submit = async (event: FormEvent) => {
    event.preventDefault()
    setWorking(true)
    setMessage(null)
    try {
      if (!validation || validatedUrl !== url) {
        const inspected = await api.validate(url)
        if (!inspected.usable)
          throw new Error('This link did not contain a usable calendar.')
        setValidation(inspected)
        setValidatedUrl(url)
        return
      }
      if (!validation.usable)
        throw new Error('This link did not contain a usable calendar.')
      if (connectionId) await api.replace(connectionId, url)
      else await api.connect(url)
      setUrl('')
      onConnected()
    } catch {
      setMessage(
        `Aether could not validate or connect this ${provider.name} calendar link. Your link was not saved.`,
      )
      setUrl('')
      setValidation(null)
      setValidatedUrl(null)
    } finally {
      setWorking(false)
    }
  }
  const headingId = `${providerId}-setup-heading`
  return (
    <Surface
      className="mt-4 p-5"
      role="dialog"
      aria-modal="true"
      aria-labelledby={headingId}
    >
      <h3
        id={headingId}
        className="text-sm font-semibold text-[var(--color-text-primary)]"
      >
        {connectionId
          ? `Replace ${provider.name} subscription link`
          : `Connect ${provider.name}`}
      </h3>
      <p className="mt-1 text-xs leading-5 text-[var(--color-text-tertiary)]">
        Paste the renewable HTTPS calendar subscription link from {provider.name}. It is
        stored as a private bearer credential and is never shown again.
      </p>
      {providerId === 'brightspace' && (
        <p className="mt-2 text-xs leading-5 text-[var(--color-text-secondary)]">
          This connection imports calendar events only. It does not grant Aether access to
          courses, assignments, grades, or materials.
        </p>
      )}
      <form className="mt-4 space-y-3" onSubmit={submit}>
        <label className="block text-sm font-medium text-[var(--color-text-primary)]">
          Calendar subscription link
          <input
            ref={inputRef}
            required
            type="url"
            autoComplete="off"
            value={url}
            onChange={(event) => {
              setUrl(event.target.value)
              setValidation(null)
              setValidatedUrl(null)
              setMessage(null)
            }}
            className="focus-ring mt-1 block w-full rounded-md border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-2"
          />
        </label>
        {message && (
          <p role="alert" className="text-sm text-[var(--color-danger)]">
            {message}
          </p>
        )}
        {validation && (
          <div
            className="rounded-md border border-[var(--color-border)] bg-[var(--color-bg)] px-3 py-3"
            role="status"
          >
            <p className="text-xs font-medium text-[var(--color-text-primary)]">
              Calendar link validated
            </p>
            <dl className="mt-2 grid gap-2 text-xs sm:grid-cols-2">
              {validation.display_name && (
                <Detail label="Calendar" value={validation.display_name} />
              )}
              {validation.public_host && (
                <Detail label="Public host" value={validation.public_host} />
              )}
              <Detail label="Events found" value={String(validation.event_count)} />
              {validation.covered_start && (
                <Detail label="Coverage starts" value={validation.covered_start} />
              )}
              {validation.covered_end && (
                <Detail label="Coverage ends" value={validation.covered_end} />
              )}
            </dl>
            {validation.warnings.map((warning) => (
              <p key={warning} className="mt-2 text-xs text-[var(--color-text-tertiary)]">
                {warning}
              </p>
            ))}
          </div>
        )}
        <div className="flex gap-2">
          <button type="submit" className="aether-button focus-ring" disabled={working}>
            {working
              ? validation
                ? 'Connecting…'
                : 'Validating…'
              : validation
                ? connectionId
                  ? 'Replace subscription link'
                  : `Connect ${provider.name}`
                : 'Validate calendar link'}
          </button>
          <button
            type="button"
            className="aether-button aether-button--secondary focus-ring"
            onClick={() => {
              setUrl('')
              setValidation(null)
              setValidatedUrl(null)
              onClose()
            }}
          >
            Cancel
          </button>
        </div>
      </form>
    </Surface>
  )
}
