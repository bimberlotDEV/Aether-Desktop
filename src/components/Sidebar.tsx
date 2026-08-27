import { NavLink } from 'react-router-dom'
import {
  Activity,
  Brain,
  CheckSquare,
  Command,
  FolderOpen,
  Layers,
  LayoutDashboard,
  Search,
  Database,
  Settings,
  Sparkles,
  ShieldCheck,
  type LucideIcon,
} from 'lucide-react'
import { cn } from '@/lib/utils'
import { useCommandStore } from '@/stores/commandStore'
import { StatusDot } from '@/components/ui/AetherUI'

interface NavItem {
  to: string
  icon: LucideIcon
  label: string
}

const workspaceItems: NavItem[] = [
  { to: '/', icon: LayoutDashboard, label: 'Pulse' },
  { to: '/spaces', icon: Layers, label: 'Spaces' },
  { to: '/tasks', icon: CheckSquare, label: 'Tasks' },
  { to: '/vault', icon: FolderOpen, label: 'Vault' },
]

const intelligenceItems: NavItem[] = [
  { to: '/sources', icon: Database, label: 'Sources' },
  { to: '/ai', icon: Sparkles, label: 'AI' },
  { to: '/actions', icon: ShieldCheck, label: 'Actions' },
  { to: '/memory', icon: Brain, label: 'Memory' },
  { to: '/activity', icon: Activity, label: 'Activity' },
]

function AetherMark() {
  return (
    <span className="aether-mark" aria-hidden="true">
      <span />
      <span />
      <span />
    </span>
  )
}

function NavGroup({ label, items }: { label: string; items: NavItem[] }) {
  return (
    <div className="aether-nav-group">
      <p className="aether-nav-label">{label}</p>
      <div className="space-y-0.5">
        {items.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            end={item.to === '/'}
            className={({ isActive }) =>
              cn('aether-nav-item focus-ring', isActive && 'aether-nav-item--active')
            }
          >
            {({ isActive }) => (
              <>
                <span className="aether-nav-indicator" aria-hidden="true" />
                <item.icon
                  size={16}
                  strokeWidth={isActive ? 2 : 1.65}
                  aria-hidden="true"
                />
                <span>{item.label}</span>
              </>
            )}
          </NavLink>
        ))}
      </div>
    </div>
  )
}

export function Sidebar() {
  const openCommandPalette = useCommandStore((state) => state.open)

  return (
    <aside className="aether-sidebar" aria-label="Aether navigation">
      <div className="aether-brand">
        <AetherMark />
        <div>
          <p className="aether-brand-name">Aether</p>
          <p className="aether-brand-tagline">Personal workspace</p>
        </div>
      </div>

      <button className="aether-command-trigger focus-ring" onClick={openCommandPalette}>
        <Search size={14} strokeWidth={1.8} aria-hidden="true" />
        <span>Search Aether</span>
        <kbd>
          <Command size={10} aria-hidden="true" />K
        </kbd>
      </button>

      <nav className="aether-sidebar-nav">
        <NavGroup label="Workspace" items={workspaceItems} />
        <NavGroup label="Intelligence" items={intelligenceItems} />
      </nav>

      <div className="aether-sidebar-footer">
        <NavLink
          to="/settings"
          className={({ isActive }) =>
            cn('aether-nav-item focus-ring', isActive && 'aether-nav-item--active')
          }
        >
          <span className="aether-nav-indicator" aria-hidden="true" />
          <Settings size={16} strokeWidth={1.65} aria-hidden="true" />
          <span>Settings</span>
        </NavLink>
        <div className="aether-local-status">
          <div className="flex items-center gap-2">
            <StatusDot />
            <span>Local & private</span>
          </div>
          <span>0.3.2</span>
        </div>
      </div>
    </aside>
  )
}
