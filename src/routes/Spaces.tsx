import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import {
  Layers,
  Plus,
  Star,
  Archive,
  MoreHorizontal,
  Copy,
  Trash2,
  ExternalLink,
  ArrowUp,
  ArrowDown,
  RotateCcw,
} from 'lucide-react'
import { useSpaces, useArchivedSpaces } from '@/hooks/useSpaces'
import { CreateSpaceModal } from '@/components/CreateSpaceModal'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import type { Space } from '@/lib/db/types'
import { cn } from '@/lib/utils'

function SpaceIcon({ icon, accent }: { icon: string | null; accent: string | null }) {
  const emoji = icon || '📚'
  return (
    <div
      className="w-8 h-8 rounded-lg flex items-center justify-center shrink-0 text-lg"
      style={{ backgroundColor: accent ? `${accent}18` : 'var(--color-accent-muted)' }}
    >
      {emoji}
    </div>
  )
}

export function Spaces() {
  const navigate = useNavigate()
  const {
    spaces,
    loading,
    toggleFavourite,
    archive,
    restore,
    duplicate,
    remove,
    reorder,
  } = useSpaces()
  const { spaces: archived } = useArchivedSpaces()
  const [showCreate, setShowCreate] = useState(false)
  const [menuOpen, setMenuOpen] = useState<string | null>(null)
  const [showArchived, setShowArchived] = useState(false)
  const [deleteTarget, setDeleteTarget] = useState<Space | null>(null)

  const favourites = spaces.filter((s) => s.favourite && !s.archived_at)
  const active = spaces.filter((s) => !s.favourite && !s.archived_at)
  const hasSpaces = spaces.length > 0

  const moveWithinGroup = async (id: string, group: Space[], offset: -1 | 1) => {
    const groupIndex = group.findIndex((space) => space.id === id)
    const target = group[groupIndex + offset]
    if (!target) return

    const orderedIds = spaces
      .filter((space) => !space.archived_at)
      .map((space) => space.id)
    const sourceIndex = orderedIds.indexOf(id)
    const targetIndex = orderedIds.indexOf(target.id)
    ;[orderedIds[sourceIndex], orderedIds[targetIndex]] = [
      orderedIds[targetIndex],
      orderedIds[sourceIndex],
    ]
    await reorder(orderedIds)
  }

  if (loading) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <p className="text-sm" style={{ color: 'var(--color-text-tertiary)' }}>
          Loading spaces…
        </p>
      </div>
    )
  }

  return (
    <div className="flex-1 flex flex-col min-h-0 overflow-y-auto">
      <div
        className="px-8 py-6 shrink-0 flex items-center justify-between"
        style={{ borderBottom: '1px solid var(--color-border)' }}
      >
        <div>
          <h1
            className="text-xl font-semibold tracking-tight"
            style={{ color: 'var(--color-text-primary)' }}
          >
            Spaces
          </h1>
          <p className="text-sm mt-1" style={{ color: 'var(--color-text-tertiary)' }}>
            {hasSpaces
              ? `${spaces.filter((s) => !s.archived_at).length} active spaces`
              : 'Create your first Space'}
          </p>
        </div>
        <button
          onClick={() => setShowCreate(true)}
          className="inline-flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-colors duration-100"
          style={{
            backgroundColor: 'var(--color-accent)',
            color: 'var(--color-accent-text)',
          }}
        >
          <Plus size={15} strokeWidth={2} />
          New Space
        </button>
      </div>

      <div className="flex-1 px-8 py-6 space-y-10 max-w-[800px]">
        {!hasSpaces ? (
          <div className="flex flex-col items-center justify-center py-20 text-center">
            <div
              className="w-14 h-14 rounded-2xl flex items-center justify-center mb-6"
              style={{ backgroundColor: 'var(--color-accent-muted)' }}
            >
              <Layers
                size={26}
                strokeWidth={1.5}
                style={{ color: 'var(--color-accent)' }}
              />
            </div>
            <h2
              className="text-lg font-semibold mb-2"
              style={{ color: 'var(--color-text-primary)' }}
            >
              No Spaces yet
            </h2>
            <p
              className="text-sm max-w-[320px] mb-6 leading-relaxed"
              style={{ color: 'var(--color-text-secondary)' }}
            >
              Spaces help you organize everything into dedicated contexts. Create a Space
              for school, work, or any project.
            </p>
            <button
              onClick={() => setShowCreate(true)}
              className="inline-flex items-center gap-2 px-5 py-2.5 rounded-lg text-sm font-medium transition-colors"
              style={{
                backgroundColor: 'var(--color-accent)',
                color: 'var(--color-accent-text)',
              }}
            >
              <Plus size={15} strokeWidth={2} /> Create your first Space
            </button>
          </div>
        ) : (
          <>
            {favourites.length > 0 && (
              <section>
                <h2
                  className="text-xs font-semibold uppercase tracking-wider mb-3"
                  style={{ color: 'var(--color-text-tertiary)' }}
                >
                  Favourites
                </h2>
                <div className="space-y-1">
                  {favourites.map((s) => (
                    <SpaceRow
                      key={s.id}
                      space={s}
                      onOpen={() => navigate(`/spaces/${s.id}`)}
                      onFavourite={() => toggleFavourite(s.id, false)}
                      onArchive={() => archive(s.id)}
                      onDuplicate={() => duplicate(s.id)}
                      onDelete={() => setDeleteTarget(s)}
                      onMoveUp={() => moveWithinGroup(s.id, favourites, -1)}
                      onMoveDown={() => moveWithinGroup(s.id, favourites, 1)}
                      canMoveUp={favourites.indexOf(s) > 0}
                      canMoveDown={favourites.indexOf(s) < favourites.length - 1}
                      menuOpen={menuOpen}
                      setMenuOpen={setMenuOpen}
                    />
                  ))}
                </div>
              </section>
            )}
            <section>
              <h2
                className="text-xs font-semibold uppercase tracking-wider mb-3"
                style={{ color: 'var(--color-text-tertiary)' }}
              >
                All Spaces
              </h2>
              <div className="space-y-1">
                {active.map((s) => (
                  <SpaceRow
                    key={s.id}
                    space={s}
                    onOpen={() => navigate(`/spaces/${s.id}`)}
                    onFavourite={() => toggleFavourite(s.id, true)}
                    onArchive={() => archive(s.id)}
                    onDuplicate={() => duplicate(s.id)}
                    onDelete={() => setDeleteTarget(s)}
                    onMoveUp={() => moveWithinGroup(s.id, active, -1)}
                    onMoveDown={() => moveWithinGroup(s.id, active, 1)}
                    canMoveUp={active.indexOf(s) > 0}
                    canMoveDown={active.indexOf(s) < active.length - 1}
                    menuOpen={menuOpen}
                    setMenuOpen={setMenuOpen}
                  />
                ))}
              </div>
            </section>
            {archived.length > 0 && (
              <section>
                <button
                  onClick={() => setShowArchived(!showArchived)}
                  className="flex items-center gap-2 text-xs font-semibold uppercase tracking-wider mb-3"
                  style={{ color: 'var(--color-text-tertiary)' }}
                >
                  <Archive size={12} /> Archived ({archived.length})
                </button>
                {showArchived && (
                  <div className="space-y-1">
                    {archived.map((s) => (
                      <SpaceRow
                        key={s.id}
                        space={s}
                        isArchived
                        onOpen={() => navigate(`/spaces/${s.id}`)}
                        onFavourite={() => {}}
                        onArchive={() => {}}
                        onRestore={() => restore(s.id)}
                        onDuplicate={() => duplicate(s.id)}
                        onDelete={() => setDeleteTarget(s)}
                        menuOpen={menuOpen}
                        setMenuOpen={setMenuOpen}
                      />
                    ))}
                  </div>
                )}
              </section>
            )}
          </>
        )}
      </div>

      {showCreate && <CreateSpaceModal onClose={() => setShowCreate(false)} />}
      {deleteTarget && (
        <ConfirmDialog
          title="Delete Space"
          message={`Permanently delete "${deleteTarget.name}"? This cannot be undone.`}
          confirmLabel="Delete"
          danger
          onConfirm={async () => {
            await remove(deleteTarget.id)
            setDeleteTarget(null)
          }}
          onCancel={() => setDeleteTarget(null)}
        />
      )}
    </div>
  )
}

