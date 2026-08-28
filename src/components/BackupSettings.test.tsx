import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({
  open: vi.fn(),
  save: vi.fn(),
  exportWorkspaceArchive: vi.fn(),
  previewWorkspaceRestore: vi.fn(),
  cancelWorkspaceRestore: vi.fn(),
  approveWorkspaceRestore: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({ open: mocks.open, save: mocks.save }))
vi.mock('@/lib/db/tauri', () => ({
  exportWorkspaceArchive: mocks.exportWorkspaceArchive,
  previewWorkspaceRestore: mocks.previewWorkspaceRestore,
  cancelWorkspaceRestore: mocks.cancelWorkspaceRestore,
  approveWorkspaceRestore: mocks.approveWorkspaceRestore,
}))

import { BackupSettings } from '@/components/BackupSettings'

const preview = {
  token: 'restore-token',
  createdAt: '2026-08-28T12:00:00Z',
  appVersion: '0.4.0',
  archiveSizeBytes: 4096,
  managedFileCount: 2,
  linkedFileCount: 1,
  expiresAt: '2026-08-28T12:10:00Z',
  counts: { spaces: 3, notes: 4, tasks: 5, memories: 2, conversations: 6 },
}

describe('Backup settings', () => {
  beforeEach(() => {
    Object.defineProperty(window, '__TAURI_INTERNALS__', {
      value: {},
      configurable: true,
    })
    mocks.open.mockReset().mockResolvedValue('C:\\Backups\\Aether.aether-backup')
    mocks.save.mockReset().mockResolvedValue('C:\\Backups\\Aether.aether-backup')
    mocks.exportWorkspaceArchive.mockReset().mockResolvedValue({
      sizeBytes: 1536,
      createdAt: '2026-08-28T12:00:00Z',
      managedFileCount: 2,
      linkedFileCount: 1,
    })
    mocks.previewWorkspaceRestore.mockReset().mockResolvedValue(preview)
    mocks.cancelWorkspaceRestore.mockReset().mockResolvedValue(true)
    mocks.approveWorkspaceRestore.mockReset().mockResolvedValue(undefined)
  })

  it('exports a complete archive and honestly explains exclusions', async () => {
    const user = userEvent.setup()
    render(<BackupSettings />)

    expect(screen.getByText(/API credentials and linked files/)).toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: 'Create complete backup' }))

    expect(mocks.exportWorkspaceArchive).toHaveBeenCalledWith(
      'C:\\Backups\\Aether.aether-backup',
    )
    expect(await screen.findByRole('status')).toHaveTextContent(
      'Complete backup saved (1.5 KB, 2 managed files).',
    )
  })

  it('previews replacement details and approves only through the opaque token', async () => {
    const user = userEvent.setup()
    render(<BackupSettings />)

    await user.click(screen.getByRole('button', { name: 'Restore from backup' }))
    const dialog = await screen.findByRole('alertdialog')
    expect(dialog).toHaveTextContent('replace—not merge')
    expect(dialog).toHaveTextContent('3 Spaces, 4 Notes, 5 Tasks')
    expect(dialog).toHaveTextContent('1 linked file')

    await user.click(
      screen.getByRole('button', { name: 'Create safety backup & restore' }),
    )
    expect(mocks.approveWorkspaceRestore).toHaveBeenCalledWith('restore-token')
    expect(mocks.cancelWorkspaceRestore).not.toHaveBeenCalled()
  })

  it('invalidates a preview when restore is cancelled', async () => {
    const user = userEvent.setup()
    render(<BackupSettings />)
    await user.click(screen.getByRole('button', { name: 'Restore from backup' }))
    await screen.findByRole('alertdialog')
    await user.click(screen.getByRole('button', { name: 'Cancel' }))
    expect(mocks.cancelWorkspaceRestore).toHaveBeenCalledWith('restore-token')
    expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument()
  })

  it('does not invoke native work when dialogs are cancelled', async () => {
    mocks.open.mockResolvedValue(null)
    mocks.save.mockResolvedValue(null)
    const user = userEvent.setup()
    render(<BackupSettings />)
    await user.click(screen.getByRole('button', { name: 'Create complete backup' }))
    await user.click(screen.getByRole('button', { name: 'Restore from backup' }))
    expect(mocks.exportWorkspaceArchive).not.toHaveBeenCalled()
    expect(mocks.previewWorkspaceRestore).not.toHaveBeenCalled()
  })
})
