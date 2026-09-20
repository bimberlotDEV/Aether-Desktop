import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({
  getNativeStatus: vi.fn(),
  getUpdateStatus: vi.fn(),
  checkForUpdate: vi.fn(),
  cancelUpdate: vi.fn(),
  installUpdate: vi.fn(),
  sendTestNotification: vi.fn(),
}))

vi.mock('@/lib/db/tauri', () => mocks)

import { NativeSettings } from '@/components/NativeSettings'

describe('Native desktop settings', () => {
  beforeEach(() => {
    Object.defineProperty(window, '__TAURI_INTERNALS__', {
      value: {},
      configurable: true,
    })
    mocks.getNativeStatus.mockReset().mockResolvedValue({
      trayAvailable: true,
      shortcut: 'Ctrl+Shift+Space',
      shortcutRegistered: true,
      notificationsAvailable: true,
      updaterConfigured: false,
    })
    mocks.sendTestNotification.mockReset().mockResolvedValue(undefined)
    mocks.getUpdateStatus.mockReset().mockResolvedValue({
      configured: false,
      channel: 'Stable',
      currentVersion: '0.5.0',
      phase: 'idle',
    })
    mocks.checkForUpdate.mockReset().mockResolvedValue(null)
    mocks.cancelUpdate.mockReset().mockResolvedValue(true)
    mocks.installUpdate.mockReset().mockResolvedValue(undefined)
  })

  it('reports native readiness and sends a fixed test notification', async () => {
    const user = userEvent.setup()
    render(<NativeSettings />)
    expect(
      await screen.findByText(/Ctrl\+Shift\+Space shows and focuses Aether/),
    ).toBeInTheDocument()
    expect(screen.getByText(/no release trust key/)).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Check for updates' })).not.toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: 'Send test' }))
    expect(mocks.sendTestNotification).toHaveBeenCalledOnce()
    expect(await screen.findByRole('status')).toHaveTextContent('Test notification sent.')
  })

  it('shows signed release details before installing through the opaque token', async () => {
    mocks.getUpdateStatus.mockResolvedValue({
      configured: true,
      channel: 'Stable',
      currentVersion: '0.5.0',
      phase: 'idle',
    })
    mocks.checkForUpdate.mockResolvedValue({
      token: 'update-token',
      currentVersion: '0.5.0',
      version: '0.5.1',
      notes: 'Security and reliability improvements.',
      publishedAt: '2026-08-28T12:00:00Z',
      expiresAt: '2026-08-28T12:10:00Z',
    })
    const user = userEvent.setup()
    render(<NativeSettings />)

    await user.click(await screen.findByRole('button', { name: 'Check for updates' }))
    expect(await screen.findByText('Aether 0.5.1 is available')).toBeInTheDocument()
    expect(screen.getByText('Security and reliability improvements.')).toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: 'Download, verify & install' }))

    expect(mocks.installUpdate).toHaveBeenCalledWith('update-token', expect.any(Function))
    expect(mocks.installUpdate).toHaveBeenCalledOnce()
  })

  it('dismisses a reviewed update without installing it', async () => {
    mocks.getUpdateStatus.mockResolvedValue({
      configured: true,
      channel: 'Stable',
      currentVersion: '0.5.0',
      phase: 'idle',
    })
    mocks.checkForUpdate.mockResolvedValue({
      token: 'dismiss-token',
      currentVersion: '0.5.0',
      version: '0.5.1',
      notes: null,
      publishedAt: null,
      expiresAt: '2026-08-28T12:10:00Z',
    })
    const user = userEvent.setup()
    render(<NativeSettings />)
    await user.click(await screen.findByRole('button', { name: 'Check for updates' }))
    await screen.findByText('Aether 0.5.1 is available')
    await user.click(screen.getByRole('button', { name: 'Dismiss' }))

    expect(mocks.cancelUpdate).toHaveBeenCalledWith('dismiss-token')
    expect(mocks.installUpdate).not.toHaveBeenCalled()
    expect(screen.queryByText('Aether 0.5.1 is available')).not.toBeInTheDocument()
  })
})
