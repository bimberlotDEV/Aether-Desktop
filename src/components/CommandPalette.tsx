import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import {
  Activity,
  Brain,
  CheckSquare,
  Database,
  FileText,
  FolderOpen,
  Layers,
  LoaderCircle,
  MemoryStick,
  MessageSquare,
  Moon,
  Search,
  Settings,
  Sparkles,
  Sun,
  type LucideIcon,
} from 'lucide-react'
import { useSpaces } from '@/hooks/useSpaces'
import * as db from '@/lib/db/tauri'
import type { UniversalSearchKind, UniversalSearchResult } from '@/lib/db/types'
import { cn } from '@/lib/utils'
import { useCommandStore, type Command } from '@/stores/commandStore'
import { useThemeStore } from '@/stores/themeStore'

function navigateTo(path: string) {
  window.history.pushState({}, '', path)
  window.dispatchEvent(new PopStateEvent('popstate'))
}

const defaultCommands: Command[] = [
  command('pulse', 'Go to Pulse', 'Open today’s workspace', ['home', 'dashboard'], '/'),
  command('spaces', 'Go to Spaces', 'View and manage Spaces', ['workspaces'], '/spaces'),
  command(
    'tasks',
    'Go to Tasks',
    'Open your global Task Inbox',
    ['todo', 'inbox'],
    '/tasks',
  ),
  command(
    'vault',
    'Go to Vault',
    'Browse managed and linked files',
    ['documents'],
    '/vault',
  ),
  command(
    'activity',
    'Go to Activity',
    'View meaningful recent activity',
    ['history'],
    '/activity',
  ),
  command('ai', 'Open AI', 'Start or continue a DeepSeek conversation', ['chat'], '/ai'),
  command(
    'memory',
    'Open Memory',
    'Review explicit context Aether remembers',
    ['context'],
    '/memory',
  ),
  command(
    'sources',
    'Open Sources',
    'Manage authorized local folders',
    ['index'],
    '/sources',
  ),
  command(
    'settings',
    'Open Settings',
    'Configure Aether preferences',
    ['config'],
    '/settings',
  ),
  {
    id: 'theme-light',
    label: 'Light Theme',
    description: 'Switch to light mode',
    keywords: ['appearance'],
    action: () => useThemeStore.getState().setTheme('light'),
  },
  {
    id: 'theme-dark',
    label: 'Dark Theme',
    description: 'Switch to dark mode',
    keywords: ['appearance'],
    action: () => useThemeStore.getState().setTheme('dark'),
  },
  {
    id: 'theme-system',
    label: 'System Theme',
    description: 'Follow system preference',
    keywords: ['appearance', 'auto'],
    action: () => useThemeStore.getState().setTheme('system'),
  },
]

function command(
  id: string,
  label: string,
  description: string,
  keywords: string[],
  path: string,
): Command {
  return { id, label, description, keywords, action: () => navigateTo(path) }
}

type PaletteItem = {
  id: string
  label: string
  description?: string
  provenance: string
  icon: LucideIcon
  action: () => void
}

const resultIcons: Record<UniversalSearchKind, LucideIcon> = {
  space: Layers,
  note: FileText,
  task: CheckSquare,
  vault: FolderOpen,
  memory: Brain,
  conversation: MessageSquare,
  activity: Activity,
  file: Database,
}

