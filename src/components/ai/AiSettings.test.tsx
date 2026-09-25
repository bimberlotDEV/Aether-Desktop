import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({
  save: vi.fn(),
  remove: vi.fn(),
  test: vi.fn(),
  updateRouting: vi.fn(),
  useAiSettings: vi.fn(),
}))

vi.mock('@/hooks/useAi', () => ({ useAiSettings: mocks.useAiSettings }))

import { AiSettings } from '@/components/ai/AiSettings'

describe('AI settings', () => {
  beforeEach(() => {
    mocks.save.mockReset().mockResolvedValue(undefined)
    mocks.remove.mockReset().mockResolvedValue(undefined)
    mocks.test.mockReset().mockResolvedValue('Connection successful.')
    mocks.updateRouting.mockReset().mockResolvedValue(undefined)
    mocks.useAiSettings.mockReturnValue({
      status: 'configured',
      statuses: [
        { provider: 'deepseek', configured: true, status: 'configured' },
        { provider: 'openai', configured: false, status: 'missing' },
      ],
      loading: false,
      error: null,
      routingSettings: {
        mode: 'automatic',
        preferredLocalRuntime: null,
        preferredLocalModel: null,
        preferredCloudProvider: null,
        preferredCloudModel: null,
        automaticCloudFallback: 'ask_when_needed',
        cloudDisclosurePolicy: 'ask_for_aether_data',
      },
      isTauri: true,
      save: mocks.save,
      remove: mocks.remove,
      test: mocks.test,
      updateRouting: mocks.updateRouting,
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

  it('shows truthful routing, local-runtime, and privacy controls', async () => {
    const user = userEvent.setup()
    render(<AiSettings />)

    expect(screen.getByText(/no local runtime is configured or available/i)).toBeVisible()
    expect(screen.getByText(/eligible local model first/i)).toBeVisible()
    await user.selectOptions(screen.getByLabelText('AI routing mode'), 'local_only')
    expect(mocks.updateRouting).toHaveBeenCalledWith(
      expect.objectContaining({ mode: 'local_only' }),
    )
    await user.selectOptions(
      screen.getByLabelText('Cloud disclosure policy'),
      'allow_explicit_attachments',
    )
    expect(mocks.updateRouting).toHaveBeenCalledWith(
      expect.objectContaining({ cloudDisclosurePolicy: 'allow_explicit_attachments' }),
    )
  })
})
