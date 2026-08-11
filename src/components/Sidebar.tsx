import { NavLink } from 'react-router-dom'
import {
  LayoutDashboard,
  Layers,
  FolderOpen,
  Activity,
  Settings,
  CheckSquare,
  Sparkles,
  Brain,
  type LucideIcon,
} from 'lucide-react'
import { cn } from '@/lib/utils'

interface NavItem {
  to: string
  icon: LucideIcon
  label: string
}

const navItems: NavItem[] = [
  { to: '/', icon: LayoutDashboard, label: 'Pulse' },
  { to: '/spaces', icon: Layers, label: 'Spaces' },
  { to: '/tasks', icon: CheckSquare, label: 'Tasks' },
  { to: '/vault', icon: FolderOpen, label: 'Vault' },
  { to: '/ai', icon: Sparkles, label: 'AI' },
  { to: '/memory', icon: Brain, label: 'Memory' },
  { to: '/activity', icon: Activity, label: 'Activity' },
  { to: '/settings', icon: Settings, label: 'Settings' },
]

export function Sidebar() {
  return (
    <aside
      className="flex flex-col h-full border-r select-none"
      style={{
        width: 208,
        backgroundColor: 'var(--color-sidebar-bg)',
        borderColor: 'var(--color-sidebar-border)',
      }}
    >
      {/* Wordmark */}
      <div
        className="flex items-center gap-2.5 px-4 h-12 shrink-0"
        style={{ borderBottom: `1px solid var(--color-sidebar-border)` }}
      >
        <span
          className="text-sm font-semibold tracking-tight"
          style={{ color: 'var(--color-text-primary)' }}
        >
          Aether
        </span>
      </div>

      {/* Navigation */}
      <nav className="flex-1 px-2 py-3 space-y-0.5 overflow-y-auto">
        {navItems.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            end={item.to === '/'}
            className={({ isActive }) =>
              cn(
                'flex items-center gap-2.5 px-2.5 py-1.5 rounded-md text-sm font-medium transition-colors duration-100',
                isActive
                  ? 'bg-[var(--color-sidebar-active)] text-[var(--color-accent)]'
                  : 'text-[var(--color-text-secondary)] hover:bg-[var(--color-sidebar-hover)] hover:text-[var(--color-text-primary)]',
              )
            }
          >
            <item.icon size={17} strokeWidth={1.75} />
            {item.label}
          </NavLink>
        ))}
      </nav>

      {/* Bottom — subtle version / status */}
      <div
        className="px-4 py-3 shrink-0"
        style={{ borderTop: `1px solid var(--color-sidebar-border)` }}
      >
        <p className="text-xs" style={{ color: 'var(--color-text-tertiary)' }}>
          Alpha · v0.3.1
        </p>
      </div>
    </aside>
  )
}
