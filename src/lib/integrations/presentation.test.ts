import { describe, expect, it } from 'vitest'
import {
  connectionStatusDetails,
  displayProviderId,
  isUnsupportedCalendarConnection,
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

  it('advertises only MyTimetable calendar setup and classifies legacy ICS rows', () => {
    expect(providerCatalog).toHaveLength(3)
    expect(
      providerCatalog.find((provider) => provider.id === 'my_timetable')?.setupSupported,
    ).toBe(true)
    expect(
      providerCatalog
        .filter((provider) => provider.id !== 'my_timetable')
        .every((provider) => !provider.setupSupported),
    ).toBe(true)
    expect(providerCatalog.map((provider) => String(provider.id))).not.toContain(
      'brightspace',
    )
    expect(isUnsupportedCalendarConnection('brightspace', 'ics_feed')).toBe(true)
    expect(isUnsupportedCalendarConnection('my_timetable', 'ics_feed')).toBe(false)
    expect(isUnsupportedCalendarConnection('calendar_ics', 'ics_feed')).toBe(false)
  })
})
