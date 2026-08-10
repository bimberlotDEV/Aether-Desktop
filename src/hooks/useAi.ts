import { useCallback, useEffect, useRef, useState } from 'react'
import type {
  AiContextItem,
  AiConversation,
  AiMessage,
  AiMode,
  AiResolvedContextItem,
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

export function useAiSettings() {
  const [status, setStatus] = useState<'configured' | 'missing' | 'unavailable'>(
    isTauri ? 'unavailable' : 'missing',
  )
  const [loading, setLoading] = useState(isTauri)
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(async () => {
    if (!isTauri) return
    setLoading(true)
    try {
      setStatus((await db.getAiKeyStatus()).status)
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Could not read AI settings.')
      setStatus('unavailable')
    } finally {
      setLoading(false)
    }
  }, [])
  useEffect(() => void load(), [load])

  const save = useCallback(async (apiKey: string) => {
    setError(null)
    try {
      await db.setAiApiKey(apiKey)
      setStatus('configured')
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Could not save the API key.')
      throw cause
    }
  }, [])
  const remove = useCallback(async () => {
    setError(null)
    await db.removeAiApiKey()
    setStatus('missing')
  }, [])
  const test = useCallback(async () => {
    setError(null)
    try {
      return await db.testAiConnection()
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Connection test failed.')
      throw cause
    }
  }, [])

  return { status, loading, error, save, remove, test, isTauri }
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

  const load = useCallback(async () => {
    if (!conversationId || !isTauri) {
      setMessages([])
      setContextItems([])
      setResolvedContext([])
      return
    }
    setLoading(true)
    setError(null)
    try {
      const [nextMessages, nextContext, nextResolved] = await Promise.all([
        db.listAiMessages(conversationId, 200),
        db.listAiContext(conversationId),
        db.resolveAiContext(conversationId),
      ])
      setMessages(nextMessages)
      setContextItems(nextContext)
      setResolvedContext(nextResolved)
    } catch (cause) {
      setError(
        cause instanceof Error ? cause.message : 'Could not load this conversation.',
      )
    } finally {
      setLoading(false)
    }
  }, [conversationId])
  useEffect(() => void load(), [load])

  const send = useCallback(
    async (content: string, retryUserMessageId?: string, mode: AiMode = 'ask') => {
      if (!conversationId) return
      const id = requestId()
      setActiveRequestId(id)
      setError(null)
      try {
        await db.streamAiMessage(
          id,
          conversationId,
          content,
          (event) => {
            if (event.event === 'started') {
              activeAssistantId.current = event.data.assistantMessage.id
              setMessages((current) => {
                const userExists = current.some(
                  (message) => message.id === event.data.userMessage.id,
                )
                return [
                  ...current,
                  ...(userExists ? [] : [event.data.userMessage]),
                  event.data.assistantMessage,
                ]
              })
            } else if (event.event === 'delta') {
              setMessages((current) =>
                current.map((message) =>
                  message.id === activeAssistantId.current
                    ? { ...message, content: message.content + event.data.content }
                    : message,
                ),
              )
            } else {
              const terminal =
                event.event === 'failed'
                  ? event.data.assistantMessage
                  : event.data.message
              setMessages((current) =>
                current.map((message) =>
                  message.id === terminal.id ? terminal : message,
                ),
              )
              activeAssistantId.current = null
              if (event.event === 'failed') setError(event.data.message)
            }
          },
          retryUserMessageId,
          mode,
        )
        notifyConversations()
      } catch (cause) {
        setError(cause instanceof Error ? cause.message : 'Could not send the message.')
        await load()
      } finally {
        setActiveRequestId(null)
      }
    },
    [conversationId, load],
  )

  const cancel = useCallback(async () => {
    if (activeRequestId) await db.cancelAiRequest(activeRequestId)
  }, [activeRequestId])

  const attach = useCallback(
    async (entityType: 'note' | 'task' | 'vault', entityId: string) => {
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
