import { act, renderHook, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { Task } from '@/lib/db/types'

const mocks = vi.hoisted(() => {
  Object.defineProperty(window, '__TAURI_INTERNALS__', {
    configurable: true,
    value: {},
  })
  return {
    listTasks: vi.fn(),
    createTask: vi.fn(),
    updateTask: vi.fn(),
    archiveTask: vi.fn(),
    restoreTask: vi.fn(),
    deleteTask: vi.fn(),
    listTaskAttention: vi.fn(),
  }
})

vi.mock('@/lib/db/tauri', () => mocks)

import { useTaskAttention, useTasks } from '@/hooks/useTasks'

const openTask: Task = {
  id: 'task-1',
  space_id: 'space-1',
  parent_task_id: null,
  title: 'Prepare outline',
  description: '',
  status: 'planned',
  priority: 'medium',
  due_date: '2026-08-11',
  tags: ['writing'],
  completed_at: null,
  archived_at: null,
  created_at: '2026-08-10T00:00:00Z',
  updated_at: '2026-08-10T00:00:00Z',
}

describe('useTasks shared invalidation', () => {
  beforeEach(() => {
    for (const mock of Object.values(mocks)) mock.mockReset()
    mocks.listTasks.mockResolvedValue([openTask])
    mocks.listTaskAttention.mockResolvedValue([openTask])
  })

  it('refreshes every mounted Task consumer after completion', async () => {
    const completed = {
      ...openTask,
      status: 'done' as const,
      completed_at: '2026-08-10T01:00:00Z',
    }
    mocks.updateTask.mockImplementation(async () => {
      mocks.listTasks.mockResolvedValue([completed])
      return completed
    })
    const first = renderHook(() => useTasks({ spaceId: 'space-1' }))
    const second = renderHook(() => useTasks({ spaceId: 'space-1' }))

    await waitFor(() => {
      expect(first.result.current.tasks).toEqual([openTask])
      expect(second.result.current.tasks).toEqual([openTask])
    })

    await act(async () => {
      await first.result.current.toggleComplete(openTask)
    })

    await waitFor(() => {
      expect(first.result.current.tasks[0]?.status).toBe('done')
      expect(second.result.current.tasks[0]?.status).toBe('done')
    })
    expect(mocks.updateTask).toHaveBeenCalledWith(
      openTask.id,
      expect.objectContaining({ status: 'done', title: openTask.title }),
    )
  })

  it('refreshes attention consumers after a one-click completion', async () => {
    mocks.updateTask.mockResolvedValue({ ...openTask, status: 'done' })
    mocks.listTaskAttention.mockResolvedValueOnce([openTask]).mockResolvedValueOnce([])
    const attention = renderHook(() => useTaskAttention())

    await waitFor(() => expect(attention.result.current.tasks).toEqual([openTask]))
    await act(async () => {
      await attention.result.current.toggleComplete(openTask)
    })
    await waitFor(() => expect(attention.result.current.tasks).toEqual([]))
  })
})
