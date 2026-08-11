import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { describe, expect, it, vi } from 'vitest'
import type { Space } from '@/lib/db/types'

const space: Space = {
  id: 'space-1',
  name: 'Wiskunde',
  description: 'Math',
  icon: 'Calculator',
  accent: '#4f46e5',
  template_type: 'school',
  favourite: false,
  archived_at: null,
  sort_order: 0,
  settings_json: null,
  parent_space_id: null,
  last_opened_at: null,
  created_at: '2026-08-11T00:00:00Z',
  updated_at: '2026-08-11T00:00:00Z',
}

vi.mock('@/hooks/useSpaces', () => ({
  useSpaces: () => ({
    spaces: [space],
    loading: false,
    toggleFavourite: vi.fn(),
    archive: vi.fn(),
    restore: vi.fn(),
    duplicate: vi.fn(),
    remove: vi.fn(),
    reorder: vi.fn(),
  }),
  useArchivedSpaces: () => ({ spaces: [], loading: false }),
}))

import { Spaces } from '@/routes/Spaces'

describe('Spaces accessibility and icon rendering', () => {
  it('renders stored icon names as icons and exposes keyboard-operable row actions', async () => {
    render(
      <MemoryRouter initialEntries={['/spaces']}>
        <Routes>
          <Route path="/spaces" element={<Spaces />} />
          <Route path="/spaces/:spaceId" element={<p>Space detail opened</p>} />
        </Routes>
      </MemoryRouter>,
    )

    expect(screen.queryByText('Calculator')).not.toBeInTheDocument()
    expect(screen.getByText('🔢')).toBeInTheDocument()
    const actions = screen.getByRole('button', { name: 'Actions for Wiskunde' })
    expect(actions).toHaveAttribute('aria-expanded', 'false')
    await userEvent.click(actions)
    expect(screen.getByRole('menu', { name: 'Actions for Wiskunde' })).toBeInTheDocument()
    expect(screen.getByRole('menuitem', { name: 'Archive' })).toBeInTheDocument()
    await userEvent.keyboard('{Escape}')
    expect(screen.queryByRole('menu')).not.toBeInTheDocument()

    await userEvent.click(screen.getByRole('button', { name: 'Open Wiskunde' }))
    expect(screen.getByText('Space detail opened')).toBeInTheDocument()
  })
})
