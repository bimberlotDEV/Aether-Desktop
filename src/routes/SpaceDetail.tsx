import { useState } from 'react'
import { useParams, useNavigate, NavLink, Routes, Route } from 'react-router-dom'
import {
  Star, Archive, Copy, Trash2, ChevronRight,
  FileText, CheckSquare, FolderOpen, Sparkles, Activity as ActivityIcon,
  LayoutDashboard,
} from 'lucide-react'
import { useSpaceDetail, useSpaces } from '@/hooks/useSpaces'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import { cn } from '@/lib/utils'
import type { Space, ModuleInstance } from '@/lib/db/types'
import { iconToEmoji } from '@/lib/iconToEmoji'
import { NotesView } from '@/routes/Notes'

const MODULE_ICONS: Record<string, React.ComponentType<{ size?: number; strokeWidth?: number }>> = {
  notes: FileText,
  tasks: CheckSquare,
  files: FolderOpen,
  ai: Sparkles,
  activity: ActivityIcon,
}

const MODULE_LABELS: Record<string, string> = {
  notes: 'Notes',
  tasks: 'Tasks',
  files: 'Files',
  ai: 'AI',
  activity: 'Activity',
}

export function SpaceDetailLayout() {
  const { spaceId } = useParams<{ spaceId: string }>()
  const navigate = useNavigate()
  const { data, loading, error } = useSpaceDetail(spaceId)
  const { toggleFavourite, archive, duplicate, remove } = useSpaces()
  const [showDelete, setShowDelete] = useState(false)

  if (loading) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <p className="text-sm" style={{ color: 'var(--color-text-tertiary)' }}>Loading…</p>
      </div>
    )
  }

  if (error || !data) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center px-8 py-12 text-center">
        <h2 className="text-lg font-semibold mb-2" style={{ color: 'var(--color-text-primary)' }}>Space not found</h2>
        <p className="text-sm mb-6" style={{ color: 'var(--color-text-secondary)' }}>This Space may have been deleted or the link is invalid.</p>
        <button onClick={() => navigate('/spaces')} className="px-4 py-2 rounded-lg text-sm font-medium"
          style={{ backgroundColor: 'var(--color-accent)', color: '#fff' }}>
          Back to Spaces
        </button>
      </div>
    )
  }

  const { space, modules, children } = data
  const isArchived = !!space.archived_at

  return (
    <div className="flex-1 flex flex-col min-h-0">
      {/* Header */}
      <div className="px-8 py-5 shrink-0 flex items-center gap-4" style={{ borderBottom: '1px solid var(--color-border)' }}>
        {/* Breadcrumb */}
        <div className="flex items-center gap-1.5 text-sm">
          <button onClick={() => navigate('/spaces')} className="hover:underline" style={{ color: 'var(--color-text-tertiary)' }}>
            Spaces
          </button>
          <ChevronRight size={14} style={{ color: 'var(--color-text-tertiary)' }} />
          <span className="font-medium" style={{ color: 'var(--color-text-primary)' }}>{space.name}</span>
        </div>

        <div className="flex-1" />

        {/* Actions */}
        <div className="flex items-center gap-1">
          <button
            onClick={() => toggleFavourite(space.id, !space.favourite)}
            className="p-2 rounded-md transition-colors hover:bg-[var(--color-bg-tertiary)]"
            title={space.favourite ? 'Unfavourite' : 'Favourite'}
          >
            <Star size={16} fill={space.favourite ? 'var(--color-warning)' : 'none'}
              style={{ color: space.favourite ? 'var(--color-warning)' : 'var(--color-text-tertiary)' }} />
          </button>
          <button onClick={() => duplicate(space.id)}
            className="p-2 rounded-md transition-colors hover:bg-[var(--color-bg-tertiary)]" title="Duplicate"
            style={{ color: 'var(--color-text-tertiary)' }}>
            <Copy size={16} />
          </button>
          {isArchived ? (
            <button onClick={() => archive(space.id)}
              className="p-2 rounded-md transition-colors hover:bg-[var(--color-bg-tertiary)]" title="Restore"
              style={{ color: 'var(--color-text-tertiary)' }}>
              <Archive size={16} />
            </button>
          ) : (
            <button onClick={() => archive(space.id)}
              className="p-2 rounded-md transition-colors hover:bg-[var(--color-bg-tertiary)]" title="Archive"
              style={{ color: 'var(--color-text-tertiary)' }}>
              <Archive size={16} />
            </button>
          )}
          <button onClick={() => setShowDelete(true)}
            className="p-2 rounded-md transition-colors hover:bg-[var(--color-bg-tertiary)]" title="Delete"
            style={{ color: 'var(--color-danger)' }}>
            <Trash2 size={16} />
          </button>
        </div>
      </div>

      {/* Module tabs */}
      <div className="flex items-center gap-0 px-8 shrink-0" style={{ borderBottom: '1px solid var(--color-border)' }}>
        <NavLink
          to={`/spaces/${space.id}`} end
          className={({ isActive }) => cn(
            'flex items-center gap-2 px-4 py-2.5 text-sm font-medium border-b-2 transition-colors -mb-px',
            isActive ? 'border-[var(--color-accent)] text-[var(--color-accent)]' : 'border-transparent text-[var(--color-text-tertiary)] hover:text-[var(--color-text-secondary)]'
          )}
        >
          <LayoutDashboard size={15} /> Overview
        </NavLink>
        {modules.map(m => {
          const ModuleIcon = MODULE_ICONS[m.module_type] || FileText
          const label = MODULE_LABELS[m.module_type] || m.module_type
          return (
            <NavLink
              key={m.id}
              to={`/spaces/${space.id}/${m.module_type}`}
              className={({ isActive }) => cn(
                'flex items-center gap-2 px-4 py-2.5 text-sm font-medium border-b-2 transition-colors -mb-px',
                isActive ? 'border-[var(--color-accent)] text-[var(--color-accent)]' : 'border-transparent text-[var(--color-text-tertiary)] hover:text-[var(--color-text-secondary)]'
              )}
            >
              <ModuleIcon size={15} /> {label}
            </NavLink>
          )
        })}
      </div>

      {/* Content area */}
      <div className="flex-1 min-h-0 overflow-y-auto">
        <Routes>
          <Route index element={<OverviewTab space={space} modules={modules}>{children}</OverviewTab>} />
          {modules.map(m => (
            <Route
              key={m.id}
              path={m.module_type}
              element={
                m.module_type === 'notes' ? (
                  <NotesView />
                ) : (
                  <ModulePlaceholder moduleType={m.module_type} />
                )
              }
            />
          ))}
        </Routes>
      </div>

      {/* Delete confirm */}
      {showDelete && (
        <ConfirmDialog
          title="Delete Space"
          message={`Permanently delete "${space.name}"? This cannot be undone.${children.length > 0 ? ` ${children.length} child Space(s) will also be deleted.` : ''}`}
          confirmLabel={children.length > 0 ? `Delete Space and ${children.length} children` : 'Delete'}
          danger
          onConfirm={async () => { await remove(space.id); setShowDelete(false); navigate('/spaces') }}
          onCancel={() => setShowDelete(false)}
        />
      )}
    </div>
  )
}