export function CommandPalette() {
  const { isOpen, query, close, setQuery } = useCommandStore()
  const { spaces } = useSpaces()
  const [selectedIndex, setSelectedIndex] = useState(0)
  const [results, setResults] = useState<UniversalSearchResult[]>([])
  const [searching, setSearching] = useState(false)
  const [searchError, setSearchError] = useState<string | null>(null)
  const inputRef = useRef<HTMLInputElement>(null)
  const requestRef = useRef(0)
  const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  const currentSpaceId = currentSpaceFromPath(window.location.pathname)

  const spaceCommands = useMemo<Command[]>(
    () =>
      spaces
        .filter((space) => !space.archived_at)
        .map((space) => ({
          id: `space-${space.id}`,
          label: space.name,
          description: space.description || `Open ${space.name}`,
          keywords: [space.name, space.template_type || ''],
          action: () => navigateTo(`/spaces/${space.id}`),
        })),
    [spaces],
  )

  const commandItems = useMemo<PaletteItem[]>(() => {
    const normalized = query.trim().toLowerCase()
    return [...defaultCommands, ...spaceCommands]
      .filter((item) => {
        if (!normalized) return true
        if (isTauri && normalized.length >= 2 && item.id.startsWith('space-'))
          return false
        return (
          item.label.toLowerCase().includes(normalized) ||
          item.description?.toLowerCase().includes(normalized) ||
          item.keywords?.some((keyword) => keyword.toLowerCase().includes(normalized))
        )
      })
      .map((item) => ({
        id: `command-${item.id}`,
        label: item.label,
        description: item.description,
        provenance: item.id.startsWith('space-') ? 'Space' : 'Command',
        icon: commandIcon(item.label),
        action: item.action,
      }))
  }, [isTauri, query, spaceCommands])

  useEffect(() => {
    const normalized = query.trim()
    const request = ++requestRef.current
    if (!isOpen || !isTauri || normalized.length < 2) {
      setResults([])
      setSearching(false)
      setSearchError(null)
      return
    }
    setSearching(true)
    setSearchError(null)
    const timer = window.setTimeout(() => {
      void db
        .universalSearch(normalized, currentSpaceId, 30)
        .then((next) => {
          if (request === requestRef.current) setResults(next)
        })
        .catch((cause) => {
          if (request === requestRef.current) {
            setResults([])
            setSearchError(message(cause))
          }
        })
        .finally(() => {
          if (request === requestRef.current) setSearching(false)
        })
    }, 140)
    return () => window.clearTimeout(timer)
  }, [currentSpaceId, isOpen, isTauri, query])

  const searchItems = useMemo<PaletteItem[]>(
    () =>
      results.map((result) => ({
        id: `result-${result.kind}-${result.entityId}`,
        label: result.title,
        description: result.subtitle,
        provenance: result.provenance,
        icon: resultIcons[result.kind],
        action: () => navigateTo(resultDestination(result)),
      })),
    [results],
  )
  const items = useMemo(
    () => [...commandItems, ...searchItems],
    [commandItems, searchItems],
  )

  useEffect(() => setSelectedIndex(0), [query])
  useEffect(() => {
    if (selectedIndex >= items.length) setSelectedIndex(Math.max(0, items.length - 1))
  }, [items.length, selectedIndex])
  useEffect(() => {
    if (isOpen) window.setTimeout(() => inputRef.current?.focus(), 50)
  }, [isOpen])

  const execute = useCallback(
    (item: PaletteItem) => {
      item.action()
      close()
    },
    [close],
  )
  const handleKeyDown = useCallback(
    (event: React.KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault()
        close()
      } else if (event.key === 'ArrowDown') {
        event.preventDefault()
        setSelectedIndex((index) => Math.min(index + 1, Math.max(0, items.length - 1)))
      } else if (event.key === 'ArrowUp') {
        event.preventDefault()
        setSelectedIndex((index) => Math.max(index - 1, 0))
      } else if (event.key === 'Enter' && items[selectedIndex]) {
        event.preventDefault()
        execute(items[selectedIndex])
      }
    },
    [close, execute, items, selectedIndex],
  )

  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key === 'k') {
        event.preventDefault()
        useCommandStore.getState().toggle()
      }
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [])

  if (!isOpen) return null
  const noResults = query.trim().length > 0 && items.length === 0 && !searching

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center pt-[12vh]"
      onClick={close}
    >
      <div className="absolute inset-0 bg-[var(--color-bg-overlay)]" />
      <div
        role="dialog"
        aria-modal="true"
        aria-label="Universal Search"
        className="aether-command-palette"
        onClick={(event) => event.stopPropagation()}
      >
        <div className="flex h-13 items-center gap-3 border-b border-[var(--color-border)] px-4">
          {searching ? (
            <LoaderCircle size={16} className="animate-spin text-[var(--color-accent)]" />
          ) : (
            <Search size={16} className="text-[var(--color-text-tertiary)]" />
          )}
          <input
            ref={inputRef}
            type="search"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Search Aether…"
            aria-label="Search commands and local workspace"
            aria-controls="universal-search-results"
            aria-activedescendant={items[selectedIndex]?.id}
            className="flex-1 bg-transparent text-sm text-[var(--color-text-primary)] outline-none"
            autoComplete="off"
            spellCheck={false}
          />
          <span className="text-[10px] text-[var(--color-text-tertiary)]">
            {isTauri ? 'LOCAL' : 'COMMANDS'}
          </span>
          <kbd className="aether-badge">ESC</kbd>
        </div>

        {searchError && (
          <p
            role="alert"
            className="border-b border-[var(--color-border)] px-4 py-2 text-xs text-[var(--color-danger)]"
          >
            Search is temporarily unavailable. Commands still work.
          </p>
        )}
        {!isTauri && query.trim().length >= 2 && (
          <p className="border-b border-[var(--color-border)] px-4 py-2 text-xs text-[var(--color-text-tertiary)]">
            Installed Aether searches your local workspace. Browser mode searches commands
            only.
          </p>
        )}

        <div
          id="universal-search-results"
          role="listbox"
          className="max-h-[420px] overflow-y-auto p-2"
        >
          {noResults ? (
            <div className="px-4 py-10 text-center">
              <p className="text-sm font-medium">Nothing found</p>
              <p className="mt-1 text-xs text-[var(--color-text-tertiary)]">
                Try a title, filename, Task, Space, or phrase from a Note.
              </p>
            </div>
          ) : (
            items.map((item, index) => {
              const Icon = item.icon
              return (
                <button
                  key={item.id}
                  id={item.id}
                  role="option"
                  aria-selected={index === selectedIndex}
                  className={cn(
                    'aether-command-result',
                    index === selectedIndex
                      ? 'bg-[var(--color-accent-muted)]'
                      : 'hover:bg-[var(--color-bg-tertiary)]',
                  )}
                  onClick={() => execute(item)}
                  onMouseEnter={() => setSelectedIndex(index)}
                >
                  <Icon
                    size={17}
                    strokeWidth={1.75}
                    className={cn(
                      'shrink-0',
                      index === selectedIndex
                        ? 'text-[var(--color-accent)]'
                        : 'text-[var(--color-text-tertiary)]',
                    )}
                  />
                  <span className="min-w-0 flex-1 text-left">
                    <span className="block truncate text-sm font-medium">
                      {item.label}
                    </span>
                    {item.description && (
                      <span className="block truncate text-xs text-[var(--color-text-tertiary)]">
                        {item.description}
                      </span>
                    )}
                  </span>
                  <span className="aether-badge shrink-0">{item.provenance}</span>
                </button>
              )
            })
          )}
        </div>
        <div className="flex items-center justify-between border-t border-[var(--color-border)] px-4 py-2 text-[10px] text-[var(--color-text-tertiary)]">
          <span>↑↓ navigate · Enter open</span>
          <span>{isTauri ? 'Private, on this device' : 'Desktop data unavailable'}</span>
        </div>
      </div>
    </div>
  )
}

