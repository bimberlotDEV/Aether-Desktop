import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { describe, expect, it, vi } from 'vitest'
import type { NoteListItem } from '@/lib/db/types'

const note: NoteListItem = {
  id: 'note-1',
  space_id: 'space-1',
  title: 'Keyboard note',
  excerpt: 'A regression fixture',
  content_format: 'markdown',
  pinned: false,
  revision: 1,
  archived_at: null,
  created_at: '2026-08-11T00:00:00Z',
  updated_at: '2026-08-11T00:00:00Z',
  last_opened_at: null,
}

vi.mock('@/hooks/useNotes', () => ({
  useNotes: () => ({
    notes: [note],
    archivedNotes: [],
    loading: false,
    create: vi.fn(),
    remove: vi.fn(),
    pin: vi.fn(),
    archive: vi.fn(),
    restore: vi.fn(),
    duplicate: vi.fn(),
  }),
  useNoteSearch: () => ({ results: [], searching: false, search: vi.fn() }),
  useNote: vi.fn(),
}))

import { NotesView } from '@/routes/Notes'

describe('Notes keyboard access', () => {
  it('exposes note selection and a keyboard-dismissible action menu', async () => {
    render(
      <MemoryRouter initialEntries={['/spaces/space-1/notes']}>
        <Routes>
          <Route path="/spaces/:spaceId/notes" element={<NotesView />} />
        </Routes>
      </MemoryRouter>,
    )

    expect(screen.getByRole('button', { name: 'Open Keyboard note' })).toBeInTheDocument()
    const actions = screen.getByRole('button', { name: 'Actions for Keyboard note' })
    expect(actions).toHaveAttribute('aria-expanded', 'false')
    await userEvent.click(actions)
    expect(
      screen.getByRole('menu', { name: 'Actions for Keyboard note' }),
    ).toBeInTheDocument()
    expect(screen.getByRole('menuitem', { name: 'Archive' })).toBeInTheDocument()
    await userEvent.keyboard('{Escape}')
    expect(screen.queryByRole('menu')).not.toBeInTheDocument()
  })
})