function OverviewTab({ space, modules, children }: { space: Space; modules: ModuleInstance[]; children: Space[] }) {
  return (
    <div className="px-8 py-6 space-y-8 max-w-[700px]">
      {/* Space info */}
      <div className="flex items-start gap-4">
        <div className="w-14 h-14 rounded-xl flex items-center justify-center text-2xl shrink-0"
          style={{ backgroundColor: space.accent ? `${space.accent}18` : 'var(--color-accent-muted)' }}>
          {space.icon ? iconToEmoji(space.icon) : '📚'}
        </div>
        <div>
          <h1 className="text-xl font-semibold" style={{ color: 'var(--color-text-primary)' }}>{space.name}</h1>
          {space.description && (
            <p className="text-sm mt-1 leading-relaxed" style={{ color: 'var(--color-text-secondary)' }}>{space.description}</p>
          )}
        </div>
      </div>

      {/* Child Spaces (School subjects) */}
      {children.length > 0 && (
        <section>
          <h2 className="text-xs font-semibold uppercase tracking-wider mb-3" style={{ color: 'var(--color-text-tertiary)' }}>
            {space.template_type === 'school' ? 'Subjects' : 'Child Spaces'} ({children.length})
          </h2>
          <div className="space-y-1">
            {children.map(child => (
              <ChildSpaceRow key={child.id} space={child} />
            ))}
          </div>
        </section>
      )}

      {/* Modules */}
      <section>
        <h2 className="text-xs font-semibold uppercase tracking-wider mb-3" style={{ color: 'var(--color-text-tertiary)' }}>
          Modules ({modules.length})
        </h2>
        <div className="space-y-1">
          {modules.map(m => {
            const ModuleIcon = MODULE_ICONS[m.module_type] || FileText
            return (
              <div key={m.id} className="flex items-center gap-3 px-3 py-2.5 rounded-lg"
                style={{ backgroundColor: 'var(--color-bg-secondary)' }}>
                <span style={{ color: 'var(--color-accent)', display: 'flex' }}><ModuleIcon size={16} strokeWidth={1.75} /></span>
                <span className="text-sm font-medium" style={{ color: 'var(--color-text-primary)' }}>
                  {MODULE_LABELS[m.module_type] || m.module_type}
                </span>
              </div>
            )
          })}
        </div>
      </section>

      {/* Empty guidance */}
      {modules.length === 0 && children.length === 0 && (
        <div className="text-center py-10">
          <p className="text-sm" style={{ color: 'var(--color-text-tertiary)' }}>
            This Space is empty. Edit it to add modules or child Spaces.
          </p>
        </div>
      )}
    </div>
  )
}

function ChildSpaceRow({ space }: { space: Space }) {
  const navigate = useNavigate()
  return (
    <button
      onClick={() => navigate(`/spaces/${space.id}`)}
      className="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-left transition-colors hover:bg-[var(--color-bg-tertiary)]"
    >
      <span className="text-lg">{space.icon ? iconToEmoji(space.icon) : '📦'}</span>
      <span className="text-sm font-medium flex-1" style={{ color: 'var(--color-text-primary)' }}>{space.name}</span>
      <ChevronRight size={14} style={{ color: 'var(--color-text-tertiary)' }} />
    </button>
  )
}

function ModulePlaceholder({ moduleType }: { moduleType: string }) {
  const label = MODULE_LABELS[moduleType] || moduleType
  const ModuleIcon2 = MODULE_ICONS[moduleType] || FileText

  return (
    <div className="flex flex-col items-center justify-center py-20 text-center px-8">
      <div className="w-12 h-12 rounded-xl flex items-center justify-center mb-5"
        style={{ backgroundColor: 'var(--color-accent-muted)' }}>
        <span style={{ color: 'var(--color-accent)', display: 'flex' }}><ModuleIcon2 size={22} strokeWidth={1.75} /></span>
      </div>
      <h2 className="text-lg font-semibold mb-2" style={{ color: 'var(--color-text-primary)' }}>
        {label}
      </h2>
      <p className="text-sm max-w-[360px] leading-relaxed" style={{ color: 'var(--color-text-secondary)' }}>
        {label} will live here once the {label.toLowerCase()} module is fully implemented.
      </p>
    </div>
  )
}
