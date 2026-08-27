import { useCallback, useEffect, useRef, useState } from 'react'
import type {
  AiContextItem,
  AiConversation,
  AiMessage,
  AiMode,
  AiResolvedContextItem,
  AiProvider,
  AiProviderStatus,
} from '@/lib/db/types'
import * as db from '@/lib/db/tauri'

const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
const conversationListeners = new Set<() => void>()

function notifyConversations() {
  conversationListeners.forEach((listener) => listener())
}

function requestId() {
  return globalThis.crypto?.randomUUID?.() ?? `request-${Date.now()}`
}

function upsertMessage(messages: AiMessage[], next: AiMessage) {
  const index = messages.findIndex((message) => message.id === next.id)
  if (index === -1) return [...messages, next]
  return messages.map((message, messageIndex) =>
    messageIndex === index ? next : message,
  )
}

export function useAiSettings() {
  const [statuses, setStatuses] = useState<AiProviderStatus[]>([])
  const [loading, setLoading] = useState(isTauri)
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(async () => {
    if (!isTauri) return
    setLoading(true)
    setError(null)
    try {
      setStatuses(await db.listAiProviderStatuses())
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Could not read AI settings.')
    } finally {
      setLoading(false)
    }
  }, [])
  useEffect(() => void load(), [load])

  const save = useCallback(
    async (provider: AiProvider['id'], apiKey: string) => {
      setError(null)
      try {
        await db.setAiProviderApiKey(provider, apiKey)
        await load()
      } catch (cause) {
        setError(cause instanceof Error ? cause.message : 'Could not save the API key.')
        throw cause
      }
    },
    [load],
  )
  const remove = useCallback(
    async (provider: AiProvider['id']) => {
      setError(null)
      await db.removeAiProviderApiKey(provider)
      await load()
    },
    [load],
  )
  const test = useCallback(async (provider: AiProvider['id']) => {
    setError(null)
    try {
      return await db.testAiProviderConnection(
        provider,
        provider === 'openai' ? 'gpt-5-mini' : 'deepseek-v4-flash',
      )
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Connection test failed.')
      throw cause
    }
  }, [])

  const status = statuses.some((item) => item.configured) ? 'configured' : 'missing'
  return { status, statuses, loading, error, save, remove, test, isTauri }
}

export function useAiConversations(spaceId?: string) {
  const [conversations, setConversations] = useState<AiConversation[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      setConversations(isTauri ? await db.listAiConversations(spaceId) : [])
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Could not load conversations.')
    } finally {
      setLoading(false)
    }
  }, [spaceId])
  useEffect(() => void load(), [load])
  useEffect(() => {
    const listener = () => void load()
    conversationListeners.add(listener)
    return () => {
      conversationListeners.delete(listener)
    }
  }, [load])

  const create = useCallback(
    async (model = 'deepseek-v4-flash') => {
      const conversation = await db.createAiConversation({ spaceId, model })
      notifyConversations()
      return conversation
    },
    [spaceId],
  )
  const rename = useCallback(async (id: string, title: string) => {
    await db.updateAiConversation(id, { title })
    notifyConversations()
  }, [])
  const archive = useCallback(async (id: string) => {
    await db.updateAiConversation(id, { archived: true })
    notifyConversations()
  }, [])
  const remove = useCallback(async (id: string) => {
    await db.deleteAiConversation(id)
    notifyConversations()
  }, [])

  return { conversations, loading, error, create, rename, archive, remove, isTauri }
}

