import { useEffect, useRef, useState } from 'react'
import {
  Archive,
  Bot,
  Brain,
  CheckSquare,
  Paperclip,
  Pencil,
  Plus,
  RefreshCw,
  Send,
  Square,
  Trash2,
  X,
} from 'lucide-react'
import type { AiConversation, AiMessage, AiMode } from '@/lib/db/types'
import { useAiConversation, useAiConversations, useAiSettings } from '@/hooks/useAi'
import { AiContextPicker } from '@/components/ai/AiContextPicker'
import { AiTaskProposal } from '@/components/ai/AiTaskProposal'
import { parseTaskProposal, type TaskProposal } from '@/lib/aiProposal'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { cn } from '@/lib/utils'

const MODES: { id: AiMode; label: string }[] = [
  { id: 'ask', label: 'Ask' },
  { id: 'summarize', label: 'Summarize' },
  { id: 'explain', label: 'Explain' },
  { id: 'plan', label: 'Plan' },
  { id: 'rewrite', label: 'Rewrite' },
  { id: 'create_tasks', label: 'Propose Tasks' },
]

function modeFromMessage(message: AiMessage): AiMode {
  try {
    const mode = JSON.parse(message.metadata_json ?? '{}').mode
    return MODES.some((candidate) => candidate.id === mode) ? mode : 'ask'
  } catch {
    return 'ask'
  }
}

function precedingUser(messages: AiMessage[], index: number) {
  for (let cursor = index - 1; cursor >= 0; cursor -= 1) {
    if (messages[cursor].role === 'user') return messages[cursor]
  }
  return undefined
}

function ConversationRow({
  conversation,
  active,
  onSelect,
  onArchive,
  onDelete,
}: {
  conversation: AiConversation
  active: boolean
  onSelect: () => void
  onArchive: () => void
  onDelete: () => void
}) {
  return (
    <div
      className={cn(
        'group flex items-center gap-1 rounded-md',
        active && 'bg-[var(--color-sidebar-active)]',
      )}
    >
      <button
        onClick={onSelect}
        className="min-w-0 flex-1 rounded-md px-3 py-2 text-left focus-ring"
      >
        <span className="block truncate text-sm font-medium">{conversation.title}</span>
        <span className="mt-0.5 block text-xs text-[var(--color-text-tertiary)]">
          {conversation.model === 'deepseek-v4-pro' ? 'V4 Pro' : 'V4 Flash'}
        </span>
      </button>
      <div className="mr-1 flex opacity-0 group-hover:opacity-100 focus-within:opacity-100">
        <button
          onClick={onArchive}
          aria-label={`Archive ${conversation.title}`}
          className="rounded p-1 hover:bg-[var(--color-bg-tertiary)] focus-ring"
        >
          <Archive size={13} />
        </button>
        <button
          onClick={onDelete}
          aria-label={`Delete ${conversation.title}`}
          className="rounded p-1 text-[var(--color-danger)] hover:bg-[var(--color-bg-tertiary)] focus-ring"
        >
          <Trash2 size={13} />
        </button>
      </div>
    </div>
  )
}

