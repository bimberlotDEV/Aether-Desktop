import { act, renderHook, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { Space } from '@/lib/db/types'

const mocks = vi.hoisted(() => {
  Object.defineProperty(window, '__TAURI_INTERNALS__', {
    configurable: true,
    value: {},
  })
  return {
    listTopLevelSpaces: vi.fn(),
    archiveSpace: vi.fn(),
  }
})

vi.mock('@/lib/db/tauri', () => mocks)

import { useSpaces } from '@/hooks/useSpaces'

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

describe('useSpaces shared invalidation', () => {
  beforeEach(() => {
    mocks.listTopLevelSpaces.mockReset()
    mocks.archiveSpace.mockReset()
    mocks.listTopLevelSpaces.mockResolvedValue([space])
    mocks.archiveSpace.mockImplementation(async () => {
      mocks.listTopLevelSpaces.mockResolvedValue([])
      return true
    })
  })

  it('refreshes every mounted Space consumer after a mutation', async () => {
    const first = renderHook(() => useSpaces())
    const second = renderHook(() => useSpaces())

    await waitFor(() => {
      expect(first.result.current.spaces).toEqual([space])
      expect(second.result.current.spaces).toEqual([space])
    })

    await act(async () => {
      await first.result.current.archive(space.id)
    })

    await waitFor(() => {
      expect(first.result.current.spaces).toEqual([])
      expect(second.result.current.spaces).toEqual([])
    })
    expect(mocks.archiveSpace).toHaveBeenCalledWith(space.id)
  })
})
