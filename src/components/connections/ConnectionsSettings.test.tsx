import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { Integration } from '@/lib/db/types'

const mocks = vi.hoisted(() => ({
  useConnections: vi.fn(),
  setEnabled: vi.fn(),
  load: vi.fn(),
}))

vi.mock('@/hooks/useConnections', () => ({ useConnections: mocks.useConnections }))

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

describe('Connections settings', () => {
  beforeEach(() => {
    mocks.setEnabled.mockReset().mockResolvedValue(undefined)
    mocks.load.mockReset()
  })

  it('shows only persisted connection state and redacts sensitive error fragments', () => {
    mocks.useConnections.mockReturnValue({
      connections: [connection],
      loading: false,
      error: null,
      updatingId: null,
      isTauri: true,
      load: mocks.load,
      setEnabled: mocks.setEnabled,
    })
    render(<ConnectionsSettings />)

    expect(screen.getByRole('heading', { name: 'Campus Calendar' })).toBeInTheDocument()
    expect(screen.getByText('Rate limited')).toBeInTheDocument()
    expect(screen.getByRole('checkbox', { name: 'Enabled' })).toBeChecked()
    expect(screen.getAllByText('events read')).toHaveLength(2)
    expect(screen.getByText('Advertised capabilities')).toBeInTheDocument()
    expect(screen.getByText('Effective capabilities')).toBeInTheDocument()
    expect(screen.getByText('failed')).toBeInTheDocument()
    expect(screen.getByText(/\[redacted\]/)).toBeInTheDocument()
    expect(screen.queryByText(/private\.example|token-example/)).not.toBeInTheDocument()
    expect(
      screen.getByText(/not available until their provider support is installed/i),
    ).toBeInTheDocument()
  })

  it('uses the generic enabled-state action without changing connection status', async () => {
    const user = userEvent.setup()
    mocks.useConnections.mockReturnValue({
      connections: [connection],
      loading: false,
      error: null,
      updatingId: null,
      isTauri: true,
      load: mocks.load,
      setEnabled: mocks.setEnabled,
    })
    render(<ConnectionsSettings />)

    await user.click(screen.getByRole('checkbox', { name: 'Enabled' }))
    expect(mocks.setEnabled).toHaveBeenCalledWith(connection, false)
  })

  it('uses a truthful empty state when no provider is configured', () => {
    mocks.useConnections.mockReturnValue({
      connections: [],
      loading: false,
      error: null,
      updatingId: null,
      isTauri: true,
      load: mocks.load,
      setEnabled: mocks.setEnabled,
    })
    render(<ConnectionsSettings />)
    expect(
      screen.getByRole('heading', { name: 'No connections configured' }),
    ).toBeInTheDocument()
    expect(screen.getByText(/only real connections/i)).toBeInTheDocument()
  })
})
