import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { AiTaskProposal } from '@/components/ai/AiTaskProposal'

const mocks = vi.hoisted(() => ({ createTasksBatch: vi.fn() }))

vi.mock('@/lib/db/tauri', () => ({ createTasksBatch: mocks.createTasksBatch }))

describe('AI Task proposal review', () => {
  beforeEach(() => mocks.createTasksBatch.mockReset().mockResolvedValue([]))

  it('creates only selected Tasks in one Space-scoped batch', async () => {
    const user = userEvent.setup()
    const onCreated = vi.fn()
    render(
      <AiTaskProposal
        spaceId="space-1"
        tasks={[
          {
            title: 'First Task',
            description: 'Keep this one',
            priority: 'high',
            dueDate: '2026-08-12',
            tags: ['release', 'release'],
          },
          {
            title: 'Second Task',
            description: '',
            priority: 'none',
            dueDate: null,
            tags: [],
          },
        ]}
        onClose={vi.fn()}
        onCreated={onCreated}
      />,
    )

    const checkboxes = screen.getAllByRole('checkbox')
    await user.click(checkboxes[1])
    await user.click(screen.getByRole('button', { name: 'Create 1 Task' }))

    expect(mocks.createTasksBatch).toHaveBeenCalledWith([
      {
        spaceId: 'space-1',
        parentTaskId: null,
        title: 'First Task',
        description: 'Keep this one',
        status: 'inbox',
        priority: 'high',
        dueDate: '2026-08-12',
        tags: ['release'],
      },
    ])
    expect(onCreated).toHaveBeenCalledOnce()
  })
})
