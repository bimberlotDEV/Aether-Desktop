import { act, renderHook, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { AiMessage, AiStreamEvent } from '@/lib/db/types'

const mocks = vi.hoisted(() => {
  Object.defineProperty(window, '__TAURI_INTERNALS__', {
    configurable: true,
    value: {},
  })
  return {
    listAiMessages: vi.fn(),
    listAiContext: vi.fn(),
    resolveAiContext: vi.fn(),
    streamAiMessage: vi.fn(),
  }
})

vi.mock('@/lib/db/tauri', () => mocks)

import { useAiConversation } from '@/hooks/useAi'

function message(
  id: string,
  role: 'user' | 'assistant',
  content: string,
  status: AiMessage['status'],
): AiMessage {
  return {
    id,
    conversation_id: 'conversation-1',
    role,
    content,
    status,
    provider_message_id: null,
    error_code: null,
    metadata_json: null,
    created_at: '2026-08-11T00:00:00Z',
    updated_at: '2026-08-11T00:00:00Z',
  }
}

describe('useAiConversation sending', () => {
  let onStreamEvent: ((event: AiStreamEvent) => void) | undefined

  beforeEach(() => {
    for (const mock of Object.values(mocks)) mock.mockReset()
    mocks.listAiMessages.mockResolvedValue([])
    mocks.listAiContext.mockResolvedValue([])
    mocks.resolveAiContext.mockResolvedValue([])
    mocks.streamAiMessage.mockImplementation(
      async (
        _requestId: string,
        _conversationId: string,
        _content: string,
        callback: (event: AiStreamEvent) => void,
      ) => {
        onStreamEvent = callback
        await new Promise<void>(() => undefined)
      },
    )
  })

  it('shows a submitted prompt immediately and replaces it with the persisted message', async () => {
    const hook = renderHook(() => useAiConversation('conversation-1'))
    await waitFor(() => expect(hook.result.current.loading).toBe(false))

    act(() => {
      void hook.result.current.send('Show this immediately')
    })

    expect(hook.result.current.messages).toEqual([
      expect.objectContaining({
        role: 'user',
        content: 'Show this immediately',
        status: 'pending',
      }),
    ])

    const userMessage = message('user-1', 'user', 'Show this immediately', 'complete')
    const assistantMessage = message('assistant-1', 'assistant', '', 'streaming')
    act(() => {
      onStreamEvent?.({
        event: 'started',
        data: {
          requestId: 'request-1',
          userMessage,
          assistantMessage,
        },
      })
    })

    expect(hook.result.current.messages).toEqual([userMessage, assistantMessage])
  })
})
