import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { Space, VaultItem } from '@/lib/db/types'

const mocks = vi.hoisted(() => ({
  selectFile: vi.fn(),
  importItem: vi.fn(),
  update: vi.fn(),
  remove: vi.fn(),
  openItem: vi.fn(),
  reveal: vi.fn(),
  useVault: vi.fn(),
}))

const space: Space = {
  id: 'space-1',
  name: 'Research',
  description: null,
  icon: null,
  accent: null,
  template_type: 'blank',
  favourite: false,
  archived_at: null,
  sort_order: 0,
  settings_json: null,
  parent_space_id: null,
  last_opened_at: null,
  created_at: '2026-08-10T00:00:00Z',
  updated_at: '2026-08-10T00:00:00Z',
}

const linkedItem: VaultItem = {
  id: 'vault-linked',
  space_id: null,
  storage_mode: 'linked',
  display_title: 'Project brief',
  original_name: 'brief.pdf',
  media_type: 'application/pdf',
  size_bytes: 2048,
  tags: ['project'],
  created_at: '2026-08-10T00:00:00Z',
  updated_at: '2026-08-10T00:00:00Z',
}

const managedItem: VaultItem = {
  ...linkedItem,
  id: 'vault-managed',
  storage_mode: 'managed',
  display_title: 'Managed notes',
  original_name: 'notes.md',
}

let visibleItems: VaultItem[] = []

vi.mock('@tauri-apps/plugin-dialog', () => ({ open: mocks.selectFile }))
vi.mock('@/hooks/useVault', () => ({
  useVaultSpaces: () => ({ spaces: [space], loading: false }),
  useVault: (filter: unknown) => {
    mocks.useVault(filter)
    return {
      items: visibleItems,
      loading: false,
      error: null,
      isTauri: true,
      importItem: mocks.importItem,
      update: mocks.update,
      remove: mocks.remove,
      openItem: mocks.openItem,
      reveal: mocks.reveal,
    }
  },
}))

import { Vault } from '@/routes/Vault'
import { VaultView } from '@/components/vault/VaultView'

describe('Vault route', () => {
  beforeEach(() => {
    visibleItems = []
    for (const mock of Object.values(mocks)) mock.mockReset().mockResolvedValue(undefined)
    mocks.selectFile.mockResolvedValue('C:\\Users\\rawan\\report.pdf')
  })

  it('imports a selected file with explicit managed ownership and metadata', async () => {
    const user = userEvent.setup()
    render(<Vault />)

    await user.click(screen.getByRole('button', { name: 'Add file' }))
    await user.click(screen.getByText('Copy into Vault'))
    await user.clear(screen.getByLabelText('Title'))
    await user.type(screen.getByLabelText('Title'), 'Quarterly report')
    await user.selectOptions(screen.getByLabelText('Space'), space.id)
    await user.type(screen.getByLabelText(/Tags/), 'finance, review')
    await user.click(screen.getByRole('button', { name: 'Add to Vault' }))

    expect(mocks.importItem).toHaveBeenCalledWith({
      path: 'C:\\Users\\rawan\\report.pdf',
      storageMode: 'managed',
      spaceId: space.id,
      displayTitle: 'Quarterly report',
      tags: ['finance', 'review'],
    })
  })

  it('passes search, ownership, and Space filters to the domain hook', async () => {
    const user = userEvent.setup()
    render(<Vault />)

    await user.type(screen.getByLabelText('Search Vault'), 'brief')
    await user.selectOptions(screen.getByLabelText('Filter by storage'), 'linked')
    await user.selectOptions(screen.getByLabelText('Filter by Space'), space.id)

    expect(mocks.useVault).toHaveBeenLastCalledWith({
      search: 'brief',
      storageMode: 'linked',
      spaceId: space.id,
    })
  })

  it('keeps import and filtering locked to an embedded Space', async () => {
    const user = userEvent.setup()
    render(<VaultView spaces={[space]} spaceId={space.id} />)

    expect(mocks.useVault).toHaveBeenLastCalledWith({ spaceId: space.id })
    expect(screen.queryByLabelText('Filter by Space')).not.toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: 'Add file' }))
    expect(screen.queryByLabelText('Space')).not.toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: 'Add to Vault' }))
    expect(mocks.importItem).toHaveBeenCalledWith(
      expect.objectContaining({ spaceId: space.id }),
    )
  })

  it('opens, reveals, and edits an item', async () => {
    visibleItems = [linkedItem]
    const user = userEvent.setup()
    render(<Vault />)

    await user.click(screen.getByRole('button', { name: 'Open Project brief' }))
    expect(mocks.openItem).toHaveBeenCalledWith(linkedItem.id)

    await user.click(screen.getByRole('button', { name: 'Actions for Project brief' }))
    await user.click(screen.getByRole('button', { name: 'Show in folder' }))
    expect(mocks.reveal).toHaveBeenCalledWith(linkedItem.id)

    await user.click(screen.getByRole('button', { name: 'Actions for Project brief' }))
    await user.click(screen.getByRole('button', { name: 'Edit details' }))
    await user.clear(screen.getByLabelText('Title'))
    await user.type(screen.getByLabelText('Title'), 'Updated brief')
    await user.click(screen.getByRole('button', { name: 'Save changes' }))
    expect(mocks.update).toHaveBeenCalledWith(
      linkedItem.id,
      expect.objectContaining({ displayTitle: 'Updated brief' }),
    )
  })

  it('explains the distinct removal consequence for linked and managed files', async () => {
    visibleItems = [linkedItem, managedItem]
    const user = userEvent.setup()
    render(<Vault />)

    await user.click(screen.getByRole('button', { name: 'Actions for Project brief' }))
    await user.click(screen.getByRole('button', { name: 'Remove' }))
    expect(
      screen.getByText(/original file stays exactly where it is/i),
    ).toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: 'Cancel' }))

    await user.click(screen.getByRole('button', { name: 'Actions for Managed notes' }))
    await user.click(screen.getByRole('button', { name: 'Remove' }))
    expect(
      screen.getByText(/permanently deletes Aether’s managed Vault copy/i),
    ).toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: 'Delete managed copy' }))
    expect(mocks.remove).toHaveBeenCalledWith(managedItem.id)
  })
})