export function AiView({ spaceId }: { spaceId?: string }) {
  const list = useAiConversations(spaceId)
  const settings = useAiSettings()
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [model, setModel] = useState('deepseek-v4-flash')
  const [draft, setDraft] = useState('')
  const [mode, setMode] = useState<AiMode>('ask')
  const [showContext, setShowContext] = useState(false)
  const [proposal, setProposal] = useState<TaskProposal | null>(null)
  const [deleteTarget, setDeleteTarget] = useState<AiConversation | null>(null)
  const [renaming, setRenaming] = useState(false)
  const [titleDraft, setTitleDraft] = useState('')
  const endRef = useRef<HTMLDivElement>(null)
  const chat = useAiConversation(selectedId)

  useEffect(() => {
    if (selectedId && list.conversations.some((item) => item.id === selectedId)) return
    setSelectedId(list.conversations[0]?.id ?? null)
  }, [list.conversations, selectedId])
  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [chat.messages])

  const selected = list.conversations.find((item) => item.id === selectedId)
  const canSend =
    !!draft.trim() &&
    !!selectedId &&
    !chat.loading &&
    !chat.streaming &&
    settings.status === 'configured'

  async function createConversation() {
    try {
      const conversation = await list.create(model)
      setSelectedId(conversation.id)
    } catch {
      // List error state is displayed in the sidebar.
    }
  }
  async function send() {
    if (!canSend) return
    const content = draft.trim()
    setDraft('')
    await chat.send(content, undefined, mode)
  }
  async function commitRename() {
    const title = titleDraft.trim()
    if (selected && title && title !== selected.title)
      await list.rename(selected.id, title)
    setRenaming(false)
  }

  if (!list.isTauri) {
    return (
      <div className="flex h-full items-center justify-center p-8 text-center">
        <div>
          <Bot size={28} className="mx-auto mb-4 text-[var(--color-accent)]" />
          <h2 className="text-lg font-semibold">AI is available in the desktop app</h2>
          <p className="mt-2 max-w-sm text-sm text-[var(--color-text-secondary)]">
            Aether keeps credentials and provider requests in the trusted Rust desktop
            process.
          </p>
        </div>
      </div>
    )
  }

  return (
    <div className="flex h-full min-h-0">
      <aside className="flex h-full w-[250px] shrink-0 flex-col border-r border-[var(--color-border)] bg-[var(--color-sidebar-bg)]">
        <div className="border-b border-[var(--color-border)] p-3">
          <div className="flex gap-2">
            <select
              aria-label="Model for new conversation"
              value={model}
              onChange={(event) => setModel(event.target.value)}
              className="min-w-0 flex-1 rounded-md border border-[var(--color-border)] bg-[var(--color-bg)] px-2 py-1.5 text-xs"
            >
              <option value="deepseek-v4-flash">V4 Flash</option>
              <option value="deepseek-v4-pro">V4 Pro</option>
            </select>
            <button
              onClick={() => void createConversation()}
              aria-label="New conversation"
              className="rounded-md bg-[var(--color-accent)] p-2 text-[var(--color-accent-text)] focus-ring"
            >
              <Plus size={15} />
            </button>
          </div>
        </div>
        <div className="min-h-0 flex-1 space-y-0.5 overflow-y-auto p-2">
          {list.loading ? (
            <p className="px-3 py-8 text-center text-xs text-[var(--color-text-tertiary)]">
              Loading…
            </p>
          ) : list.conversations.length === 0 ? (
            <p className="px-3 py-8 text-center text-xs leading-relaxed text-[var(--color-text-tertiary)]">
              No conversations yet. Start one when you need AI.
            </p>
          ) : (
            list.conversations.map((conversation) => (
              <ConversationRow
                key={conversation.id}
                conversation={conversation}
                active={selectedId === conversation.id}
                onSelect={() => setSelectedId(conversation.id)}
                onArchive={() => void list.archive(conversation.id)}
                onDelete={() => setDeleteTarget(conversation)}
              />
            ))
          )}
          {list.error && (
            <p role="alert" className="px-3 py-2 text-xs text-[var(--color-danger)]">
              {list.error}
            </p>
          )}
        </div>
      </aside>

      <section className="flex min-w-0 flex-1 flex-col">
        {!selected ? (
          <div className="flex flex-1 items-center justify-center p-8 text-center">
            <div>
              <Brain size={26} className="mx-auto mb-4 text-[var(--color-accent)]" />
              <h2 className="text-lg font-semibold">Start a focused conversation</h2>
              <p className="mt-2 max-w-sm text-sm text-[var(--color-text-secondary)]">
                Choose Flash for everyday work or Pro for more demanding reasoning.
              </p>
              <button
                onClick={() => void createConversation()}
                className="mt-5 rounded-md bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-[var(--color-accent-text)] focus-ring"
              >
                New conversation
              </button>
            </div>
          </div>
        ) : (
          <>
            <header className="border-b border-[var(--color-border)] px-5 py-3">
              <div className="flex items-center gap-2">
                {renaming ? (
                  <input
                    aria-label="Conversation title"
                    value={titleDraft}
                    onChange={(event) => setTitleDraft(event.target.value)}
                    onBlur={() => void commitRename()}
                    onKeyDown={(event) => {
                      if (event.key === 'Enter') void commitRename()
                      if (event.key === 'Escape') setRenaming(false)
                    }}
                    autoFocus
                    className="min-w-0 flex-1 rounded-md border border-[var(--color-border)] bg-[var(--color-bg)] px-2 py-1 text-sm font-semibold outline-none"
                  />
                ) : (
                  <button
                    onClick={() => {
                      setTitleDraft(selected.title)
                      setRenaming(true)
                    }}
                    className="flex min-w-0 flex-1 items-center gap-2 rounded-sm text-left focus-ring"
                  >
                    <span className="truncate text-sm font-semibold">
                      {selected.title}
                    </span>
                    <Pencil size={12} className="text-[var(--color-text-tertiary)]" />
                  </button>
                )}
                <span className="text-xs text-[var(--color-text-tertiary)]">
                  {selected.model === 'deepseek-v4-pro' ? 'V4 Pro' : 'V4 Flash'}
                </span>
              </div>
              <div className="mt-2 flex flex-wrap items-center gap-1.5">
                <button
                  onClick={() => setShowContext(true)}
                  className="inline-flex items-center gap-1.5 rounded-md border border-[var(--color-border)] px-2 py-1 text-xs hover:bg-[var(--color-bg-tertiary)] focus-ring"
                >
                  <Paperclip size={12} /> Attach context
                </button>
                {chat.resolvedContext.map((item) => (
                  <span
                    key={item.attachmentId}
                    title={item.detail}
                    className="inline-flex items-center gap-1 rounded-md bg-[var(--color-bg-secondary)] px-2 py-1 text-xs"
                  >
                    <span className="capitalize text-[var(--color-text-tertiary)]">
                      {item.entityType}
                    </span>
                    {item.title}
                    <button
                      onClick={() => void chat.detach(item.attachmentId)}
                      aria-label={`Detach ${item.title}`}
                      className="rounded-sm hover:text-[var(--color-danger)] focus-ring"
                    >
                      <X size={11} />
                    </button>
                  </span>
                ))}
              </div>
            </header>
            <div className="min-h-0 flex-1 overflow-y-auto px-6 py-5">
              {chat.loading ? (
                <p className="py-12 text-center text-sm text-[var(--color-text-tertiary)]">
                  Loading messages…
                </p>
              ) : chat.messages.length === 0 ? (
                <div className="mx-auto mt-16 max-w-md text-center">
                  <Bot size={24} className="mx-auto mb-3 text-[var(--color-accent)]" />
                  <h2 className="text-base font-semibold">
                    What would you like to work on?
                  </h2>
                  <p className="mt-2 text-sm leading-relaxed text-[var(--color-text-secondary)]">
                    Aether sends only this conversation and the context shown above to
                    DeepSeek.
                  </p>
                </div>
              ) : (
                <div className="mx-auto max-w-[760px] space-y-5">
                  {chat.messages.map((message, index) => {
                    const proposals =
                      message.role === 'assistant' && message.status === 'complete'
                        ? parseTaskProposal(message.content)
                        : null
                    const retryUser =
                      message.role === 'assistant' &&
                      ['error', 'cancelled'].includes(message.status)
                        ? precedingUser(chat.messages, index)
                        : undefined
                    return (
                      <article
                        key={message.id}
                        className={cn(
                          'flex',
                          message.role === 'user' ? 'justify-end' : 'justify-start',
                        )}
                      >
                        <div
                          className={cn(
                            'max-w-[82%] rounded-xl px-4 py-3 text-sm leading-relaxed',
                            message.role === 'user'
                              ? 'bg-[var(--color-accent)] text-[var(--color-accent-text)]'
                              : 'border border-[var(--color-border)] bg-[var(--color-bg-secondary)] text-[var(--color-text-primary)]',
                          )}
                        >
                          <div className="whitespace-pre-wrap break-words">
                            {message.content ||
                              (message.status === 'streaming'
                                ? 'Thinking…'
                                : 'No response')}
                          </div>
                          {message.status === 'cancelled' && (
                            <p className="mt-2 text-xs text-[var(--color-warning)]">
                              Response cancelled
                            </p>
                          )}
                          {message.status === 'error' && (
                            <p className="mt-2 text-xs text-[var(--color-danger)]">
                              Response failed
                            </p>
                          )}
                          <div className="mt-2 flex gap-2">
                            {retryUser && !chat.streaming && (
                              <button
                                onClick={() =>
                                  void chat.send(
                                    '',
                                    retryUser.id,
                                    modeFromMessage(message),
                                  )
                                }
                                className="inline-flex items-center gap-1 text-xs font-medium text-[var(--color-accent)]"
                              >
                                <RefreshCw size={11} /> Retry
                              </button>
                            )}
                            {proposals && (
                              <button
                                onClick={() => setProposal(proposals)}
                                className="inline-flex items-center gap-1 text-xs font-medium text-[var(--color-accent)]"
                              >
                                <CheckSquare size={12} /> Review proposed Tasks
                              </button>
                            )}
                          </div>
                        </div>
                      </article>
                    )
                  })}
                  <div ref={endRef} />
                </div>
              )}
              {chat.error && (
                <p
                  role="alert"
                  className="mx-auto mt-4 max-w-[760px] text-sm text-[var(--color-danger)]"
                >
                  {chat.error}
                </p>
              )}
            </div>
            <footer className="border-t border-[var(--color-border)] px-5 py-4">
              <div className="mx-auto max-w-[780px]">
                <div className="mb-2 flex flex-wrap gap-1">
                  {MODES.map((candidate) => (
                    <button
                      key={candidate.id}
                      onClick={() => setMode(candidate.id)}
                      className={cn(
                        'rounded-md px-2 py-1 text-xs focus-ring',
                        mode === candidate.id
                          ? 'bg-[var(--color-accent-muted)] text-[var(--color-accent)]'
                          : 'text-[var(--color-text-tertiary)] hover:bg-[var(--color-bg-tertiary)]',
                      )}
                    >
                      {candidate.label}
                    </button>
                  ))}
                </div>
                {settings.status !== 'configured' && (
                  <p className="mb-2 text-xs text-[var(--color-warning)]">
                    Configure a DeepSeek API key in Settings before sending.
                  </p>
                )}
                <div className="flex items-end gap-2 rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] p-2">
                  <textarea
                    aria-label="Message"
                    value={draft}
                    onChange={(event) => setDraft(event.target.value)}
                    onKeyDown={(event) => {
                      if (event.key === 'Enter' && !event.shiftKey) {
                        event.preventDefault()
                        void send()
                      }
                    }}
                    placeholder={
                      mode === 'create_tasks'
                        ? 'Describe the work you want broken into Tasks…'
                        : 'Message DeepSeek…'
                    }
                    rows={2}
                    maxLength={32000}
                    className="max-h-40 min-h-12 flex-1 resize-none bg-transparent px-2 py-1 text-sm outline-none"
                  />
                  {chat.streaming ? (
                    <button
                      onClick={() => void chat.cancel()}
                      aria-label="Cancel response"
                      className="rounded-md bg-[var(--color-danger)] p-2.5 text-white focus-ring"
                    >
                      <Square size={14} fill="currentColor" />
                    </button>
                  ) : (
                    <button
                      onClick={() => void send()}
                      disabled={!canSend}
                      aria-label="Send message"
                      className="rounded-md bg-[var(--color-accent)] p-2.5 text-[var(--color-accent-text)] disabled:opacity-40 focus-ring"
                    >
                      <Send size={15} />
                    </button>
                  )}
                </div>
                <p className="mt-2 text-center text-[10px] text-[var(--color-text-tertiary)]">
                  Conversation history and visible attachments are sent to DeepSeek. AI
                  can make mistakes.
                </p>
              </div>
            </footer>
          </>
        )}
      </section>

      {showContext && selectedId && (
        <AiContextPicker
          spaceId={spaceId}
          attached={chat.contextItems}
          onAttach={chat.attach}
          onClose={() => setShowContext(false)}
        />
      )}
      {proposal && (
        <AiTaskProposal
          tasks={proposal}
          spaceId={spaceId}
          onClose={() => setProposal(null)}
          onCreated={() => setProposal(null)}
        />
      )}
      {deleteTarget && (
        <ConfirmDialog
          title={`Delete “${deleteTarget.title}”?`}
          message="This permanently deletes the conversation, its messages, and its attachment references. Local Notes, Tasks, and files are not deleted."
          confirmLabel="Delete conversation"
          danger
          onConfirm={() => {
            const id = deleteTarget.id
            setDeleteTarget(null)
            if (selectedId === id) setSelectedId(null)
            void list.remove(id)
          }}
          onCancel={() => setDeleteTarget(null)}
        />
      )}
    </div>
  )
}
