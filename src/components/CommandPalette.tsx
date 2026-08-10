import { useEffect, useRef, useState, useCallback, useMemo } from 'react'
import { useCommandStore, type Command } from '@/stores/commandStore'
import { cn } from '@/lib/utils'
import {
  Search,
  Settings,
  Sun,
  Moon,
  Monitor,
  Plus,
  CheckSquare,
  LayoutDashboard,
  Layers,
  FolderOpen,
  Sparkles,
  type LucideIcon,
} from 'lucide-react'
import { useThemeStore } from '@/stores/themeStore'
import { useSpaces } from '@/hooks/useSpaces'

function navigateTo(path: string) {
  window.history.pushState({}, '', path)
  window.dispatchEvent(new PopStateEvent('popstate'))
}

// Default commands
const defaultCommands: Command[] = [
  {
    id: 'pulse',
    label: 'Go to Pulse',
    description: 'Navigate to the home dashboard',
    keywords: ['home', 'dashboard'],
    action: () => navigateTo('/'),
  },
  {
    id: 'spaces',
    label: 'Go to Spaces',
    description: 'View and manage your spaces',
    keywords: ['workspaces'],
    action: () => navigateTo('/spaces'),
  },
  {
    id: 'vault',
    label: 'Go to Vault',
    description: 'Browse files and knowledge',
    keywords: ['files', 'documents'],
    action: () => navigateTo('/vault'),
  },
  {
    id: 'activity',
    label: 'Go to Activity',
    description: 'View recent activity',
    keywords: ['history', 'timeline'],
    action: () => navigateTo('/activity'),
  },
  {
    id: 'ai',
    label: 'Open AI',
    description: 'Start or continue a DeepSeek conversation',
    keywords: ['chat', 'deepseek', 'assistant'],
    action: () => navigateTo('/ai'),
  },
  {
    id: 'create-note',
    label: 'Create Note',
    description: 'Create a new note',
    keywords: ['note', 'new', 'write'],
    action: () => navigateTo('/spaces'),
  },
  {
    id: 'tasks',
    label: 'Go to Tasks',
    description: 'Open your global Task Inbox',
    keywords: ['tasks', 'todo', 'inbox'],
    action: () => navigateTo('/tasks'),
  },
  {
    id: 'search-notes',
    label: 'Search Notes',
    description: 'Search through all notes',
    keywords: ['find', 'note', 'text'],
    action: () => navigateTo('/spaces'),
  },
  {
    id: 'settings',
    label: 'Open Settings',
    description: 'Configure Aether preferences',
    keywords: ['preferences', 'config'],
    action: () => navigateTo('/settings'),
  },
  {
    id: 'theme-light',
    label: 'Light Theme',
    description: 'Switch to light mode',
    keywords: ['appearance', 'mode'],
    action: () => {
      useThemeStore.getState().setTheme('light')
    },
  },
  {
    id: 'theme-dark',
    label: 'Dark Theme',
    description: 'Switch to dark mode',
    keywords: ['appearance', 'mode'],
    action: () => {
      useThemeStore.getState().setTheme('dark')
    },
  },
  {
    id: 'theme-system',
    label: 'System Theme',
    description: 'Follow system preference',
    keywords: ['appearance', 'auto'],
    action: () => {
      useThemeStore.getState().setTheme('system')
    },
  },
]

const iconMap: Record<string, LucideIcon> = {
  'Go to': Layers,
  Navigate: LayoutDashboard,
  Browse: FolderOpen,
  View: Layers,
  Open: Settings,
  AI: Sparkles,
  Create: Plus,
  Tasks: CheckSquare,
  Switch: Sun,
  Light: Sun,
  Dark: Moon,
  System: Monitor,
}

function getIcon(label: string): LucideIcon {
  if (label.includes('AI')) return Sparkles
  for (const [key, icon] of Object.entries(iconMap)) {
    if (label.includes(key)) return icon
  }
  return Search
}

