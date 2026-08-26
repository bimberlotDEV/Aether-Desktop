import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({
  open: vi.fn(),
  listSources: vi.fn(),
  createSource: vi.fn(),
  scanSource: vi.fn(),
  listIndexedFiles: vi.fn(),
  updateSourceSpace: vi.fn(),
  revokeSource: vi.fn(),
}))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: mocks.open }))
vi.mock('@/lib/db/tauri', () => mocks)
vi.mock('@/hooks/useSpaces', () => ({ useSpaces: () => ({ spaces: [] }) }))
import { Sources } from '@/routes/Sources'

const source = {
  id: 'source-1',
  rootPath: 'C:\\Work',
  displayName: 'Work',
  spaceId: null,
  scanStatus: 'complete',
  lastScanAt: '2026-08-25T20:00:00Z',
  lastError: null,
  createdAt: '2026-08-25T20:00:00Z',
  updatedAt: '2026-08-25T20:00:00Z',
}

describe('Sources route', () => {
  beforeEach(() => {
    Object.defineProperty(window, '__TAURI_INTERNALS__', {
      value: {},
      configurable: true,
    })
    vi.spyOn(window, 'confirm').mockReturnValue(true)
    for (const mock of Object.values(mocks)) mock.mockReset()
    mocks.open.mockResolvedValue('C:\\Work')
    mocks.listSources.mockResolvedValue([])
    mocks.createSource.mockResolvedValue(source)
    mocks.scanSource.mockResolvedValue({
      sourceId: 'source-1',
      scanned: 1,
      added: 1,
      changed: 0,
      renamed: 0,
      removed: 0,
      unchanged: 0,
      skipped: 0,
      errors: 0,
      truncated: false,
      completedAt: '2026-08-25T20:00:00Z',
    })
    mocks.listIndexedFiles.mockResolvedValue([])
    mocks.updateSourceSpace.mockResolvedValue(source)
    mocks.revokeSource.mockResolvedValue(true)
  })

  it('authorizes only the selected folder and scans it through narrow commands', async () => {
    const user = userEvent.setup()
    render(<Sources />)
    await user.click(await screen.findByRole('button', { name: 'Choose a folder' }))
    expect(mocks.createSource).toHaveBeenCalledWith({
      rootPath: 'C:\\Work',
      displayName: 'Work',
      spaceId: null,
    })
    expect(mocks.scanSource).toHaveBeenCalledWith('source-1')
    expect(await screen.findByRole('status')).toHaveTextContent('Scan complete')
  })

  it('requires confirmation and explains that revocation does not change files', async () => {
    mocks.listSources.mockResolvedValue([source])
    const user = userEvent.setup()
    render(<Sources />)
    await user.click(await screen.findByRole('button', { name: 'Revoke Work' }))
    expect(window.confirm).toHaveBeenCalledWith(
      expect.stringMatching(/Files in the folder will not be changed/),
    )
    await waitFor(() => expect(mocks.revokeSource).toHaveBeenCalledWith('source-1'))
  })

  it('does not pretend to index in browser mode', () => {
    Reflect.deleteProperty(window, '__TAURI_INTERNALS__')
    render(<Sources />)
    expect(
      screen.getByText('Sources are available in the installed app'),
    ).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Add folder' })).toBeDisabled()
    expect(mocks.listSources).not.toHaveBeenCalled()
  })
})
