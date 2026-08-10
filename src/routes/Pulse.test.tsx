import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { describe, expect, it, vi } from 'vitest'
import type { Task } from '@/lib/db/types'

const toggleComplete = vi.fn().mockResolvedValue(undefined)

function localDate(offset: number): string {
  const date = new Date()
  date.setDate(date.getDate() + offset)
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`
}

function makeTask(id: string, title: string, dueDate: string): Task {
  return {
    id,
    space_id: null,
    parent_task_id: null,
    title,
    description: '',
    status: 'planned',
    priority: 'none',
    due_date: dueDate,
    tags: [],
    completed_at: null,
    archived_at: null,
    created_at: '2026-08-10T00:00:00Z',
    updated_at: '2026-08-10T00:00:00Z',
  }
}

const overdue = makeTask('overdue', 'Send invoice', localDate(-1))
const upcoming = makeTask('upcoming', 'Plan tomorrow', localDate(1))

vi.mock('@/hooks/useSpaces', () => ({
  useSpaces: () => ({ spaces: [] }),
}))

vi.mock('@/hooks/useNotes', () => ({
  useGlobalNotes: () => ({ recent: [], pinned: [], loading: false }),
}))

vi.mock('@/hooks/useTasks', () => ({
  useTaskAttention: () => ({
    tasks: [overdue, upcoming],
    loading: false,
    error: null,
    toggleComplete,
  }),
}))

import { Pulse } from '@/routes/Pulse'

describe('Pulse Task attention', () => {
  it('shows global Task attention without requiring a Space', async () => {
    const user = userEvent.setup()
    render(
      <MemoryRouter>
        <Pulse />
      </MemoryRouter>,
    )

    expect(screen.getByText('Overdue')).toBeInTheDocument()
    expect(screen.getByText('Send invoice')).toBeInTheDocument()
    expect(screen.getByText('Coming up')).toBeInTheDocument()
    expect(screen.getByText('Plan tomorrow')).toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: 'Complete Send invoice' }))
    expect(toggleComplete).toHaveBeenCalledWith(overdue)
  })
})
