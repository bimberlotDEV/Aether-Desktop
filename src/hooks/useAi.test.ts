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

  it('does not let an older in-flight load erase a newly started message stream', async () => {
    let resolveInitialMessages: ((messages: AiMessage[]) => void) | undefined
    mocks.listAiMessages.mockImplementationOnce(
      () =>
        new Promise<AiMessage[]>((resolve) => {
          resolveInitialMessages = resolve
        }),
    )

    const userMessage = message('user-race', 'user', 'Keep this visible', 'complete')
    const assistantMessage = message('assistant-race', 'assistant', '', 'streaming')
    mocks.streamAiMessage.mockImplementation(
      async (
        requestId: string,
        _conversationId: string,
        _content: string,
        callback: (event: AiStreamEvent) => void,
      ) => {
        callback({
          event: 'started',
          data: { requestId, userMessage, assistantMessage },
        })
        await new Promise<void>(() => undefined)
      },
    )

    const hook = renderHook(() => useAiConversation('conversation-1'))
    await waitFor(() => expect(mocks.listAiMessages).toHaveBeenCalledOnce())

    act(() => {
      void hook.result.current.send('Keep this visible')
    })
    await waitFor(() =>
      expect(hook.result.current.messages).toEqual([userMessage, assistantMessage]),
    )

    await act(async () => {
      resolveInitialMessages?.([])
      await Promise.resolve()
    })

    expect(hook.result.current.messages).toEqual([userMessage, assistantMessage])
  })

  it('reconciles persisted messages when the WebView misses stream events', async () => {
    const userMessage = message('user-persisted', 'user', 'Persist this', 'complete')
    const assistantMessage = message(
      'assistant-persisted',
      'assistant',
      'Visible without reload',
      'complete',
    )
    mocks.streamAiMessage.mockResolvedValue(undefined)
    mocks.listAiMessages
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce([userMessage, assistantMessage])

    const hook = renderHook(() => useAiConversation('conversation-1'))
    await waitFor(() => expect(hook.result.current.loading).toBe(false))

    await act(async () => {
      await hook.result.current.send('Persist this')
    })

    expect(hook.result.current.messages).toEqual([userMessage, assistantMessage])
    expect(mocks.listAiMessages).toHaveBeenCalledTimes(2)
  })

  it('upserts a terminal answer even if its started event was missed', async () => {
    const assistantMessage = message(
      'assistant-terminal',
      'assistant',
      'Terminal answer',
      'complete',
    )
    mocks.streamAiMessage.mockImplementation(
      async (
        _requestId: string,
        _conversationId: string,
        _content: string,
        callback: (event: AiStreamEvent) => void,
      ) => {
        callback({ event: 'complete', data: { message: assistantMessage } })
      },
    )
    mocks.listAiMessages
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce([assistantMessage])

    const hook = renderHook(() => useAiConversation('conversation-1'))
    await waitFor(() => expect(hook.result.current.loading).toBe(false))

    await act(async () => {
      await hook.result.current.send('Answer this')
    })

    expect(hook.result.current.messages).toEqual([assistantMessage])
  })
})
