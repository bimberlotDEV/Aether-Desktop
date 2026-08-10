import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({
  getNativeStatus: vi.fn(),
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
  })

  it('reports native readiness and sends a fixed test notification', async () => {
    const user = userEvent.setup()
    render(<NativeSettings />)
    expect(
      await screen.findByText(/Ctrl\+Shift\+Space shows and focuses Aether/),
    ).toBeInTheDocument()
    expect(screen.getByText(/Updates are intentionally disabled/)).toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: 'Send test' }))
    expect(mocks.sendTestNotification).toHaveBeenCalledOnce()
    expect(await screen.findByRole('status')).toHaveTextContent('Test notification sent.')
  })
})
