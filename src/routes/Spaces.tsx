import { useEffect, useState } from 'react'
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
import { iconToEmoji } from '@/lib/iconToEmoji'
import {
  Button,
  EmptyState,
  Page,
  PageHeader,
  SectionLabel,
} from '@/components/ui/AetherUI'

function SpaceIcon({ icon, accent }: { icon: string | null; accent: string | null }) {
  const emoji = icon ? iconToEmoji(icon) : '📚'
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

  useEffect(() => {
    if (!menuOpen) return
    const closeMenu = (event: KeyboardEvent) => {
      if (event.key === 'Escape') setMenuOpen(null)
    }
    window.addEventListener('keydown', closeMenu)
    return () => window.removeEventListener('keydown', closeMenu)
  }, [menuOpen])

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
    <Page width="default">
      <PageHeader
        eyebrow="Structure"
        icon={Layers}
        title="Spaces"
        description={
          hasSpaces
            ? `${spaces.filter((space) => !space.archived_at).length} active contexts for focused work.`
            : 'Create focused contexts for projects, study, and everything you want to keep together.'
        }
        actions={
          <Button variant="primary" icon={Plus} onClick={() => setShowCreate(true)}>
            New Space
          </Button>
        }
      />

      <div className="space-y-8">
        {!hasSpaces ? (
          <EmptyState
            icon={Layers}
            eyebrow="Your workspace, composed"
            title="Create a place for what matters"
            description="A Space keeps its Notes, Tasks, files, Memory, and AI context together without mixing it into the rest of your workspace."
            action={{
              label: 'Create your first Space',
              onClick: () => setShowCreate(true),
            }}
          />
        ) : (
          <>
            {favourites.length > 0 && (
              <section>
                <SectionLabel meta={`${favourites.length}`}>Favourites</SectionLabel>
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
              <SectionLabel meta={`${active.length}`}>All Spaces</SectionLabel>
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
    </Page>
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
        'aether-surface aether-surface--interactive group flex items-center rounded-xl transition-colors duration-100',
        isArchived && 'opacity-50',
      )}
      style={{ backgroundColor: isMenuOpen ? 'var(--color-bg-tertiary)' : undefined }}
    >
      <button
        type="button"
        onClick={onOpen}
        className="flex min-w-0 flex-1 items-center gap-3 px-3 py-2.5 text-left focus-ring"
        aria-label={`Open ${space.name}`}
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
      </button>
      <div className="relative pr-2">
        <button
          type="button"
          onClick={() => setMenuOpen(isMenuOpen ? null : space.id)}
          aria-label={`Actions for ${space.name}`}
          aria-expanded={isMenuOpen}
          className="p-1 rounded-md opacity-0 group-hover:opacity-100 focus:opacity-100 transition-opacity hover:bg-[var(--color-bg-tertiary)] focus-ring"
          style={{ color: 'var(--color-text-tertiary)' }}
        >
          <MoreHorizontal size={15} />
        </button>
        {isMenuOpen && (
          <>
            <div className="fixed inset-0 z-10" onClick={() => setMenuOpen(null)} />
            <div
              role="menu"
              aria-label={`Actions for ${space.name}`}
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
      type="button"
      role="menuitem"
      onClick={onClick}
      className="flex items-center gap-2.5 w-full px-3 py-2 text-sm text-left transition-colors hover:bg-[var(--color-bg-tertiary)]"
      style={{ color: danger ? 'var(--color-danger)' : 'var(--color-text-primary)' }}
    >
      <Icon size={14} /> {label}
    </button>
  )
}