function commandIcon(label: string): LucideIcon {
  if (label.includes('Theme'))
    return label.includes('Light') ? Sun : label.includes('Dark') ? Moon : MemoryStick
  if (label.includes('AI')) return Sparkles
  if (label.includes('Memory')) return Brain
  if (label.includes('Sources')) return Database
  if (label.includes('Tasks')) return CheckSquare
  if (label.includes('Vault')) return FolderOpen
  if (label.includes('Settings')) return Settings
  return Layers
}

function currentSpaceFromPath(path: string) {
  const match = path.match(/^\/spaces\/([^/]+)/)
  return match?.[1] ?? null
}

function resultDestination(result: UniversalSearchResult) {
  switch (result.kind) {
    case 'space':
      return `/spaces/${result.entityId}`
    case 'note':
      return result.spaceId ? `/spaces/${result.spaceId}/notes` : '/spaces'
    case 'task':
      return result.spaceId ? `/spaces/${result.spaceId}/tasks` : '/tasks'
    case 'vault':
      return result.spaceId ? `/spaces/${result.spaceId}/files` : '/vault'
    case 'memory':
      return result.spaceId ? `/spaces/${result.spaceId}/memory` : '/memory'
    case 'conversation':
      return result.spaceId ? `/spaces/${result.spaceId}/ai` : '/ai'
    case 'file':
      return '/sources'
    case 'activity':
      return '/activity'
  }
}

function message(cause: unknown) {
  return cause instanceof Error ? cause.message : String(cause)
}