export function useAiConversation(conversationId: string | null) {
  const [messages, setMessages] = useState<AiMessage[]>([])
  const [contextItems, setContextItems] = useState<AiContextItem[]>([])
  const [resolvedContext, setResolvedContext] = useState<AiResolvedContextItem[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [activeRequestId, setActiveRequestId] = useState<string | null>(null)
  const activeAssistantId = useRef<string | null>(null)
  const activeRequestIdRef = useRef<string | null>(null)
  const messageRevision = useRef(0)
  const loadSequence = useRef(0)
  const conversationIdRef = useRef(conversationId)
  conversationIdRef.current = conversationId

  const load = useCallback(async () => {
    if (!conversationId || !isTauri) {
      loadSequence.current += 1
      setMessages([])
      setContextItems([])
      setResolvedContext([])
      setLoading(false)
      return
    }
    const sequence = ++loadSequence.current
    setLoading(true)
    setError(null)
    const revisionAtStart = messageRevision.current
    try {
      const [nextMessages, nextContext, nextResolved] = await Promise.all([
        db.listAiMessages(conversationId, 200),
        db.listAiContext(conversationId),
        db.resolveAiContext(conversationId),
      ])
      if (
        conversationIdRef.current !== conversationId ||
        loadSequence.current !== sequence
      )
        return
      if (
        activeRequestIdRef.current === null &&
        messageRevision.current === revisionAtStart
      ) {
        setMessages(nextMessages)
      }
      setContextItems(nextContext)
      setResolvedContext(nextResolved)
    } catch (cause) {
      if (
        conversationIdRef.current === conversationId &&
        loadSequence.current === sequence
      ) {
        setError(
          cause instanceof Error ? cause.message : 'Could not load this conversation.',
        )
      }
    } finally {
      if (
        conversationIdRef.current === conversationId &&
        loadSequence.current === sequence
      )
        setLoading(false)
    }
  }, [conversationId])
  useEffect(() => void load(), [load])

  const send = useCallback(
    async (content: string, retryUserMessageId?: string, mode: AiMode = 'ask') => {
      if (!conversationId) return
      const id = requestId()
      const optimisticUserMessageId = retryUserMessageId ? null : `optimistic-user-${id}`
      activeRequestIdRef.current = id
      messageRevision.current += 1
      setActiveRequestId(id)
      setError(null)
      if (optimisticUserMessageId) {
        const now = new Date().toISOString()
        setMessages((current) => [
          ...current,
          {
            id: optimisticUserMessageId,
            conversation_id: conversationId,
            role: 'user',
            content,
            status: 'pending',
            provider_message_id: null,
            error_code: null,
            metadata_json: JSON.stringify({ mode }),
            provider: null,
            model: null,
            routing_mode: null,
            route_reason: null,
            created_at: now,
            updated_at: now,
          },
        ])
      }
      try {
        await db.streamAiMessage(
          id,
          conversationId,
          content,
          (event) => {
            messageRevision.current += 1
            if (event.event === 'started') {
              activeAssistantId.current = event.data.assistantMessage.id
              setMessages((current) => {
                const withoutOptimistic = optimisticUserMessageId
                  ? current.filter((message) => message.id !== optimisticUserMessageId)
                  : current
                return upsertMessage(
                  upsertMessage(withoutOptimistic, event.data.userMessage),
                  event.data.assistantMessage,
                )
              })
            } else if (event.event === 'delta') {
              const assistantId = activeAssistantId.current
              setMessages((current) =>
                current.map((message) =>
                  message.id === assistantId
                    ? { ...message, content: message.content + event.data.content }
                    : message,
                ),
              )
            } else {
              const terminal =
                event.event === 'failed'
                  ? event.data.assistantMessage
                  : event.data.message
              setMessages((current) => upsertMessage(current, terminal))
              activeAssistantId.current = null
              if (event.event === 'failed') setError(event.data.message)
            }
          },
          retryUserMessageId,
          mode,
        )
        // Channel delivery can be interrupted by WebView lifecycle or timing. SQLite is
        // authoritative, so reconcile after every completed command instead of requiring
        // the user to reload the entire page to see the persisted response.
        const persistedMessages = await db.listAiMessages(conversationId, 200)
        if (conversationIdRef.current === conversationId) {
          messageRevision.current += 1
          setMessages(persistedMessages)
        }
        notifyConversations()
      } catch (cause) {
        setError(cause instanceof Error ? cause.message : 'Could not send the message.')
        activeRequestIdRef.current = null
        messageRevision.current += 1
        await load()
      } finally {
        activeRequestIdRef.current = null
        setActiveRequestId(null)
      }
    },
    [conversationId, load],
  )

  const cancel = useCallback(async () => {
    if (activeRequestId) await db.cancelAiRequest(activeRequestId)
  }, [activeRequestId])

  const attach = useCallback(
    async (entityType: 'note' | 'task' | 'vault' | 'memory', entityId: string) => {
      if (!conversationId) return
      await db.addAiContext(conversationId, entityType, entityId)
      await load()
    },
    [conversationId, load],
  )
  const detach = useCallback(
    async (attachmentId: string) => {
      await db.removeAiContext(attachmentId)
      await load()
    },
    [load],
  )

  return {
    messages,
    contextItems,
    resolvedContext,
    loading,
    error,
    streaming: activeRequestId !== null,
    load,
    send,
    cancel,
    attach,
    detach,
    isTauri,
  }
}
