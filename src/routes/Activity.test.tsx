import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({ listActivity: vi.fn() }))
vi.mock('@/lib/db/tauri', () => mocks)

import { Activity } from '@/routes/Activity'

describe('Activity route', () => {
  beforeEach(() => {
    Object.defineProperty(window, '__TAURI_INTERNALS__', {
      value: {},
      configurable: true,
    })
    mocks.listActivity.mockReset()
    mocks.listActivity.mockResolvedValue([
      {
        id: 'event-1',
        eventType: 'task_completed',
        title: 'Completed task Ship continuity',
        detail: null,
        spaceId: 'space-1',
        spaceName: 'Aether',
        entityType: 'task',
        entityId: 'task-1',
        destination: '/spaces/space-1/tasks',
        createdAt: new Date().toISOString(),
      },
    ])
  })

  it('renders real curated events and navigates to their safe destination', async () => {
    render(
      <MemoryRouter initialEntries={['/activity']}>
        <Routes>
          <Route path="/activity" element={<Activity />} />
          <Route path="/spaces/:spaceId/tasks" element={<p>Space Tasks opened</p>} />
        </Routes>
      </MemoryRouter>,
    )

    expect(await screen.findByText('Completed task Ship continuity')).toBeInTheDocument()
    expect(screen.getByText('Aether')).toBeInTheDocument()
    await userEvent.click(screen.getByRole('button', { name: /Completed task/ }))
    expect(screen.getByText('Space Tasks opened')).toBeInTheDocument()
  })

  it('does not fabricate persisted events in browser mode', () => {
    Reflect.deleteProperty(window, '__TAURI_INTERNALS__')
    render(
      <MemoryRouter>
        <Activity />
      </MemoryRouter>,
    )
    expect(
      screen.getByText('Your local timeline stays on this device'),
    ).toBeInTheDocument()
    expect(mocks.listActivity).not.toHaveBeenCalled()
  })
})
