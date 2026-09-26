import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { IcsValidation, Integration } from '@/lib/db/types'

const mocks = vi.hoisted(() => ({
  useConnections: vi.fn(),
  setEnabled: vi.fn(),
  load: vi.fn(),
  refresh: vi.fn(),
  disconnectCalendar: vi.fn(),
  removeUnsupportedCalendar: vi.fn(),
  validateMyTimetable: vi.fn(),
  connectMyTimetable: vi.fn(),
  replaceMyTimetableLink: vi.fn(),
}))
vi.mock('@/hooks/useConnections', () => ({ useConnections: mocks.useConnections }))
vi.mock('@/lib/db/tauri', () => ({
  validateMyTimetable: mocks.validateMyTimetable,
  connectMyTimetable: mocks.connectMyTimetable,
  replaceMyTimetableLink: mocks.replaceMyTimetableLink,
}))

import { ConnectionsSettings } from '@/components/connections/ConnectionsSettings'

const connection: Integration = {
  id: 'calendar-1',
  provider_id: 'campus_calendar',
  enabled: true,
  advertised_capabilities: ['events_read', 'sync_status'],
  effective_capabilities: ['events_read'],
  auth_type: 'oauth',
  connection_status: 'rate_limited',
  sync_status: 'failed',
  last_attempted_at: '2026-09-21T10:00:00.000Z',
  last_successful_sync_at: '2026-09-20T10:00:00.000Z',
  next_allowed_sync_at: '2026-09-21T11:00:00.000Z',
  last_sync_error_code: 'rate_limited',
  last_sync_error_message: 'Token token-example123456 at https://private.example failed.',
  rate_limit_remaining: 0,
  retry_after_at: '2026-09-21T11:00:00.000Z',
}

function renderWith(overrides = {}) {
  mocks.useConnections.mockReturnValue({
    connections: [connection],
    loading: false,
    error: null,
    updatingId: null,
    isTauri: true,
    load: mocks.load,
    setEnabled: mocks.setEnabled,
    refresh: mocks.refresh,
    disconnectCalendar: mocks.disconnectCalendar,
    removeUnsupportedCalendar: mocks.removeUnsupportedCalendar,
    ...overrides,
  })
  return render(<ConnectionsSettings />)
}

