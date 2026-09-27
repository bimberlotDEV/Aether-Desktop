import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { AiConversation, AiMessage } from '@/lib/db/types'

const hookMocks = vi.hoisted(() => ({
  useAiConversations: vi.fn(),
  useAiSettings: vi.fn(),
  useAiConversation: vi.fn(),
}))

vi.mock('@/hooks/useAi', () => hookMocks)

import { AppErrorBoundary } from '@/components/AppErrorBoundary'
import { AiView } from '@/components/ai/AiView'

const conversation: AiConversation = {
  id: 'conversation-1',
  space_id: null,
  title: 'Render regression',
  provider: 'deepseek',
  model: 'deepseek-v4-flash',
  system_context_version: 1,
  archived_at: null,
  created_at: '2026-08-11T00:00:00Z',
  updated_at: '2026-08-11T00:00:00Z',
  last_opened_at: null,
}

function userMessage(id: string, content: string): AiMessage {
  return {
    id,
    conversation_id: conversation.id,
    role: 'user',
    content,
    status: 'complete',
    provider_message_id: null,
    error_code: null,
    metadata_json: null,
    provider: null,
    model: null,
    routing_mode: null,
    route_reason: null,
    route_policy_mode: null,
    execution_location: null,
    runtime_id: null,
    created_at: '2026-08-11T00:00:00Z',
    updated_at: '2026-08-11T00:00:00Z',
  }
}

function routedAssistant(): AiMessage {
  return {
    ...userMessage('assistant-1', 'Cloud answer'),
    role: 'assistant',
    provider: 'openai',
    model: 'gpt-5-mini',
    routing_mode: 'auto',
    route_reason: 'Automatic used cloud because no eligible local runtime is configured.',
    route_policy_mode: 'automatic',
    execution_location: 'cloud',
    runtime_id: 'openai',
  }
}

describe('AiView message scrolling', () => {
  let messages: AiMessage[]
  let originalScrollIntoView: PropertyDescriptor | undefined

  beforeEach(() => {
    for (const mock of Object.values(hookMocks)) mock.mockReset()
    vi.spyOn(console, 'error').mockImplementation(() => undefined)
    originalScrollIntoView = Object.getOwnPropertyDescriptor(
      Element.prototype,
      'scrollIntoView',
    )
    Object.defineProperty(Element.prototype, 'scrollIntoView', {
      configurable: true,
      writable: true,
      value: vi.fn(() => Promise.resolve()),
    })

    messages = [userMessage('message-1', 'First prompt')]
    hookMocks.useAiConversations.mockReturnValue({
      conversations: [conversation],
      loading: false,
      error: null,
      routingSettings: { mode: 'automatic' },
      create: vi.fn(),
      rename: vi.fn(),
      archive: vi.fn(),
      remove: vi.fn(),
      isTauri: true,
    })
    hookMocks.useAiSettings.mockReturnValue({
      status: 'configured',
      statuses: [
        { provider: 'deepseek', configured: true, status: 'configured' },
        { provider: 'openai', configured: false, status: 'missing' },
      ],
      loading: false,
      error: null,
      save: vi.fn(),
      remove: vi.fn(),
      test: vi.fn(),
      isTauri: true,
    })
    hookMocks.useAiConversation.mockImplementation(() => ({
      messages,
      contextItems: [],
      resolvedContext: [],
      loading: false,
      error: null,
      streaming: false,
      activity: null,
      disclosure: null,
      load: vi.fn(),
      send: vi.fn(),
      cancel: vi.fn(),
      approveDisclosure: vi.fn(),
      attach: vi.fn(),
      detach: vi.fn(),
      isTauri: true,
    }))
  })

  afterEach(() => {
    if (originalScrollIntoView) {
      Object.defineProperty(Element.prototype, 'scrollIntoView', originalScrollIntoView)
    } else {
      delete (Element.prototype as Partial<Element>).scrollIntoView
    }
    vi.restoreAllMocks()
  })

  it('does not return a browser scroll promise as the React effect cleanup', async () => {
    const view = render(
      <AppErrorBoundary>
        <AiView />
      </AppErrorBoundary>,
    )
    await screen.findByText('First prompt')

    messages = [...messages, userMessage('message-2', 'Second prompt')]
    view.rerender(
      <AppErrorBoundary>
        <AiView />
      </AppErrorBoundary>,
    )

    await screen.findByText('Second prompt')
    await waitFor(() => expect(Element.prototype.scrollIntoView).toHaveBeenCalled())
    expect(screen.queryByRole('alert')).not.toBeInTheDocument()
  })

  it('renders locality-aware route provenance without raw policy data', async () => {
    messages = [userMessage('message-1', 'Question'), routedAssistant()]
    render(<AiView />)

    expect(
      await screen.findByText('Automatic → Cloud · OpenAI · GPT-5 mini', {
        exact: false,
      }),
    ).toBeVisible()
    expect(screen.queryByText(/request hash/i)).not.toBeInTheDocument()
  })

  it('reports the no-local-runtime state independently of cloud credentials', async () => {
    hookMocks.useAiSettings.mockReturnValue({
      status: 'configured',
      statuses: [{ provider: 'openai', configured: true, status: 'configured' }],
      routingSettings: { mode: 'local_only' },
      loading: false,
      error: null,
      isTauri: true,
    })
    render(<AiView />)

    expect(await screen.findByText(/no local runtime is configured/i)).toBeVisible()
    expect(screen.getByRole('button', { name: 'Send message' })).toBeDisabled()
  })

  it('shows a minimized cloud tool-result approval instead of raw tool data', async () => {
    const approveDisclosure = vi.fn()
    hookMocks.useAiConversation.mockReturnValue({
      messages,
      contextItems: [],
      resolvedContext: [],
      loading: false,
      error: null,
      streaming: true,
      activity: {
        phase: 'awaiting_disclosure_approval',
        label: 'Waiting for permission to share local context…',
      },
      disclosure: {
        requestId: 'request-1',
        provider: 'openai',
        model: 'gpt-5-mini',
        categories: ['calendar'],
        itemCount: 3,
        reason: 'Use the requested local context to answer this message.',
      },
      load: vi.fn(),
      send: vi.fn(),
      cancel: vi.fn(),
      approveDisclosure,
      attach: vi.fn(),
      detach: vi.fn(),
      isTauri: true,
    })
    const user = userEvent.setup()
    render(<AiView />)

    expect(
      await screen.findByText(/send 3 calendar items to OpenAI/i),
    ).toBeVisible()
    expect(screen.queryByText(/request-1|tool_call|arguments/i)).not.toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: 'Approve once' }))
    expect(approveDisclosure).toHaveBeenCalledOnce()
  })
})