export function CommandPalette() {
  const { isOpen, query, close, setQuery } = useCommandStore()
  const [selectedIndex, setSelectedIndex] = useState(0)
  const inputRef = useRef<HTMLInputElement>(null)
  const listRef = useRef<HTMLDivElement>(null)

  const { spaces } = useSpaces()
  const spaceCommands = useMemo(() => {
    return spaces
      .filter((s) => !s.archived_at)
      .map((s) => ({
        id: `space-${s.id}`,
        label: s.name,
        description: s.description || `Open ${s.name}`,
        keywords: [s.name, s.template_type || ''],
        action: () => navigateTo(`/spaces/${s.id}`),
      }))
  }, [spaces])

  const allCommands = useMemo(() => {
    return [...defaultCommands, ...spaceCommands]
  }, [spaceCommands])

  const filtered = useMemo(() => {
    if (!query.trim()) return allCommands
    const q = query.toLowerCase()
    return allCommands.filter(
      (c) =>
        c.label.toLowerCase().includes(q) ||
        c.description?.toLowerCase().includes(q) ||
        c.keywords?.some((k) => k.toLowerCase().includes(q)),
    )
  }, [query, allCommands])

  // Reset selection when query changes
  useEffect(() => {
    setSelectedIndex(0)
  }, [query])

  // Focus input when opened
  useEffect(() => {
    if (isOpen) {
      setTimeout(() => inputRef.current?.focus(), 50)
    }
  }, [isOpen])

  const execute = useCallback(
    (cmd: Command) => {
      cmd.action()
      close()
    },
    [close],
  )

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault()
        close()
      } else if (e.key === 'ArrowDown') {
        e.preventDefault()
        setSelectedIndex((i) => Math.min(i + 1, filtered.length - 1))
      } else if (e.key === 'ArrowUp') {
        e.preventDefault()
        setSelectedIndex((i) => Math.max(i - 1, 0))
      } else if (e.key === 'Enter') {
        e.preventDefault()
        if (filtered[selectedIndex]) execute(filtered[selectedIndex])
      }
    },
    [filtered, selectedIndex, execute, close],
  )

  // Global keyboard shortcut
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault()
        const store = useCommandStore.getState()
        store.toggle()
      }
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [])

  if (!isOpen) return null

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center pt-[15vh]"
      onClick={close}
    >
      {/* Backdrop */}
      <div
        className="absolute inset-0"
        style={{ backgroundColor: 'var(--color-bg-overlay)' }}
      />

      {/* Palette */}
      <div
        className="relative w-full max-w-[520px] rounded-xl shadow-2xl glass-surface overflow-hidden"
        onClick={(e) => e.stopPropagation()}
        style={{
          animation: 'commandIn 150ms var(--ease-out)',
        }}
      >
        {/* Search input */}
        <div
          className="flex items-center gap-3 px-4 h-12"
          style={{ borderBottom: `1px solid var(--color-border)` }}
        >
          <Search size={16} style={{ color: 'var(--color-text-tertiary)' }} />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Search commands..."
            className="flex-1 bg-transparent outline-none text-sm"
            style={{ color: 'var(--color-text-primary)' }}
            autoComplete="off"
            spellCheck={false}
          />
          <kbd
            className="text-[10px] font-medium px-1.5 py-0.5 rounded"
            style={{
              backgroundColor: 'var(--color-bg-tertiary)',
              color: 'var(--color-text-tertiary)',
              border: `1px solid var(--color-border)`,
            }}
          >
            ESC
          </kbd>
        </div>

        {/* Results */}
        <div ref={listRef} className="max-h-[320px] overflow-y-auto p-2">
          {filtered.length === 0 ? (
            <div
              className="py-8 text-center text-sm"
              style={{ color: 'var(--color-text-tertiary)' }}
            >
              No commands found
            </div>
          ) : (
            filtered.map((cmd, i) => {
              const Icon = getIcon(cmd.label)
              return (
                <button
                  key={cmd.id}
                  className={cn(
                    'flex items-center gap-3 w-full px-3 py-2.5 rounded-lg text-left transition-colors duration-75',
                    i === selectedIndex
                      ? 'bg-[var(--color-accent-muted)]'
                      : 'hover:bg-[var(--color-bg-tertiary)]',
                  )}
                  onClick={() => execute(cmd)}
                  onMouseEnter={() => setSelectedIndex(i)}
                >
                  <Icon
                    size={17}
                    strokeWidth={1.75}
                    style={{
                      color:
                        i === selectedIndex
                          ? 'var(--color-accent)'
                          : 'var(--color-text-tertiary)',
                    }}
                  />
                  <div className="flex-1 min-w-0">
                    <div
                      className="text-sm font-medium truncate"
                      style={{ color: 'var(--color-text-primary)' }}
                    >
                      {cmd.label}
                    </div>
                    {cmd.description && (
                      <div
                        className="text-xs truncate"
                        style={{ color: 'var(--color-text-tertiary)' }}
                      >
                        {cmd.description}
                      </div>
                    )}
                  </div>
                </button>
              )
            })
          )}
        </div>
      </div>

      <style>{`
        @keyframes commandIn {
          from {
            opacity: 0;
            transform: scale(0.97) translateY(-8px);
          }
          to {
            opacity: 1;
            transform: scale(1) translateY(0);
          }
        }
      `}</style>
    </div>
  )
}