describe('Connections settings', () => {
  beforeEach(() => {
    mocks.setEnabled.mockReset().mockResolvedValue(undefined)
    mocks.load.mockReset()
    mocks.refresh.mockReset()
    mocks.disconnectCalendar.mockReset()
    mocks.removeUnsupportedCalendar.mockReset()
    const validation = {
      usable: true,
      event_count: 1,
      error_code: null,
      display_name: null,
      covered_start: null,
      covered_end: null,
      public_host: 'calendar.example',
      warnings: [],
    }
    mocks.validateMyTimetable.mockReset().mockResolvedValue(validation)
    mocks.connectMyTimetable.mockReset().mockResolvedValue(undefined)
    mocks.replaceMyTimetableLink.mockReset().mockResolvedValue(undefined)
  })

  it('presents only persisted metadata, with safe errors and capability distinction', () => {
    renderWith()
    expect(screen.getByRole('heading', { name: 'Campus Calendar' })).toBeInTheDocument()
    expect(screen.getByText('Rate limited')).toBeInTheDocument()
    expect(screen.getByRole('checkbox', { name: 'Enabled' })).toBeChecked()
    expect(screen.getByText('Advertised capabilities')).toBeInTheDocument()
    expect(screen.getByText('Effective capabilities')).toBeInTheDocument()
    expect(screen.getByText('Requests remaining')).toBeInTheDocument()
    expect(screen.getByText(/\[redacted\]/)).toBeInTheDocument()
    expect(screen.queryByText(/private\.example|token-example/)).not.toBeInTheDocument()
  })

  it('uses persisted enabled state and offers the MyTimetable calendar setup', async () => {
    const user = userEvent.setup()
    renderWith()
    await user.click(screen.getByRole('checkbox', { name: 'Enabled' }))
    expect(mocks.setEnabled).toHaveBeenCalledWith(connection, false)
    await user.click(screen.getAllByRole('button', { name: 'Connect calendar' })[0])
    expect(
      screen.getByRole('dialog', { name: 'Connect MyTimetable' }),
    ).toBeInTheDocument()
    expect(screen.getByText(/never shown again/i)).toBeInTheDocument()
  })

  it('renders an accessible unavailable state and inert catalog when no record exists', () => {
    renderWith({ connections: [], error: 'Local storage is unavailable.' })
    expect(screen.getByRole('alert')).toHaveTextContent('Connections are unavailable')
    expect(screen.getByRole('button', { name: 'Try again' })).toBeInTheDocument()
    expect(screen.getByText('Available connections')).toBeInTheDocument()
    expect(screen.getAllByText('Setup unavailable')).toHaveLength(2)
  })

  it('receives focus when opened from the Connections command', () => {
    window.history.replaceState({}, '', '#connections')
    renderWith()
    expect(document.activeElement).toBe(screen.getByLabelText('Connected services'))
    window.history.replaceState({}, '', '/')
  })

  it('shows a truthful disabled MyTimetable state with management controls but never a saved link', () => {
    const timetable = {
      ...connection,
      provider_id: 'my_timetable',
      enabled: false,
      connection_status: 'degraded' as const,
      sync_status: 'failed' as const,
      last_sync_error_message: 'https://calendar.example/private?bearer=secret',
    }
    renderWith({ connections: [timetable] })
    expect(screen.getByText('Disabled')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Refresh now' })).toBeInTheDocument()
    expect(
      screen.getByRole('button', { name: 'Replace subscription link' }),
    ).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Disconnect' })).toBeInTheDocument()
    expect(screen.queryByText(/calendar\.example|bearer=secret/)).not.toBeInTheDocument()
  })

  it('clears the subscription input when setup is cancelled', async () => {
    const user = userEvent.setup()
    renderWith({ connections: [] })
    await user.click(screen.getAllByRole('button', { name: 'Connect calendar' })[0])
    const input = screen.getByRole('textbox', { name: 'Calendar subscription link' })
    await user.type(input, 'https://calendar.example/private')
    await user.click(screen.getByRole('button', { name: 'Cancel' }))
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
    await user.click(screen.getAllByRole('button', { name: 'Connect calendar' })[0])
    expect(
      screen.getByRole('textbox', { name: 'Calendar subscription link' }),
    ).toHaveValue('')
  })

  it('validates and replaces without retaining the link after success or failure', async () => {
    const user = userEvent.setup()
    const timetable = { ...connection, provider_id: 'my_timetable' }
    renderWith({ connections: [timetable] })
    await user.click(screen.getByRole('button', { name: 'Replace subscription link' }))
    const input = screen.getByRole('textbox', { name: 'Calendar subscription link' })
    await user.type(input, 'https://calendar.example/new?token=private')
    await user.click(screen.getByRole('button', { name: 'Validate calendar link' }))
    expect(mocks.validateMyTimetable).toHaveBeenCalledWith(
      'https://calendar.example/new?token=private',
    )
    await user.click(
      within(screen.getByRole('dialog')).getByRole('button', {
        name: 'Replace subscription link',
      }),
    )
    expect(mocks.replaceMyTimetableLink).toHaveBeenCalledWith(
      timetable.id,
      'https://calendar.example/new?token=private',
    )
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: 'Replace subscription link' }))
    mocks.validateMyTimetable.mockRejectedValueOnce(new Error('bad link'))
    const retry = screen.getByRole('textbox', { name: 'Calendar subscription link' })
    await user.type(retry, 'https://calendar.example/bad?token=private')
    await user.click(screen.getByRole('button', { name: 'Validate calendar link' }))
    expect(screen.getByRole('alert')).toHaveTextContent('not saved')
    expect(retry).toHaveValue('')
    expect(screen.queryByText(/calendar\.example|token=private/)).not.toBeInTheDocument()
  })

  it('routes refresh and confirmed disconnect through the connection actions', async () => {
    const user = userEvent.setup()
    const confirm = vi.spyOn(window, 'confirm').mockReturnValue(true)
    const timetable = { ...connection, provider_id: 'my_timetable' }
    renderWith({ connections: [timetable] })
    await user.click(screen.getByRole('button', { name: 'Refresh now' }))
    expect(mocks.refresh).toHaveBeenCalledWith(timetable)
    await user.click(screen.getByRole('button', { name: 'Disconnect' }))
    expect(confirm).toHaveBeenCalled()
    expect(mocks.disconnectCalendar).toHaveBeenCalledWith(timetable)
    confirm.mockRestore()
  })

  it('keeps keyboard focus on the newly opened replacement dialog control', async () => {
    const user = userEvent.setup()
    const timetable = { ...connection, provider_id: 'my_timetable' }
    renderWith({ connections: [timetable] })
    await user.tab()
    await user.click(screen.getByRole('button', { name: 'Replace subscription link' }))
    expect(screen.getByRole('dialog')).toBeInTheDocument()
    expect(
      screen.getByRole('textbox', { name: 'Calendar subscription link' }),
    ).toHaveFocus()
  })

  it('shows the validating transition while the private link check is in flight', async () => {
    const user = userEvent.setup()
    let resolveValidation: (value: IcsValidation) => void = () => undefined
    mocks.validateMyTimetable.mockReturnValueOnce(
      new Promise((resolve) => {
        resolveValidation = resolve
      }),
    )
    renderWith({ connections: [] })
    await user.click(screen.getAllByRole('button', { name: 'Connect calendar' })[0])
    await user.type(
      screen.getByRole('textbox', { name: 'Calendar subscription link' }),
      'https://calendar.example/feed.ics',
    )
    await user.click(screen.getByRole('button', { name: 'Validate calendar link' }))
    expect(screen.getByRole('button', { name: 'Validating…' })).toBeDisabled()
    resolveValidation({
      usable: true,
      event_count: 0,
      error_code: null,
      display_name: null,
      covered_start: null,
      covered_end: null,
      public_host: null,
      warnings: ['Calendar contains no events'],
    })
    expect(await screen.findByText('Calendar link validated')).toBeInTheDocument()
  })

  it('shows a distinct syncing state while a manual refresh is active', () => {
    const timetable = {
      ...connection,
      provider_id: 'my_timetable',
      connection_status: 'syncing' as const,
      sync_status: 'syncing' as const,
    }
    renderWith({ connections: [timetable], updatingId: timetable.id })
    expect(screen.getByText('Syncing')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Refresh now' })).toBeDisabled()
  })

  it('does not advertise Brightspace and offers generic cleanup for a legacy row', async () => {
    const user = userEvent.setup()
    const confirm = vi.spyOn(window, 'confirm').mockReturnValue(true)
    const legacy = {
      ...connection,
      provider_id: 'brightspace',
      auth_type: 'ics_feed' as const,
      connection_status: 'unsupported' as const,
    }
    renderWith({ connections: [legacy] })

    expect(
      screen.queryByRole('button', { name: 'Connect Brightspace' }),
    ).not.toBeInTheDocument()
    expect(screen.getAllByRole('button', { name: 'Connect calendar' })).toHaveLength(1)
    expect(screen.queryByRole('button', { name: 'Refresh now' })).not.toBeInTheDocument()
    expect(screen.queryByText('Advertised capabilities')).not.toBeInTheDocument()
    expect(screen.queryByRole('checkbox', { name: 'Enabled' })).not.toBeInTheDocument()
    expect(screen.getByText(/no longer supported/i)).toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: 'Remove connection' }))
    expect(confirm).toHaveBeenCalled()
    expect(mocks.removeUnsupportedCalendar).toHaveBeenCalledWith(legacy)
    confirm.mockRestore()
  })
})
