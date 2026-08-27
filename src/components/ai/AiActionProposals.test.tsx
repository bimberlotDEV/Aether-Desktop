import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({
  previewAiActionProposal: vi.fn(),
  executeAction: vi.fn(),
  cancelAction: vi.fn(),
}))
vi.mock('@/lib/db/tauri', () => mocks)

import { AiActionProposals } from '@/components/ai/AiActionProposals'

describe('AI Action proposal review', () => {
  beforeEach(() => {
    mocks.previewAiActionProposal.mockReset().mockResolvedValue({
      token: 'opaque-token',
      actionType: 'createTask',
      title: 'Create Task',
      summary: 'Create Task “Ship safely”',
      consequence: 'Adds one open Task',
      expiresAt: '2026-08-27T12:00:00Z',
    })
    mocks.executeAction.mockReset().mockResolvedValue({})
    mocks.cancelAction.mockReset().mockResolvedValue(true)
  })

  it('requires backend preview and explicit approval before execution', async () => {
    const user = userEvent.setup()
    render(
      <AiActionProposals
        actions={[
          {
            index: 0,
            actionType: 'createTask',
            title: 'Ship safely',
            detail: '',
          },
        ]}
        conversationId="conversation-1"
        messageId="message-1"
        onClose={vi.fn()}
        onExecuted={vi.fn()}
      />,
    )

    expect(mocks.executeAction).not.toHaveBeenCalled()
    await user.click(screen.getByRole('button', { name: 'Preview Action' }))
    expect(mocks.previewAiActionProposal).toHaveBeenCalledWith(
      'conversation-1',
      'message-1',
      0,
    )
    expect(mocks.executeAction).not.toHaveBeenCalled()
    await user.click(screen.getByRole('button', { name: 'Approve and create' }))
    expect(mocks.executeAction).toHaveBeenCalledWith('opaque-token')
    expect(await screen.findByText('Review complete')).toBeInTheDocument()
  })
})
