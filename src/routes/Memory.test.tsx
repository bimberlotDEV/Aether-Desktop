import { fireEvent, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { MemoryItem, Space } from '@/lib/db/types'

const mocks = vi.hoisted(() => ({
  create: vi.fn(),
  update: vi.fn(),
  remove: vi.fn(),
  useMemory: vi.fn(),
}))

const space: Space = {
  id: 'space-1',
  name: 'Aether',
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
const item: MemoryItem = {
  id: 'memory-1',
  space_id: 'space-1',
  title: 'Release constraint',
  content: 'Windows first',
  reason: 'Keeps planning aligned',
  category: 'constraint',
  source: 'user',
  created_at: '2026-08-10T00:00:00Z',
  updated_at: '2026-08-10T00:00:00Z',
}

vi.mock('@/hooks/useMemory', () => ({
  useMemory: (filter: unknown) => mocks.useMemory(filter),
}))

import { MemoryView } from '@/components/memory/MemoryView'

describe('Memory', () => {
  beforeEach(() => {
    mocks.create.mockReset().mockResolvedValue(undefined)
    mocks.update.mockReset().mockResolvedValue(undefined)
    mocks.remove.mockReset().mockResolvedValue(undefined)
    mocks.useMemory.mockReset().mockReturnValue({
      items: [item],
      loading: false,
      error: null,
      isTauri: true,
      create: mocks.create,
      update: mocks.update,
      remove: mocks.remove,
    })
  })

  it('creates explicit Space Memory with attribution and reason', async () => {
    const user = userEvent.setup()
    render(<MemoryView spaceId={space.id} spaces={[space]} />)
    await user.click(screen.getByRole('button', { name: 'Remember' }))
    fireEvent.change(screen.getByLabelText('Title'), {
      target: { value: 'Preferred tone' },
    })
    fireEvent.change(screen.getByLabelText('What should Aether remember?'), {
      target: { value: 'Keep answers concise' },
    })
    fireEvent.change(screen.getByLabelText('Why is this useful?'), {
      target: { value: 'Makes reviews faster' },
    })
    await user.selectOptions(screen.getByLabelText('Category'), 'preference')
    expect(
      screen.getByText(/never turns chats into permanent Memory automatically/i),
    ).toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: 'Save Memory' }))
    expect(mocks.create).toHaveBeenCalledWith({
      spaceId: 'space-1',
      title: 'Preferred tone',
      content: 'Keep answers concise',
      reason: 'Makes reviews faster',
      category: 'preference',
    })
  })

  it('edits and confirms permanent deletion', async () => {
    const user = userEvent.setup()
    render(<MemoryView spaces={[space]} />)
    await user.click(screen.getByRole('button', { name: 'Edit Release constraint' }))
    await user.clear(screen.getByLabelText('Title'))
    await user.type(screen.getByLabelText('Title'), 'Updated constraint')
    await user.click(screen.getByRole('button', { name: 'Save Memory' }))
    expect(mocks.update).toHaveBeenCalledWith(
      'memory-1',
      expect.objectContaining({ title: 'Updated constraint' }),
    )

    await user.click(screen.getByRole('button', { name: 'Delete Release constraint' }))
    await user.click(screen.getByRole('button', { name: 'Delete Memory' }))
    expect(mocks.remove).toHaveBeenCalledWith('memory-1')
  })
})
