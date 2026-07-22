import { useState, useEffect } from 'react'
import { NavLink, useParams } from 'react-router-dom'
import {
  LayoutDashboard, Layers, FolderOpen, Activity, Settings,
  Plus, Star, ChevronDown, ChevronRight, type LucideIcon,
} from 'lucide-react'
import { cn } from '@/lib/utils'
import type { Space } from '@/lib/db/types'
import { useSpaces } from '@/hooks/useSpaces'

interface NavItem {
  to: string
  icon: LucideIcon
  label: string
}

const staticItems: NavItem[] = [
  { to: '/', icon: LayoutDashboard, label: 'Pulse' },
  { to: '/spaces', icon: Layers, label: 'Spaces' },
  { to: '/vault', icon: FolderOpen, label: 'Vault' },
  { to: '/activity', icon: Activity, label: 'Activity' },
  { to: '/settings', icon: Settings, label: 'Settings' },
]

export function Sidebar() {
  const { spaceId } = useParams()
  const { spaces, loading } = useSpaces()
  const [expandedParents, setExpandedParents] = useState<Set<string>>(new Set())

  const favourites = spaces.filter(s => s.favourite && !s.archived_at).slice(0, 5)
  const topLevel = spaces.filter(s => !s.parent_space_id && !s.archived_at && !s.favourite)

  // Auto-expand parent of current space
  useEffect(() => {
    if (spaceId) {
      const current = spaces.find(s => s.id === spaceId)
      if (current?.parent_space_id) {
        setExpandedParents(prev => new Set([...prev, current.parent_space_id!]))
      }
    }
  }, [spaceId, spaces])

  const toggleParent = (id: string) => {
    setExpandedParents(prev => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  }

  return (
    <aside
      className="flex flex-col h-full border-r select-none"
      style={{
        width: 220,
        backgroundColor: 'var(--color-sidebar-bg)',
        borderColor: 'var(--color-sidebar-border)',
      }}
    >
      {/* Wordmark */}
      <div
        className="flex items-center gap-2.5 px-4 h-12 shrink-0"
        style={{ borderBottom: `1px solid var(--color-sidebar-border)` }}
      >
        <span className="text-sm font-semibold tracking-tight" style={{ color: 'var(--color-text-primary)' }}>
          Aether
        </span>
      </div>

      {/* Static navigation */}
      <nav className="px-2 py-2 space-y-0.5">
        {staticItems.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            end={item.to === '/'}
            className={({ isActive }) =>
              cn(
                'flex items-center gap-2.5 px-2.5 py-1.5 rounded-md text-sm font-medium transition-colors duration-100',
                isActive
                  ? 'bg-[var(--color-sidebar-active)] text-[var(--color-accent)]'
                  : 'text-[var(--color-text-secondary)] hover:bg-[var(--color-sidebar-hover)] hover:text-[var(--color-text-primary)]'
              )
            }
          >
            <item.icon size={17} strokeWidth={1.75} />
            {item.label}
          </NavLink>
        ))}
      </nav>

      {/* Divider + Favourites */}
      {favourites.length > 0 && (
        <>
          <div className="mx-3 my-1" style={{ borderTop: '1px solid var(--color-sidebar-border)' }} />
          <div className="px-3 py-1">
            <span className="text-[10px] font-semibold uppercase tracking-wider" style={{ color: 'var(--color-text-tertiary)' }}>
              Favourites
            </span>
          </div>
          <nav className="px-2 space-y-0.5">
            {favourites.map(s => (
              <NavLink
                key={s.id}
                to={`/spaces/${s.id}`}
                className={({ isActive }) =>
                  cn(
                    'flex items-center gap-2.5 px-2.5 py-1.5 rounded-md text-sm font-medium transition-colors duration-100',
                    isActive
                      ? 'bg-[var(--color-sidebar-active)] text-[var(--color-accent)]'
                      : 'text-[var(--color-text-secondary)] hover:bg-[var(--color-sidebar-hover)] hover:text-[var(--color-text-primary)]'
                  )
                }
              >
                <Star size={13} fill="var(--color-warning)" style={{ color: 'var(--color-warning)', flexShrink: 0 }} />
                <span className="truncate">{s.name}</span>
              </NavLink>
            ))}
          </nav>
        </>
      )}

      {/* Divider + All Spaces */}
      {topLevel.length > 0 && (
        <>
          <div className="mx-3 my-1" style={{ borderTop: '1px solid var(--color-sidebar-border)' }} />
          <div className="px-3 py-1 flex items-center justify-between">
            <span className="text-[10px] font-semibold uppercase tracking-wider" style={{ color: 'var(--color-text-tertiary)' }}>
              Spaces
            </span>
            <NavLink to="/spaces" className="p-0.5 rounded hover:bg-[var(--color-sidebar-hover)]" style={{ color: 'var(--color-text-tertiary)' }}>
              <Plus size={12} />
            </NavLink>
          </div>
          <nav className="px-2 space-y-0.5 overflow-y-auto flex-1">
            {topLevel.map(s => {
              const children = spaces.filter(c => c.parent_space_id === s.id && !c.archived_at)
              const hasChildren = children.length > 0
              const isExpanded = expandedParents.has(s.id)

              return (
                <div key={s.id}>
                  <NavLink
                    to={`/spaces/${s.id}`}
                    className={({ isActive }) =>
                      cn(
                        'flex items-center gap-2.5 px-2.5 py-1.5 rounded-md text-sm font-medium transition-colors duration-100 group',
                        isActive
                          ? 'bg-[var(--color-sidebar-active)] text-[var(--color-accent)]'
                          : 'text-[var(--color-text-secondary)] hover:bg-[var(--color-sidebar-hover)] hover:text-[var(--color-text-primary)]'
                      )
                    }
                  >
                    <span className="truncate flex-1">{s.name}</span>
                    {hasChildren && (
                      <button
                        onClick={(e) => { e.preventDefault(); e.stopPropagation(); toggleParent(s.id) }}
                        className="p-0.5 rounded opacity-0 group-hover:opacity-100 transition-opacity hover:bg-[var(--color-bg-tertiary)]"
                      >
                        {isExpanded ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
                      </button>
                    )}
                  </NavLink>
                  {hasChildren && isExpanded && (
                    <div className="ml-3 pl-3" style={{ borderLeft: '1px solid var(--color-sidebar-border)' }}>
                      {children.map(c => (
                        <NavLink
                          key={c.id}
                          to={`/spaces/${c.id}`}
                          className={({ isActive }) =>
                            cn(
                              'flex items-center gap-2 px-2.5 py-1 rounded-md text-[13px] font-medium transition-colors duration-100',
                              isActive
                                ? 'text-[var(--color-accent)]'
                                : 'text-[var(--color-text-tertiary)] hover:text-[var(--color-text-secondary)]'
                            )
                          }
                        >
                          <span className="truncate">{c.name}</span>
                        </NavLink>
                      ))}
                    </div>
                  )}
                </div>
              )
            })}
          </nav>
        </>
      )}

      {/* Bottom */}
      <div
        className="px-4 py-3 shrink-0"
        style={{ borderTop: `1px solid var(--color-sidebar-border)` }}
      >
        <p className="text-xs" style={{ color: 'var(--color-text-tertiary)' }}>
          Alpha · v0.3.0
        </p>
      </div>
    </aside>
  )
}
