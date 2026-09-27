import { render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { Sidebar } from '@/components/Sidebar'

describe('Sidebar visual navigation', () => {
  it('keeps every collapsed navigation target named and marks the active route', () => {
    render(
      <MemoryRouter initialEntries={['/tasks']}>
        <Sidebar />
      </MemoryRouter>,
    )

    expect(screen.getByRole('link', { name: 'Tasks' })).toHaveClass(
      'aether-nav-item--active',
    )
    expect(screen.getByRole('link', { name: 'Pulse' })).not.toHaveClass(
      'aether-nav-item--active',
    )
    expect(screen.getByRole('link', { name: 'Settings' })).toBeVisible()
    expect(screen.getByRole('button', { name: 'Search Aether' })).toBeVisible()
  })
})
