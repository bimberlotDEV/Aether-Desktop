import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { describe, expect, it, vi } from 'vitest'

vi.mock('@/hooks/useSpaces', () => ({
  useArchivedSpaces: () => ({ spaces: [], loading: false }),
  useSpaces: () => ({ restore: vi.fn(), remove: vi.fn() }),
}))

import { ArchivedSpaces } from '@/routes/ArchivedSpaces'

describe('ArchivedSpaces navigation', () => {
  it('names the icon-only back button and navigates back to Spaces', async () => {
    render(
      <MemoryRouter initialEntries={['/spaces/archived']}>
        <Routes>
          <Route path="/spaces/archived" element={<ArchivedSpaces />} />
          <Route path="/spaces" element={<p>Active Spaces</p>} />
        </Routes>
      </MemoryRouter>,
    )

    await userEvent.click(screen.getByRole('button', { name: 'Back to Spaces' }))
    expect(screen.getByText('Active Spaces')).toBeInTheDocument()
  })
})
