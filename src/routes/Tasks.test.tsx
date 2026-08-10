import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { Task } from '@/lib/db/types'

const mocks = vi.hoisted(() => ({
  create: vi.fn(),
  update: vi.fn(),
  toggleComplete: vi.fn(),
  archive: vi.fn(),
  restore: vi.fn(),
  remove: vi.fn(),
}))

const task: Task = {
  id: 'task-1',
  space_id: null,
  parent_task_id: null,
  title: 'Review release notes',
  description: '',
  status: 'inbox',
  priority: 'high',
  due_date: null,
  tags: ['release'],
  completed_at: null,
  archived_at: null,
  created_at: '2026-08-10T00:00:00Z',
  updated_at: '2026-08-10T00:00:00Z',
}

const archivedTask: Task = {
  ...task,
  id: 'task-archived',
  title: 'Old draft',
  archived_at: '2026-08-10T02:00:00Z',
}

vi.mock('@/hooks/useTasks', () => ({
  useTasks: (filter: { includeArchived?: boolean }) => ({
    tasks: filter.includeArchived ? [task, archivedTask] : [task],
    loading: false,
    error: null,
    load: vi.fn(),
    ...mocks,
  }),
}))

vi.mock('@/hooks/useSpaces', () => ({
  useSpaces: () => ({ spaces: [] }),
}))

import { Tasks } from '@/routes/Tasks'

describe('Tasks route', () => {
  beforeEach(() => {
    for (const mock of Object.values(mocks)) mock.mockReset().mockResolvedValue(true)
  })

  it('quick-captures a global Inbox Task and completes an existing Task', async () => {
    const user = userEvent.setup()
    render(<Tasks />)

    await user.type(screen.getByLabelText('Quick Task title'), 'Book dentist')
    await user.click(screen.getByRole('button', { name: 'Add' }))
    expect(mocks.create).toHaveBeenCalledWith(
      expect.objectContaining({
        spaceId: null,
        title: 'Book dentist',
        status: 'inbox',
      }),
    )

    await user.click(
      screen.getByRole('button', { name: 'Complete Review release notes' }),
    )
    expect(mocks.toggleComplete).toHaveBeenCalledWith(task)
  })

  it('opens the full editor and submits Task metadata', async () => {
    const user = userEvent.setup()
    render(<Tasks />)

    await user.click(screen.getByRole('button', { name: 'New Task' }))
    await user.type(screen.getByLabelText('Title'), 'Plan launch')
    await user.selectOptions(screen.getByLabelText('Priority'), 'high')
    await user.type(screen.getByLabelText(/Tags/), 'launch, focus')
    await user.click(screen.getByRole('button', { name: 'Save Task' }))

    expect(mocks.create).toHaveBeenCalledWith(
      expect.objectContaining({
        title: 'Plan launch',
        priority: 'high',
        tags: ['launch', 'focus'],
      }),
    )
  })

  it('creates a subtask with its parent context', async () => {
    const user = userEvent.setup()
    render(<Tasks />)

    await user.click(
      screen.getByRole('button', { name: 'Add subtask to Review release notes' }),
    )
    await user.type(screen.getByLabelText('Title'), 'Check screenshots')
    await user.click(screen.getByRole('button', { name: 'Save Task' }))

    expect(mocks.create).toHaveBeenCalledWith(
      expect.objectContaining({
        parentTaskId: task.id,
        title: 'Check screenshots',
      }),
    )
  })

  it('restores and permanently deletes archived Tasks through confirmation', async () => {
    const user = userEvent.setup()
    render(<Tasks />)

    await user.click(screen.getByRole('button', { name: 'Archived (1)' }))
    await user.click(screen.getByRole('button', { name: 'Restore Old draft' }))
    expect(mocks.restore).toHaveBeenCalledWith(archivedTask.id)

    await user.click(screen.getByRole('button', { name: 'Delete Old draft permanently' }))
    expect(screen.getByText(/Permanently delete "Old draft"/)).toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: 'Delete permanently' }))
    expect(mocks.remove).toHaveBeenCalledWith(archivedTask.id)
  })
})
