export const connectionStatusDetails = {
  connected: {
    label: 'Connected',
    tone: 'success',
    description: 'Available to this workspace.',
  },
  syncing: {
    label: 'Syncing',
    tone: 'accent',
    description: 'A local sync is in progress.',
  },
  degraded: {
    label: 'Degraded',
    tone: 'warning',
    description: 'Available, but its recent sync was unreliable.',
  },
  reauthentication_required: {
    label: 'Reauthentication required',
    tone: 'warning',
    description: 'Authentication is needed before the next sync.',
  },
  permission_denied: {
    label: 'Permission denied',
    tone: 'danger',
    description: 'The connected account no longer grants the required access.',
  },
  rate_limited: {
    label: 'Rate limited',
    tone: 'warning',
    description: 'The service has temporarily limited sync requests.',
  },
  institution_configuration_required: {
    label: 'Institution configuration required',
    tone: 'warning',
    description: 'This service needs configuration from its institution.',
  },
  unsupported: {
    label: 'Unsupported',
    tone: 'quiet',
    description: 'This connection type is unsupported by this Aether version.',
  },
  disconnected: {
    label: 'Disconnected',
    tone: 'quiet',
    description: 'No active connection is available.',
  },
} as const

export type ConnectionStatusTone =
  (typeof connectionStatusDetails)[keyof typeof connectionStatusDetails]['tone']

export const providerCatalog = [
  {
    id: 'my_timetable',
    name: 'MyTimetable',
    category: 'School schedule',
    setupSupported: true,
  },
  {
    id: 'brightspace',
    name: 'Brightspace',
    category: 'Learning platform',
    setupSupported: false,
  },
  { id: 'github', name: 'GitHub', category: 'Development', setupSupported: false },
  { id: 'n8n', name: 'n8n', category: 'Automation', setupSupported: false },
] as const

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

export function displayCapability(capability: string) {
  return capability.replaceAll('_', ' ')
}

export function safeErrorSummary(error: string | null) {
  if (!error) return null
  const safe = error.replace(sensitiveValuePattern, '[redacted]').trim()
  if (!safe || safe === '[redacted]') {
    return 'A connection error was recorded. Details are unavailable for privacy.'
  }
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
