import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({
  save: vi.fn(),
  remove: vi.fn(),
  test: vi.fn(),
  useAiSettings: vi.fn(),
}))

vi.mock('@/hooks/useAi', () => ({ useAiSettings: mocks.useAiSettings }))

import { AiSettings } from '@/components/ai/AiSettings'

describe('AI settings', () => {
  beforeEach(() => {
    mocks.save.mockReset().mockResolvedValue(undefined)
    mocks.remove.mockReset().mockResolvedValue(undefined)
    mocks.test.mockReset().mockResolvedValue('Connection successful.')
    mocks.useAiSettings.mockReturnValue({
      status: 'configured',
      statuses: [
        { provider: 'deepseek', configured: true, status: 'configured' },
        { provider: 'openai', configured: false, status: 'missing' },
      ],
      loading: false,
      error: null,
      isTauri: true,
      save: mocks.save,
      remove: mocks.remove,
      test: mocks.test,
    })
  })

  it('replaces, tests, and removes a stored key without exposing it', async () => {
    const user = userEvent.setup()
    render(<AiSettings />)

    const input = screen.getByLabelText('DeepSeek API key')
    expect(input).toHaveAttribute('type', 'password')
    await user.type(input, 'sk-secret')
    await user.click(screen.getByRole('button', { name: 'Replace' }))
    expect(mocks.save).toHaveBeenCalledWith('deepseek', 'sk-secret')
    expect(input).toHaveValue('')

    await user.click(screen.getByRole('button', { name: 'Test DeepSeek' }))
    expect(mocks.test).toHaveBeenCalledWith('deepseek')
    expect(await screen.findByRole('status')).toHaveTextContent('Connection successful.')

    await user.click(screen.getByRole('button', { name: 'Remove DeepSeek key' }))
    await user.click(screen.getAllByRole('button', { name: 'Remove DeepSeek key' })[1])
    expect(mocks.remove).toHaveBeenCalledWith('deepseek')
  })
})
