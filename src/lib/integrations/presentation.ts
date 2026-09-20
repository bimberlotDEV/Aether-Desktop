import type { Integration, IntegrationUpdateInput } from '@/lib/db/types'

export const connectionStatusDetails = {
  connected: {
    label: 'Connected',
    tone: 'success',
    description: 'The connection is available.',
  },
  syncing: {
    label: 'Syncing',
    tone: 'accent',
    description: 'A sync is currently in progress.',
  },
  degraded: {
    label: 'Degraded',
    tone: 'warning',
    description: 'The connection is available with reduced reliability.',
  },
  reauthentication_required: {
    label: 'Reauthentication required',
    tone: 'warning',
    description: 'The connection needs authentication before it can continue.',
  },
  permission_denied: {
    label: 'Permission denied',
    tone: 'danger',
    description: 'The connection does not have the required permission.',
  },
  rate_limited: {
    label: 'Rate limited',
    tone: 'warning',
    description: 'The connection is temporarily limited by its service.',
  },
  institution_configuration_required: {
    label: 'Institution configuration required',
    tone: 'warning',
    description: 'This connection needs configuration from its institution.',
  },
  unsupported: {
    label: 'Unsupported',
    tone: 'quiet',
    description: 'This connection type is not supported by this version of Aether.',
  },
  disconnected: {
    label: 'Disconnected',
    tone: 'quiet',
    description: 'No active connection is available.',
  },
} as const

export type ConnectionStatusTone =
  (typeof connectionStatusDetails)[keyof typeof connectionStatusDetails]['tone']

const sensitiveValuePattern =
  /\b(?:https?:\/\/|www\.)\S+|\b(?:sk|pk|api[_-]?key|token|secret|bearer|password)[_-]?[a-z0-9][a-z0-9._-]{5,}\b/gi

export function displayProviderId(providerId: string) {
  if (/https?:\/\/|www\./i.test(providerId)) return 'Configured provider'
  return providerId
    .split(/[-_./]+/)
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(' ')
}

export function safeErrorSummary(error: string | null) {
  if (!error) return null
  const safe = error.replace(sensitiveValuePattern, '[redacted]').trim()
  if (!safe || safe === '[redacted]')
    return 'A connection error was recorded. Details are unavailable for privacy.'
  return safe.length > 240 ? `${safe.slice(0, 237)}…` : safe
}

export function formatConnectionTime(value: string | null) {
  if (!value) return 'Not yet'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return 'Unavailable'
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(date)
}

export function integrationUpdateWithEnabled(
  integration: Integration,
  enabled: boolean,
): IntegrationUpdateInput {
  return {
    enabled,
    advertisedCapabilities: integration.advertised_capabilities,
    effectiveCapabilities: integration.effective_capabilities,
    syncModes: integration.sync_modes,
    syncConfig: integration.sync_config,
    connectionStatus: integration.connection_status,
    syncStatus: integration.sync_status,
    disconnectReason: integration.disconnect_reason,
    lastAttemptedAt: integration.last_attempted_at,
    lastSuccessfulSyncAt: integration.last_successful_sync_at,
    nextAllowedSyncAt: integration.next_allowed_sync_at,
    lastSyncErrorCode: integration.last_sync_error_code,
    lastSyncErrorMessage: integration.last_sync_error_message,
    lastSyncEtag: integration.last_sync_etag,
    lastSyncLastModified: integration.last_sync_last_modified,
    syncCursor: integration.sync_cursor,
    rateLimitRemaining: integration.rate_limit_remaining,
    retryAfterAt: integration.retry_after_at,
    credentialExpiresAt: integration.credential_expires_at,
    credentialRotatedAt: integration.credential_rotated_at,
  }
}
