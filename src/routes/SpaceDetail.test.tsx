import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { Space } from '@/lib/db/types'

const state = vi.hoisted(() => ({
  isDesktop: true,
  reload: vi.fn(),
  data: {
    spaceId: 'space-1',
    spaceName: 'Aether',
    lastWorkedAt: new Date().toISOString(),
    recentNotes: [],
    openTasks: [
      {
        id: 'task-1',
        title: 'Finish Milestone D',
        detail: 'high priority',
        updatedAt: new Date().toISOString(),
        destination: '/spaces/space-1/tasks',
        provenance: 'Task',
      },
    ],
    recentFiles: [],
    latestConversation: {
      id: 'conversation-1',
      title: 'Milestone D review',
      detail: 'Most recently used conversation',
      updatedAt: new Date().toISOString(),
      destination: '/spaces/space-1/ai',
      provenance: 'AI conversation',
    },
    recentActivity: [
      {
        id: 'event-1',
        eventType: 'note_edited',
        title: 'Edited note Continuity plan',
        detail: null,
        spaceId: 'space-1',
        spaceName: 'Aether',
        entityType: 'note',
        entityId: 'note-1',
        destination: '/spaces/space-1/notes',
        createdAt: new Date().toISOString(),
      },
    ],
    suggestedNextStep: {
      title: 'Continue task: Finish Milestone D',
      detail: 'high priority',
      destination: '/spaces/space-1/tasks',
      sourceType: 'task',
      sourceId: 'task-1',
    },
  },
}))

vi.mock('@/hooks/useContinuity', () => ({
  useSpaceContinuity: () => ({
    data: state.isDesktop ? state.data : null,
    loading: false,
    error: null,
    isDesktop: state.isDesktop,
    reload: state.reload,
  }),
}))

import { OverviewTab } from '@/routes/SpaceDetail'

const space: Space = {
  id: 'space-1',
  name: 'Aether',
  description: 'Product workspace',
  icon: null,
  accent: null,
  template_type: null,
  favourite: false,
  archived_at: null,
  sort_order: 0,
  settings_json: null,
  parent_space_id: null,
  last_opened_at: null,
  created_at: '2026-08-26T00:00:00Z',
  updated_at: '2026-08-26T00:00:00Z',
}

describe('Space continuity overview', () => {
  beforeEach(() => {
    state.isDesktop = true
  })

  it('shows the deterministic next step and opens its supported destination', async () => {
    render(
      <MemoryRouter initialEntries={['/spaces/space-1']}>
        <Routes>
          <Route
            path="/spaces/space-1"
            element={
              <OverviewTab space={space} modules={[]}>
                {[]}
              </OverviewTab>
            }
          />
          <Route path="/spaces/space-1/tasks" element={<p>Tasks opened</p>} />
        </Routes>
      </MemoryRouter>,
    )
    expect(screen.getByText('Continue task: Finish Milestone D')).toBeInTheDocument()
    expect(screen.getByText('Milestone D review')).toBeInTheDocument()
    expect(screen.getByText('Edited note Continuity plan')).toBeInTheDocument()
    await userEvent.click(screen.getByRole('button', { name: /Continue task/ }))
    expect(screen.getByText('Tasks opened')).toBeInTheDocument()
  })

  it('shows an honest installed-app disclosure in browser mode', () => {
    state.isDesktop = false
    render(
      <MemoryRouter>
        <OverviewTab space={space} modules={[]}>
          {[]}
        </OverviewTab>
      </MemoryRouter>,
    )
    expect(
      screen.getByText(/Open Aether Desktop to compose this overview/),
    ).toBeInTheDocument()
    expect(
      screen.queryByText('Continue task: Finish Milestone D'),
    ).not.toBeInTheDocument()
  })
})
