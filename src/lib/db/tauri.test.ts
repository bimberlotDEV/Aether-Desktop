import { beforeEach, describe, expect, it, vi } from 'vitest'

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }))

vi.mock('@tauri-apps/api/core', () => ({ invoke }))

import {
  createSpaceWithModules,
  createTask,
  listTaskAttention,
  listTasks,
  updateNote,
  updateSpace,
  updateTask,
} from '@/lib/db/tauri'

describe('Tauri database boundary', () => {
  beforeEach(() => {
    invoke.mockReset()
    invoke.mockResolvedValue(null)
  })

  it('maps Space form values to the command argument contract', async () => {
    await createSpaceWithModules({
      name: 'Research',
      moduleTypes: ['notes', 'tasks'],
    })

    expect(invoke).toHaveBeenCalledWith('create_space_with_modules', {
      name: 'Research',
      description: null,
      icon: null,
      accent: null,
      templateType: null,
      parentSpaceId: null,
      moduleTypes: ['notes', 'tasks'],
    })
  })

  it('maps cleared optional Space fields to empty persisted values', async () => {
    await updateSpace('space-1', {
      name: 'Renamed',
      description: null,
      icon: null,
      accent: '#6366f1',
    })

    expect(invoke).toHaveBeenCalledWith('update_space', {
      id: 'space-1',
      name: 'Renamed',
      description: '',
      icon: '',
      accent: '#6366f1',
      settingsJson: null,
      parentSpaceId: null,
    })
  })

  it('passes the optimistic revision used to protect Note autosave', async () => {
    await updateNote('note-1', {
      title: 'Title',
      content: 'Body',
      excerpt: 'Body',
      expectedRevision: 7,
    })

    expect(invoke).toHaveBeenCalledWith('update_note', {
      id: 'note-1',
      title: 'Title',
      content: 'Body',
      excerpt: 'Body',
      expectedRevision: 7,
    })
  })

  it('passes full-state Task inputs through the command boundary', async () => {
    const input = {
      spaceId: 'space-1',
      parentTaskId: null,
      title: 'Ship Phase 5',
      description: 'Complete the persistence slice',
      status: 'in_progress' as const,
      priority: 'high' as const,
      dueDate: '2026-08-12',
      tags: ['release'],
    }

    await createTask(input)
    await updateTask('task-1', input)

    expect(invoke).toHaveBeenNthCalledWith(1, 'create_task', { input })
    expect(invoke).toHaveBeenNthCalledWith(2, 'update_task', { id: 'task-1', input })
  })

  it('maps Task filters and attention dates without changing their semantics', async () => {
    await listTasks({
      spaceId: 'space-1',
      status: 'planned',
      priority: 'medium',
      search: 'release',
      includeArchived: false,
      limit: 50,
    })
    await listTaskAttention('2026-08-10', '2026-08-17', 12)

    expect(invoke).toHaveBeenNthCalledWith(1, 'list_tasks', {
      filter: {
        spaceId: 'space-1',
        status: 'planned',
        priority: 'medium',
        search: 'release',
        includeArchived: false,
        limit: 50,
      },
    })
    expect(invoke).toHaveBeenNthCalledWith(2, 'list_task_attention', {
      today: '2026-08-10',
      horizon: '2026-08-17',
      limit: 12,
    })
  })
})
