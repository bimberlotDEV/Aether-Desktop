import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({
  save: vi.fn(),
  exportWorkspaceBackup: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({ save: mocks.save }))
vi.mock('@/lib/db/tauri', () => ({
  exportWorkspaceBackup: mocks.exportWorkspaceBackup,
}))

import { BackupSettings } from '@/components/BackupSettings'

describe('Backup settings', () => {
  beforeEach(() => {
    Object.defineProperty(window, '__TAURI_INTERNALS__', {
      value: {},
      configurable: true,
    })
    mocks.save.mockReset().mockResolvedValue('C:\\Backups\\Aether.aether-backup.db')
    mocks.exportWorkspaceBackup.mockReset().mockResolvedValue({
      sizeBytes: 1536,
      createdAt: '2026-08-10T12:00:00Z',
    })
  })

  it('exports to the selected path and explains exclusions', async () => {
    const user = userEvent.setup()
    render(<BackupSettings />)

    expect(screen.getByText(/API credentials/)).toBeInTheDocument()
    expect(
      screen.getByText(/contents of managed or linked Vault files/),
    ).toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: 'Choose location and export' }))

    expect(mocks.exportWorkspaceBackup).toHaveBeenCalledWith(
      'C:\\Backups\\Aether.aether-backup.db',
    )
    expect(await screen.findByRole('status')).toHaveTextContent('Backup saved (1.5 KB).')
  })

  it('does not invoke an export when the save dialog is cancelled', async () => {
    mocks.save.mockResolvedValue(null)
    const user = userEvent.setup()
    render(<BackupSettings />)
    await user.click(screen.getByRole('button', { name: 'Choose location and export' }))
    expect(mocks.exportWorkspaceBackup).not.toHaveBeenCalled()
  })
})
