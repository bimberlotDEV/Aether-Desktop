import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { ActionPreview, ActionResult, Source, Space } from '@/lib/db/types'

const propose = vi.fn().mockResolvedValue(undefined)
const approve = vi.fn().mockResolvedValue(undefined)
const cancel = vi.fn().mockResolvedValue(undefined)
const reset = vi.fn()

const space = {
  id: 'space-1',
  name: 'Studio',
  description: null,
  icon: null,
  accent: null,
  template_type: null,
  favourite: false,
  archived_at: null,
  sort_order: 0,
  settings_json: null,
  parent_space_id: null,
  last_opened_at: null,
  created_at: '2026-08-27',
  updated_at: '2026-08-27',
} satisfies Space
const source = {
  id: 'source-1',
  rootPath: 'C:\\Studio',
  displayName: 'Studio files',
  spaceId: 'space-1',
  scanStatus: 'complete',
  lastScanAt: '2026-08-27',
  lastError: null,
  createdAt: '2026-08-27',
  updatedAt: '2026-08-27',
} satisfies Source

let actionState: {
  isDesktop: boolean
  spaces: Space[]
  sources: Source[]
  preview: ActionPreview | null
  result: ActionResult | null
  loading: boolean
  busy: boolean
  error: string | null
  propose: typeof propose
  approve: typeof approve
  cancel: typeof cancel
  reset: typeof reset
}

vi.mock('@/hooks/useActions', () => ({ useActions: () => actionState }))
import { Actions } from '@/routes/Actions'

function renderActions() {
  return render(
    <MemoryRouter>
      <Actions />
    </MemoryRouter>,
  )
}

describe('Safe Actions', () => {
  beforeEach(() => {
    propose.mockClear()
    approve.mockClear()
    cancel.mockClear()
    reset.mockClear()
    actionState = {
      isDesktop: true,
      spaces: [space],
      sources: [source],
      preview: null,
      result: null,
      loading: false,
      busy: false,
      error: null,
      propose,
      approve,
      cancel,
      reset,
    }
  })

  it('is honest in browser mode and exposes no action form', () => {
    actionState.isDesktop = false
    renderActions()
    expect(
      screen.getByText(/available in the installed desktop app/i),
    ).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /Preview/ })).not.toBeInTheDocument()
  })

  it('builds an exact bounded file proposal without executing it', async () => {
    const user = userEvent.setup()
    renderActions()
    await user.selectOptions(screen.getByLabelText('Action'), 'copyFile')
    await user.selectOptions(screen.getByLabelText('Authorized Source'), 'source-1')
    await user.type(screen.getByLabelText('Indexed source file'), 'in/report.pdf')
    await user.type(screen.getByLabelText('New relative destination'), 'out/report.pdf')
    await user.click(screen.getByRole('button', { name: 'Preview Copy file' }))
    expect(propose).toHaveBeenCalledWith({
      type: 'copyFile',
      sourceId: 'source-1',
      fromRelativePath: 'in/report.pdf',
      toRelativePath: 'out/report.pdf',
    })
    expect(approve).not.toHaveBeenCalled()
  })

  it('requires an explicit approval or cancellation of the one-time preview', async () => {
    const user = userEvent.setup()
    actionState.preview = {
      token: 'opaque',
      actionType: 'copyFile',
      title: 'Copy report.pdf',
      summary: 'Copy one indexed file.',
      consequence: 'Creates out/report.pdf.',
      expiresAt: '2026-08-27T18:00:00Z',
    }
    const view = renderActions()
    expect(approve).not.toHaveBeenCalled()
    await user.click(screen.getByRole('button', { name: 'Approve and execute' }))
    expect(approve).toHaveBeenCalledTimes(1)
    view.unmount()
    renderActions()
    await user.click(screen.getByRole('button', { name: 'Cancel' }))
    expect(cancel).toHaveBeenCalledTimes(1)
  })

  it('shows the recorded result and provides a route to it', () => {
    actionState.result = {
      actionType: 'createTask',
      title: 'Task created',
      detail: 'Added Review brief.',
      destination: '/tasks',
      executedAt: '2026-08-27T17:00:00Z',
    }
    renderActions()
    expect(screen.getByText('Task created')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Open result' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'New Action' })).toBeInTheDocument()
  })
})
