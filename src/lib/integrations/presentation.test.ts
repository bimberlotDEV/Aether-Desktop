import { describe, expect, it } from 'vitest'
import {
  connectionStatusDetails,
  displayProviderId,
  providerCatalog,
  safeErrorSummary,
} from '@/lib/integrations/presentation'

describe('integration presentation helpers', () => {
  it('covers every persisted connection status with a distinct label', () => {
    expect(Object.keys(connectionStatusDetails)).toEqual([
      'connected',
      'syncing',
      'degraded',
      'reauthentication_required',
      'permission_denied',
      'rate_limited',
      'institution_configuration_required',
      'unsupported',
      'disconnected',
    ])
    expect(
      new Set(Object.values(connectionStatusDetails).map((status) => status.label)),
    ).toHaveLength(9)
  })

  it('keeps provider labels readable without exposing an URL', () => {
    expect(displayProviderId('campus_calendar.v2')).toBe('Campus Calendar V2')
    expect(displayProviderId('https://private.example/feed')).toBe('Configured provider')
  })

  it('redacts URL and credential-shaped error values', () => {
    expect(
      safeErrorSummary(
        'Request to https://private.example/token failed with token-example123456.',
      ),
    ).toBe('Request to [redacted] failed with [redacted].')
  })

  it('marks only the two approved ICS providers as available for setup', () => {
    expect(providerCatalog).toHaveLength(4)
    expect(
      providerCatalog.find((provider) => provider.id === 'my_timetable')?.setupSupported,
    ).toBe(true)
    expect(
      providerCatalog
        .filter(
          (provider) => provider.id !== 'my_timetable' && provider.id !== 'brightspace',
        )
        .every((provider) => !provider.setupSupported),
    ).toBe(true)
    expect(
      providerCatalog.find((provider) => provider.id === 'brightspace')?.setupSupported,
    ).toBe(true)
    expect(
      providerCatalog.find((provider) => provider.id === 'brightspace')?.description,
    ).toMatch(/Calendar-only/)
  })
})
