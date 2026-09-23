import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { Integration } from '@/lib/db/types'

const mocks = vi.hoisted(() => ({
  useConnections: vi.fn(),
  setEnabled: vi.fn(),
  load: vi.fn(),
  refresh: vi.fn(),
  disconnectMyTimetable: vi.fn(),
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
  sync_modes: ['manual'],
  sync_config: {},
  connection_status: 'rate_limited',
  sync_status: 'failed',
  disconnect_reason: null,
  last_attempted_at: '2026-09-21T10:00:00.000Z',
  last_successful_sync_at: '2026-09-20T10:00:00.000Z',
  next_allowed_sync_at: '2026-09-21T11:00:00.000Z',
  last_sync_error_code: 'rate_limited',
  last_sync_error_message: 'Token token-example123456 at https://private.example failed.',
  last_sync_etag: null,
  last_sync_last_modified: null,
  sync_cursor: null,
  rate_limit_remaining: 0,
  retry_after_at: '2026-09-21T11:00:00.000Z',
  credential_expires_at: null,
  credential_rotated_at: null,
  sync_execution_scope: 'desktop_running',
  created_at: '2026-09-20T10:00:00.000Z',
  updated_at: '2026-09-21T10:00:00.000Z',
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
    disconnectMyTimetable: mocks.disconnectMyTimetable,
    ...overrides,
  })
  return render(<ConnectionsSettings />)
}

describe('Connections settings', () => {
  beforeEach(() => {
    mocks.setEnabled.mockReset().mockResolvedValue(undefined)
    mocks.load.mockReset()
    mocks.refresh.mockReset()
    mocks.disconnectMyTimetable.mockReset()
    mocks.validateMyTimetable.mockReset().mockResolvedValue({ usable: true })
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

  it('uses the persisted enabled-state action and offers setup only for MyTimetable', async () => {
    const user = userEvent.setup()
    renderWith()
    await user.click(screen.getByRole('checkbox', { name: 'Enabled' }))
    expect(mocks.setEnabled).toHaveBeenCalledWith(connection, false)
    await user.click(screen.getByRole('button', { name: 'Connect calendar' }))
    expect(
      screen.getByRole('dialog', { name: 'Connect MyTimetable' }),
    ).toBeInTheDocument()
    expect(screen.getByText(/never shown again/i)).toBeInTheDocument()
  })

  it('renders an accessible unavailable state and inert catalog when no record exists', () => {
    renderWith({ connections: [], error: 'Local storage is unavailable.' })
    expect(screen.getByRole('alert')).toHaveTextContent('Connections are unavailable')
    expect(screen.getByRole('button', { name: 'Try again' })).toBeInTheDocument()
    expect(screen.getByText('Not available to set up yet')).toBeInTheDocument()
    expect(screen.getAllByText('Setup unavailable')).toHaveLength(3)
  })

  it('receives focus when opened from the Connections command', () => {
    window.history.replaceState({}, '', '#connections')
    renderWith()
    expect(document.activeElement).toBe(screen.getByLabelText('Connected services'))
    window.history.replaceState({}, '', '/')
  })

  it('shows a truthful disabled MyTimetable state with management controls but never a saved link', () => {
    const timetable = { ...connection, provider_id: 'my_timetable', enabled: false, connection_status: 'degraded' as const, sync_status: 'failed' as const, last_sync_error_message: 'https://calendar.example/private?bearer=secret' }
    renderWith({ connections: [timetable] })
    expect(screen.getByText('Disabled')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Refresh now' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Replace subscription link' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Disconnect' })).toBeInTheDocument()
    expect(screen.queryByText(/calendar\.example|bearer=secret/)).not.toBeInTheDocument()
  })

  it('clears the subscription input when setup is cancelled', async () => {
    const user = userEvent.setup()
    renderWith({ connections: [] })
    await user.click(screen.getByRole('button', { name: 'Connect calendar' }))
    const input = screen.getByRole('textbox', { name: 'Calendar subscription link' })
    await user.type(input, 'https://calendar.example/private')
    await user.click(screen.getByRole('button', { name: 'Cancel' }))
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: 'Connect calendar' }))
    expect(screen.getByRole('textbox', { name: 'Calendar subscription link' })).toHaveValue('')
  })

  it('validates and replaces without retaining the link after success or failure', async () => {
    const user = userEvent.setup()
    const timetable = { ...connection, provider_id: 'my_timetable' }
    renderWith({ connections: [timetable] })
    await user.click(screen.getByRole('button', { name: 'Replace subscription link' }))
    const input = screen.getByRole('textbox', { name: 'Calendar subscription link' })
    await user.type(input, 'https://calendar.example/new?token=private')
    await user.click(screen.getByRole('button', { name: 'Validate and replace' }))
    expect(mocks.validateMyTimetable).toHaveBeenCalledWith(
      'https://calendar.example/new?token=private',
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
    await user.click(screen.getByRole('button', { name: 'Validate and replace' }))
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
    expect(mocks.disconnectMyTimetable).toHaveBeenCalledWith(timetable)
    confirm.mockRestore()
  })

  it('keeps keyboard focus on the newly opened replacement dialog control', async () => {
    const user = userEvent.setup()
    const timetable = { ...connection, provider_id: 'my_timetable' }
    renderWith({ connections: [timetable] })
    await user.tab()
    await user.click(screen.getByRole('button', { name: 'Replace subscription link' }))
    expect(screen.getByRole('dialog')).toBeInTheDocument()
    expect(screen.getByRole('textbox', { name: 'Calendar subscription link' })).toHaveFocus()
  })

  it('shows the validating transition while the private link check is in flight', async () => {
    const user = userEvent.setup()
    let resolveValidation: (value: { usable: boolean }) => void = () => undefined
    mocks.validateMyTimetable.mockReturnValueOnce(
      new Promise((resolve) => {
        resolveValidation = resolve
      }),
    )
    renderWith({ connections: [] })
    await user.click(screen.getByRole('button', { name: 'Connect calendar' }))
    await user.type(
      screen.getByRole('textbox', { name: 'Calendar subscription link' }),
      'https://calendar.example/feed.ics',
    )
    await user.click(screen.getByRole('button', { name: 'Validate and connect' }))
    expect(screen.getByRole('button', { name: 'Validating…' })).toBeDisabled()
    resolveValidation({ usable: true })
    await screen.findByText('No connections configured')
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
})
