import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { Archive, RotateCcw, Trash2, ChevronLeft } from 'lucide-react'
import { useArchivedSpaces, useSpaces } from '@/hooks/useSpaces'
import { ConfirmDialog } from '@/components/ConfirmDialog'
import type { Space } from '@/lib/db/types'

export function ArchivedSpaces() {
  const navigate = useNavigate()
  const { spaces, loading } = useArchivedSpaces()
  const { restore, remove } = useSpaces()
  const [deleteTarget, setDeleteTarget] = useState<Space | null>(null)

  return (
    <div className="flex-1 flex flex-col min-h-0 overflow-y-auto">
      <div
        className="px-8 py-6 shrink-0 flex items-center gap-4"
        style={{ borderBottom: '1px solid var(--color-border)' }}
      >
        <button
          type="button"
          onClick={() => navigate('/spaces')}
          aria-label="Back to Spaces"
          title="Back to Spaces"
          className="p-1 rounded-md hover:bg-[var(--color-bg-tertiary)]"
          style={{ color: 'var(--color-text-tertiary)' }}
        >
          <ChevronLeft size={18} />
        </button>
        <div>
          <h1
            className="text-xl font-semibold tracking-tight"
            style={{ color: 'var(--color-text-primary)' }}
          >
            Archived Spaces
          </h1>
          <p className="text-sm mt-1" style={{ color: 'var(--color-text-tertiary)' }}>
            {spaces.length} archived space(s)
          </p>
        </div>
      </div>

      <div className="flex-1 px-8 py-6 max-w-[800px]">
        {loading ? (
          <p className="text-sm" style={{ color: 'var(--color-text-tertiary)' }}>
            Loading…
          </p>
        ) : spaces.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-20 text-center">
            <div
              className="w-12 h-12 rounded-xl flex items-center justify-center mb-5"
              style={{ backgroundColor: 'var(--color-bg-tertiary)' }}
            >
              <Archive
                size={22}
                strokeWidth={1.75}
                style={{ color: 'var(--color-text-tertiary)' }}
              />
            </div>
            <h2
              className="text-lg font-semibold mb-2"
              style={{ color: 'var(--color-text-primary)' }}
            >
              No archived Spaces
            </h2>
            <p className="text-sm" style={{ color: 'var(--color-text-secondary)' }}>
              Archived Spaces will appear here.
            </p>
          </div>
        ) : (
          <div className="space-y-1">
            {spaces.map((s) => (
              <div
                key={s.id}
                className="flex items-center gap-3 px-3 py-2.5 rounded-lg opacity-60 hover:opacity-100 transition-opacity"
                style={{ backgroundColor: 'var(--color-bg-secondary)' }}
              >
                <span className="text-lg">📦</span>
                <div className="flex-1 min-w-0">
                  <span
                    className="text-sm font-medium"
                    style={{ color: 'var(--color-text-primary)' }}
                  >
                    {s.name}
                  </span>
                  {s.archived_at && (
                    <span
                      className="text-xs ml-2"
                      style={{ color: 'var(--color-text-tertiary)' }}
                    >
                      Archived {new Date(s.archived_at).toLocaleDateString()}
                    </span>
                  )}
                </div>
                <button
                  onClick={() => restore(s.id)}
                  className="p-1.5 rounded-md hover:bg-[var(--color-bg-tertiary)]"
                  title="Restore"
                  style={{ color: 'var(--color-text-tertiary)' }}
                >
                  <RotateCcw size={15} />
                </button>
                <button
                  onClick={() => setDeleteTarget(s)}
                  className="p-1.5 rounded-md hover:bg-[var(--color-bg-tertiary)]"
                  title="Delete permanently"
                  style={{ color: 'var(--color-danger)' }}
                >
                  <Trash2 size={15} />
                </button>
              </div>
            ))}
          </div>
        )}
      </div>

      {deleteTarget && (
        <ConfirmDialog
          title="Delete permanently"
          message={`"${deleteTarget.name}" will be permanently deleted. This cannot be undone.`}
          confirmLabel="Delete permanently"
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
