import { useState } from 'react'
import { Check, X, X as XIcon } from 'lucide-react'
import { IconPicker } from '@/components/IconPicker'
import { SPACE_ACCENTS, SPACE_MODULES } from '@/components/spaceOptions'
import { useSpaces } from '@/hooks/useSpaces'
import { cn } from '@/lib/utils'
import type { ModuleInstance, Space } from '@/lib/db/types'

interface Props {
  space: Space
  modules: ModuleInstance[]
  onClose: () => void
}

export function EditSpaceModal({ space, modules, onClose }: Props) {
  const { update } = useSpaces()
  const [name, setName] = useState(space.name)
  const [description, setDescription] = useState(space.description ?? '')
  const [icon, setIcon] = useState<string | null>(space.icon)
  const [accent, setAccent] = useState<string | null>(space.accent)
  const [selectedModules, setSelectedModules] = useState(() =>
    modules.map((module) => module.module_type),
  )
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const toggleModule = (moduleType: string) => {
    setSelectedModules((current) =>
      current.includes(moduleType)
        ? current.filter((item) => item !== moduleType)
        : [...current, moduleType],
    )
  }

  const handleSubmit = async () => {
    if (!name.trim() || selectedModules.length === 0) return
    setSubmitting(true)
    setError(null)
    try {
      await update(space.id, {
        name: name.trim(),
        description: description.trim() || null,
        icon,
        accent,
        moduleTypes: selectedModules,
      })
      onClose()
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : 'Failed to update Space')
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      onClick={onClose}
    >
      <div
        className="absolute inset-0"
        style={{ backgroundColor: 'var(--color-bg-overlay)' }}
      />
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby="edit-space-title"
        className="relative w-full max-w-[520px] rounded-xl shadow-2xl overflow-hidden"
        style={{
          backgroundColor: 'var(--color-bg-elevated)',
          border: '1px solid var(--color-border)',
        }}
        onClick={(event) => event.stopPropagation()}
      >
        <div
          className="flex items-center justify-between px-5 py-4"
          style={{ borderBottom: '1px solid var(--color-border)' }}
        >
          <h2
            id="edit-space-title"
            className="text-base font-semibold"
            style={{ color: 'var(--color-text-primary)' }}
          >
            Edit Space
          </h2>
          <button
            onClick={onClose}
            aria-label="Close"
            className="p-1 rounded-md hover:bg-[var(--color-bg-tertiary)]"
            style={{ color: 'var(--color-text-tertiary)' }}
          >
            <X size={18} />
          </button>
        </div>

        <div className="px-5 py-4 max-h-[65vh] overflow-y-auto space-y-5">
          <div>
            <label
              htmlFor="edit-space-name"
              className="text-xs font-medium mb-1.5 block"
              style={{ color: 'var(--color-text-secondary)' }}
            >
              Name *
            </label>
            <input
              id="edit-space-name"
              value={name}
              onChange={(event) => setName(event.target.value)}
              maxLength={80}
              className="w-full px-3 py-2 rounded-lg text-sm outline-none border"
              style={{
                backgroundColor: 'var(--color-bg-secondary)',
                borderColor: 'var(--color-border)',
                color: 'var(--color-text-primary)',
              }}
            />
          </div>
          <div>
            <label
              htmlFor="edit-space-description"
              className="text-xs font-medium mb-1.5 block"
              style={{ color: 'var(--color-text-secondary)' }}
            >
              Description
            </label>
            <textarea
              id="edit-space-description"
              value={description}
              onChange={(event) => setDescription(event.target.value)}
              maxLength={200}
              rows={2}
              className="w-full px-3 py-2 rounded-lg text-sm outline-none border resize-none"
              style={{
                backgroundColor: 'var(--color-bg-secondary)',
                borderColor: 'var(--color-border)',
                color: 'var(--color-text-primary)',
              }}
            />
          </div>
          <div>
            <span
              className="text-xs font-medium mb-1.5 block"
              style={{ color: 'var(--color-text-secondary)' }}
            >
              Icon
            </span>
            <IconPicker value={icon} onChange={setIcon} />
          </div>
          <div>
            <span
              className="text-xs font-medium mb-1.5 block"
              style={{ color: 'var(--color-text-secondary)' }}
            >
              Accent
            </span>
            <div className="flex gap-2">
              {SPACE_ACCENTS.map((option) => (
                <button
                  key={option.id}
                  onClick={() => setAccent(option.value)}
                  title={option.label}
                  aria-label={`${option.label} accent`}
                  className={cn(
                    'w-8 h-8 rounded-full transition-transform',
                    accent === option.value && 'scale-110 ring-2 ring-offset-2',
                  )}
                  style={{ backgroundColor: option.value }}
                />
              ))}
              {accent && (
                <button
                  onClick={() => setAccent(null)}
                  aria-label="Remove accent"
                  className="w-8 h-8 rounded-full flex items-center justify-center"
                  style={{ border: '1px solid var(--color-border)' }}
                >
                  <XIcon size={12} style={{ color: 'var(--color-text-tertiary)' }} />
                </button>
              )}
            </div>
          </div>
          <fieldset>
            <legend
              className="text-xs font-medium mb-1.5"
              style={{ color: 'var(--color-text-secondary)' }}
            >
              Modules
            </legend>
            <div className="space-y-2">
              {SPACE_MODULES.map((module) => {
                const active = selectedModules.includes(module.id)
                return (
                  <button
                    key={module.id}
                    type="button"
                    aria-pressed={active}
                    onClick={() => toggleModule(module.id)}
                    className="w-full flex items-center justify-between gap-3 p-3 rounded-lg border text-left"
                    style={{
                      borderColor: active ? 'var(--color-accent)' : 'var(--color-border)',
                      backgroundColor: active
                        ? 'var(--color-accent-muted)'
                        : 'transparent',
                    }}
                  >
                    <span>
                      <span
                        className="text-sm font-medium block"
                        style={{ color: 'var(--color-text-primary)' }}
                      >
                        {module.label}
                      </span>
                      <span
                        className="text-xs"
                        style={{ color: 'var(--color-text-tertiary)' }}
                      >
                        {module.description}
                      </span>
                    </span>
                    <span
                      className={cn(
                        'w-4 h-4 rounded border flex items-center justify-center',
                        active
                          ? 'bg-[var(--color-accent)] border-[var(--color-accent)]'
                          : 'border-[var(--color-border)]',
                      )}
                    >
                      {active && <Check size={11} style={{ color: '#fff' }} />}
                    </span>
                  </button>
                )
              })}
            </div>
          </fieldset>
          {error && (
            <p role="alert" className="text-xs" style={{ color: 'var(--color-danger)' }}>
              {error}
            </p>
          )}
        </div>

        <div
          className="flex justify-end gap-2 px-5 py-4"
          style={{ borderTop: '1px solid var(--color-border)' }}
        >
          <button
            onClick={onClose}
            className="px-4 py-2 rounded-lg text-sm font-medium"
            style={{ color: 'var(--color-text-secondary)' }}
          >
            Cancel
          </button>
          <button
            onClick={handleSubmit}
            disabled={!name.trim() || selectedModules.length === 0 || submitting}
            className="px-4 py-2 rounded-lg text-sm font-medium disabled:opacity-30"
            style={{
              backgroundColor: 'var(--color-accent)',
              color: 'var(--color-accent-text)',
            }}
          >
            {submitting ? 'Saving…' : 'Save changes'}
          </button>
        </div>
      </div>
    </div>
  )
}
