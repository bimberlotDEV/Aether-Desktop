import { useState } from 'react'
import { Search } from 'lucide-react'
import { cn } from '@/lib/utils'
import { iconToEmoji } from '@/lib/iconToEmoji'

// Curated icon set for Spaces
const ICONS: Record<string, { icon: string; label: string }[]> = {
  General: [
    { icon: 'Layers', label: 'Stack' },
    { icon: 'FolderOpen', label: 'Folder' },
    { icon: 'Star', label: 'Star' },
    { icon: 'Heart', label: 'Heart' },
    { icon: 'Zap', label: 'Lightning' },
    { icon: 'Compass', label: 'Compass' },
    { icon: 'Target', label: 'Target' },
    { icon: 'Award', label: 'Award' },
  ],
  School: [
    { icon: 'GraduationCap', label: 'Graduation' },
    { icon: 'BookOpen', label: 'Book' },
    { icon: 'PenTool', label: 'Pen' },
    { icon: 'Calculator', label: 'Calculator' },
    { icon: 'FlaskConical', label: 'Science' },
    { icon: 'Languages', label: 'Languages' },
    { icon: 'Microscope', label: 'Microscope' },
    { icon: 'Library', label: 'Library' },
  ],
  Work: [
    { icon: 'Briefcase', label: 'Briefcase' },
    { icon: 'Building2', label: 'Building' },
    { icon: 'Presentation', label: 'Presentation' },
    { icon: 'LineChart', label: 'Chart' },
    { icon: 'Calendar', label: 'Calendar' },
    { icon: 'Mail', label: 'Mail' },
    { icon: 'Users', label: 'People' },
    { icon: 'MessageSquare', label: 'Chat' },
  ],
  Personal: [
    { icon: 'Home', label: 'Home' },
    { icon: 'Dumbbell', label: 'Fitness' },
    { icon: 'Utensils', label: 'Food' },
    { icon: 'Plane', label: 'Travel' },
    { icon: 'Music', label: 'Music' },
    { icon: 'Camera', label: 'Camera' },
    { icon: 'Gamepad2', label: 'Gaming' },
    { icon: 'ShoppingBag', label: 'Shopping' },
  ],
  Creative: [
    { icon: 'Palette', label: 'Palette' },
    { icon: 'Pencil', label: 'Pencil' },
    { icon: 'Film', label: 'Film' },
    { icon: 'Code2', label: 'Code' },
    { icon: 'Globe', label: 'Globe' },
    { icon: 'Sparkles', label: 'Sparkles' },
    { icon: 'Gem', label: 'Gem' },
    { icon: 'Crown', label: 'Crown' },
  ],
}

interface Props {
  value: string | null
  onChange: (icon: string) => void
}

export function IconPicker({ value, onChange }: Props) {
  const [search, setSearch] = useState('')
  const [activeCategory, setActiveCategory] = useState('General')

  const allIcons = Object.values(ICONS).flat()
  const filtered = search
    ? allIcons.filter(i => i.label.toLowerCase().includes(search.toLowerCase()) || i.icon.toLowerCase().includes(search.toLowerCase()))
    : ICONS[activeCategory] || []

  return (
    <div>
      {/* Search */}
      <div className="flex items-center gap-2 px-3 py-2 rounded-lg mb-3" style={{ backgroundColor: 'var(--color-bg-tertiary)' }}>
        <Search size={14} style={{ color: 'var(--color-text-tertiary)' }} />
        <input
          value={search} onChange={e => setSearch(e.target.value)}
          placeholder="Search icons…" className="flex-1 bg-transparent outline-none text-sm"
          style={{ color: 'var(--color-text-primary)' }}
        />
      </div>

      {/* Categories (hide when searching) */}
      {!search && (
        <div className="flex gap-1 mb-3 flex-wrap">
          {Object.keys(ICONS).map(cat => (
            <button
              key={cat}
              onClick={() => setActiveCategory(cat)}
              className="px-2.5 py-1 rounded-md text-xs font-medium transition-colors"
              style={{
                backgroundColor: cat === activeCategory ? 'var(--color-accent-muted)' : 'transparent',
                color: cat === activeCategory ? 'var(--color-accent)' : 'var(--color-text-tertiary)',
              }}
            >
              {cat}
            </button>
          ))}
        </div>
      )}

      {/* Grid */}
      <div className="grid grid-cols-6 gap-2 max-h-[180px] overflow-y-auto">
        {filtered.map(({ icon, label }) => (
          <button
            key={icon}
            onClick={() => onChange(icon)}
            title={label}
            className={cn(
              'flex flex-col items-center gap-1 p-2 rounded-lg transition-colors',
              value === icon ? 'bg-[var(--color-accent-muted)]' : 'hover:bg-[var(--color-bg-tertiary)]'
            )}
          >
            <span className="text-lg">{iconToEmoji(icon)}</span>
            <span className="text-[9px] truncate max-w-full" style={{ color: value === icon ? 'var(--color-accent)' : 'var(--color-text-tertiary)' }}>
              {label}
            </span>
          </button>
        ))}
      </div>
    </div>
  )
}

export { ICONS }