function SpaceRow({
  space,
  isArchived,
  onOpen,
  onFavourite,
  onArchive,
  onRestore,
  onDuplicate,
  onDelete,
  onMoveUp,
  onMoveDown,
  canMoveUp,
  canMoveDown,
  menuOpen,
  setMenuOpen,
}: {
  space: Space
  isArchived?: boolean
  onOpen: () => void
  onFavourite: () => void
  onArchive: () => void
  onRestore?: () => void
  onDuplicate: () => void
  onDelete: () => void
  onMoveUp?: () => void
  onMoveDown?: () => void
  canMoveUp?: boolean
  canMoveDown?: boolean
  menuOpen: string | null
  setMenuOpen: (id: string | null) => void
}) {
  const isMenuOpen = menuOpen === space.id
  return (
    <div
      className={cn(
        'group flex items-center gap-3 px-3 py-2.5 rounded-lg transition-colors duration-100 cursor-pointer',
        isArchived && 'opacity-50',
      )}
      style={{ backgroundColor: isMenuOpen ? 'var(--color-bg-tertiary)' : undefined }}
      onClick={onOpen}
    >
      <SpaceIcon icon={space.icon} accent={space.accent} />
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span
            className="text-sm font-medium truncate"
            style={{ color: 'var(--color-text-primary)' }}
          >
            {space.name}
          </span>
          {space.favourite && (
            <Star
              size={11}
              fill="var(--color-warning)"
              style={{ color: 'var(--color-warning)' }}
            />
          )}
        </div>
        {space.description && (
          <p
            className="text-xs truncate mt-0.5"
            style={{ color: 'var(--color-text-tertiary)' }}
          >
            {space.description}
          </p>
        )}
      </div>
      <div className="relative" onClick={(e) => e.stopPropagation()}>
        <button
          onClick={() => setMenuOpen(isMenuOpen ? null : space.id)}
          className="p-1 rounded-md opacity-0 group-hover:opacity-100 transition-opacity hover:bg-[var(--color-bg-tertiary)]"
          style={{ color: 'var(--color-text-tertiary)' }}
        >
          <MoreHorizontal size={15} />
        </button>
        {isMenuOpen && (
          <>
            <div className="fixed inset-0 z-10" onClick={() => setMenuOpen(null)} />
            <div
              className="absolute right-0 top-8 z-20 w-44 py-1 rounded-lg shadow-lg border"
              style={{
                backgroundColor: 'var(--color-bg-elevated)',
                borderColor: 'var(--color-border)',
              }}
            >
              <MenuItem
                icon={ExternalLink}
                label="Open"
                onClick={() => {
                  setMenuOpen(null)
                  onOpen()
                }}
              />
              {isArchived ? (
                <MenuItem
                  icon={RotateCcw}
                  label="Restore"
                  onClick={() => {
                    setMenuOpen(null)
                    onRestore?.()
                  }}
                />
              ) : (
                <>
                  <MenuItem
                    icon={Star}
                    label={space.favourite ? 'Unfavourite' : 'Favourite'}
                    onClick={() => {
                      setMenuOpen(null)
                      onFavourite()
                    }}
                  />
                  {canMoveUp && (
                    <MenuItem
                      icon={ArrowUp}
                      label="Move up"
                      onClick={() => {
                        setMenuOpen(null)
                        onMoveUp?.()
                      }}
                    />
                  )}
                  {canMoveDown && (
                    <MenuItem
                      icon={ArrowDown}
                      label="Move down"
                      onClick={() => {
                        setMenuOpen(null)
                        onMoveDown?.()
                      }}
                    />
                  )}
                  <MenuItem
                    icon={Archive}
                    label="Archive"
                    onClick={() => {
                      setMenuOpen(null)
                      onArchive()
                    }}
                  />
                </>
              )}
              <MenuItem
                icon={Copy}
                label="Duplicate"
                onClick={() => {
                  setMenuOpen(null)
                  onDuplicate()
                }}
              />
              <div style={{ borderTop: '1px solid var(--color-border)' }} />
              <MenuItem
                icon={Trash2}
                label="Delete"
                danger
                onClick={() => {
                  setMenuOpen(null)
                  onDelete()
                }}
              />
            </div>
          </>
        )}
      </div>
    </div>
  )
}

function MenuItem({
  icon: Icon,
  label,
  danger,
  onClick,
}: {
  icon: React.ComponentType<{ size?: number }>
  label: string
  danger?: boolean
  onClick: () => void
}) {
  return (
    <button
      onClick={onClick}
      className="flex items-center gap-2.5 w-full px-3 py-2 text-sm text-left transition-colors hover:bg-[var(--color-bg-tertiary)]"
      style={{ color: danger ? 'var(--color-danger)' : 'var(--color-text-primary)' }}
    >
      <Icon size={14} /> {label}
    </button>
  )
}
