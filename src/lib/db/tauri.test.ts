import { beforeEach, describe, expect, it, vi } from 'vitest'

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }))

vi.mock('@tauri-apps/api/core', () => ({ invoke }))

import { createSpaceWithModules, updateNote, updateSpace } from '@/lib/db/tauri'

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
})
