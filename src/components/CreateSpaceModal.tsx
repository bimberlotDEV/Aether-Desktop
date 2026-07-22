import { useState, useEffect, useRef } from 'react'
import { X, ChevronLeft, ChevronRight, Check, X as XIcon } from 'lucide-react'
import { useSpaces } from '@/hooks/useSpaces'
import { IconPicker } from '@/components/IconPicker'
import { cn } from '@/lib/utils'

const ACCENTS = [
  { id: 'indigo', value: '#6366f1', label: 'Indigo' },
  { id: 'blue', value: '#3b82f6', label: 'Blue' },
  { id: 'emerald', value: '#10b981', label: 'Emerald' },
  { id: 'amber', value: '#f59e0b', label: 'Amber' },
  { id: 'rose', value: '#f43f5e', label: 'Rose' },
  { id: 'violet', value: '#8b5cf6', label: 'Violet' },
  { id: 'teal', value: '#14b8a6', label: 'Teal' },
  { id: 'slate', value: '#64748b', label: 'Slate' },
]

const MODULES = [
  { id: 'notes', label: 'Notes', description: 'Write, organize, and search notes' },
  { id: 'tasks', label: 'Tasks', description: 'Track to-dos with priorities and due dates' },
  { id: 'files', label: 'Files', description: 'Import and manage documents' },
  { id: 'ai', label: 'AI', description: 'AI-powered assistance and summarization' },
  { id: 'activity', label: 'Activity', description: 'Timeline of meaningful actions' },
]

const DEFAULT_SUBJECTS = ['Chemistry', 'Mathematics', 'Physics', 'English', 'Internship', 'Exams']

interface Props {
  onClose: () => void
  initialParentId?: string
}

export function CreateSpaceModal({ onClose, initialParentId }: Props) {
  const { create } = useSpaces()
  const [step, setStep] = useState(0)
  const [submitting, setSubmitting] = useState(false)

  // Step 1: Template
  const [template, setTemplate] = useState<'blank' | 'school'>('blank')

  // Step 2: Details
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [icon, setIcon] = useState<string | null>(null)
  const [accent, setAccent] = useState<string | null>(null)

  // Step 3: Modules
  const [selectedModules, setSelectedModules] = useState<string[]>(['notes', 'tasks', 'files', 'ai', 'activity'])

  // Step 4: School subjects
  const [subjects, setSubjects] = useState<string[]>([])
  const [newSubject, setNewSubject] = useState('')

  const nameRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (step === 1) nameRef.current?.focus()
  }, [step])

  const totalSteps = template === 'school' ? 4 : 3
  const canNext = step === 0
    ? true
    : step === 1
      ? name.trim().length > 0
      : step === 2
        ? selectedModules.length > 0
        : true

  const handleSubmit = async () => {
    setSubmitting(true)
    try {
      await create({
        name: name.trim(),
        description: description.trim() || undefined,
        icon: icon || undefined,
        accent: accent || undefined,
        templateType: template,
        moduleTypes: selectedModules,
        parentSpaceId: initialParentId,
        subjects: template === 'school' ? subjects.map(s => ({ name: s })) : undefined,
      })
      onClose()
    } catch (e) {
      console.error('Failed to create space:', e)
    } finally {
      setSubmitting(false)
    }
  }

  const toggleModule = (id: string) => {
    setSelectedModules(prev =>
      prev.includes(id) ? prev.filter(m => m !== id) : [...prev, id]
    )
  }

  const addSubject = () => {
    const s = newSubject.trim()
    if (s && !subjects.includes(s)) {
      setSubjects([...subjects, s])
      setNewSubject('')
    }
  }

  const removeSubject = (s: string) => {
    setSubjects(subjects.filter(x => x !== s))
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center" onClick={onClose}>
      <div className="absolute inset-0" style={{ backgroundColor: 'var(--color-bg-overlay)' }} />
      <div
        className="relative w-full max-w-[480px] rounded-xl shadow-2xl overflow-hidden"
        style={{ backgroundColor: 'var(--color-bg-elevated)', border: '1px solid var(--color-border)' }}
        onClick={e => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-4" style={{ borderBottom: '1px solid var(--color-border)' }}>
          <div className="flex items-center gap-3">
            {step > 0 && (
              <button onClick={() => setStep(step - 1)} className="p-1 rounded-md hover:bg-[var(--color-bg-tertiary)]" style={{ color: 'var(--color-text-secondary)' }}>
                <ChevronLeft size={18} />
              </button>
            )}
            <h2 className="text-base font-semibold" style={{ color: 'var(--color-text-primary)' }}>
              {step === 0 ? 'New Space' : step === 1 ? 'Details' : step === 2 ? 'Modules' : 'Subjects'}
            </h2>
          </div>
          <button onClick={onClose} className="p-1 rounded-md hover:bg-[var(--color-bg-tertiary)]" style={{ color: 'var(--color-text-tertiary)' }}>
            <X size={18} />
          </button>
        </div>

        {/* Steps indicator */}
        <div className="flex gap-1 px-5 py-2">
          {Array.from({ length: totalSteps }).map((_, i) => (
            <div
              key={i}
              className="h-1 flex-1 rounded-full transition-colors"
              style={{ backgroundColor: i <= step ? 'var(--color-accent)' : 'var(--color-border)' }}
            />
          ))}
        </div>

        {/* Content */}
        <div className="px-5 py-4 max-h-[50vh] overflow-y-auto">
          {/* Step 0: Template */}
          {step === 0 && (
            <div className="space-y-3">
              <button
                onClick={() => { setTemplate('blank'); setStep(1) }}
                className="w-full text-left p-4 rounded-lg border transition-colors hover:bg-[var(--color-bg-secondary)]"
                style={{ borderColor: 'var(--color-border)' }}
              >
                <h3 className="text-sm font-semibold mb-1" style={{ color: 'var(--color-text-primary)' }}>Blank Space</h3>
                <p className="text-xs leading-relaxed" style={{ color: 'var(--color-text-tertiary)' }}>
                  Start with an empty Space. Choose modules manually and build from scratch.
                </p>
              </button>
              <button
                onClick={() => { setTemplate('school'); setStep(1) }}
                className="w-full text-left p-4 rounded-lg border transition-colors hover:bg-[var(--color-bg-secondary)]"
                style={{ borderColor: 'var(--color-border)' }}
              >
                <h3 className="text-sm font-semibold mb-1" style={{ color: 'var(--color-text-primary)' }}>School Space</h3>
                <p className="text-xs leading-relaxed" style={{ color: 'var(--color-text-tertiary)' }}>
                  Create a School Space with optional subject child Spaces for each course.
                </p>
              </button>
            </div>
          )}

          {/* Step 1: Details */}
          {step === 1 && (
            <div className="space-y-4">
              <div>
                <label className="text-xs font-medium mb-1.5 block" style={{ color: 'var(--color-text-secondary)' }}>Name *</label>
                <input
                  ref={nameRef}
                  value={name}
                  onChange={e => setName(e.target.value)}
                  placeholder="Space name"
                  maxLength={80}
                  className="w-full px-3 py-2 rounded-lg text-sm outline-none border transition-colors"
                  style={{
                    backgroundColor: 'var(--color-bg-secondary)',
                    borderColor: 'var(--color-border)',
                    color: 'var(--color-text-primary)',
                  }}
                  onKeyDown={e => { if (e.key === 'Enter' && canNext) setStep(2) }}
                />
              </div>
              <div>
                <label className="text-xs font-medium mb-1.5 block" style={{ color: 'var(--color-text-secondary)' }}>Description</label>
                <textarea
                  value={description}
                  onChange={e => setDescription(e.target.value)}
                  placeholder="What is this Space for?"
                  maxLength={200}
                  rows={2}
                  className="w-full px-3 py-2 rounded-lg text-sm outline-none border resize-none transition-colors"
                  style={{
                    backgroundColor: 'var(--color-bg-secondary)',
                    borderColor: 'var(--color-border)',
                    color: 'var(--color-text-primary)',
                  }}
                />
              </div>
              <div>
                <label className="text-xs font-medium mb-1.5 block" style={{ color: 'var(--color-text-secondary)' }}>Icon</label>
                <IconPicker value={icon} onChange={setIcon} />
              </div>
              <div>
                <label className="text-xs font-medium mb-1.5 block" style={{ color: 'var(--color-text-secondary)' }}>Accent</label>
                <div className="flex gap-2">
                  {ACCENTS.map(a => (
                    <button
                      key={a.id}
                      onClick={() => setAccent(a.value)}
                      title={a.label}
                      className={cn(
                        'w-8 h-8 rounded-full transition-transform',
                        accent === a.value && 'scale-110 ring-2 ring-offset-2'
                      )}
                      style={{ backgroundColor: a.value }}
                    />
                  ))}
                  {accent && (
                    <button onClick={() => setAccent(null)} className="w-8 h-8 rounded-full flex items-center justify-center" style={{ border: '1px solid var(--color-border)' }}>
                      <XIcon size={12} style={{ color: 'var(--color-text-tertiary)' }} />
                    </button>
                  )}
                </div>
              </div>
            </div>
          )}

          {/* Step 2: Modules */}
          {step === 2 && (
            <div className="space-y-2">
              <p className="text-xs mb-3" style={{ color: 'var(--color-text-tertiary)' }}>Select the modules you want in this Space.</p>
              {MODULES.map(m => {
                const active = selectedModules.includes(m.id)
                return (
                  <button
                    key={m.id}
                    onClick={() => toggleModule(m.id)}
                    className="w-full text-left p-3 rounded-lg border transition-colors"
                    style={{
                      borderColor: active ? 'var(--color-accent)' : 'var(--color-border)',
                      backgroundColor: active ? 'var(--color-accent-muted)' : 'transparent',
                    }}
                  >
                    <div className="flex items-center justify-between">
                      <span className="text-sm font-medium" style={{ color: 'var(--color-text-primary)' }}>{m.label}</span>
                      <div
                        className={cn('w-4 h-4 rounded border flex items-center justify-center transition-colors',
                          active ? 'bg-[var(--color-accent)] border-[var(--color-accent)]' : 'border-[var(--color-border)]'
                        )}
                      >
                        {active && <Check size={11} style={{ color: '#fff' }} />}
                      </div>
                    </div>
                    <p className="text-xs mt-0.5" style={{ color: 'var(--color-text-tertiary)' }}>{m.description}</p>
                  </button>
                )
              })}
            </div>
          )}

          {/* Step 3: School subjects */}
          {step === 3 && (
            <div className="space-y-3">
              <p className="text-xs" style={{ color: 'var(--color-text-tertiary)' }}>Add subject Spaces for your courses. You can skip this and add subjects later.</p>

              {/* Suggested subjects */}
              <div className="flex flex-wrap gap-1.5">
                {DEFAULT_SUBJECTS.filter(s => !subjects.includes(s)).map(s => (
                  <button
                    key={s}
                    onClick={() => setSubjects([...subjects, s])}
                    className="px-2.5 py-1 rounded-md text-xs font-medium transition-colors hover:bg-[var(--color-bg-tertiary)]"
                    style={{ border: '1px solid var(--color-border)', color: 'var(--color-text-secondary)' }}
                  >
                    + {s}
                  </button>
                ))}
              </div>

              {/* Selected subjects */}
              {subjects.length > 0 && (
                <div className="space-y-1">
                  {subjects.map(s => (
                    <div key={s} className="flex items-center justify-between px-3 py-2 rounded-lg" style={{ backgroundColor: 'var(--color-bg-secondary)' }}>
                      <span className="text-sm" style={{ color: 'var(--color-text-primary)' }}>{s}</span>
                      <button onClick={() => removeSubject(s)} className="p-0.5 rounded hover:bg-[var(--color-bg-tertiary)]" style={{ color: 'var(--color-text-tertiary)' }}>
                        <XIcon size={14} />
                      </button>
                    </div>
                  ))}
                </div>
              )}

              {/* Custom subject input */}
              <div className="flex gap-2">
                <input
                  value={newSubject}
                  onChange={e => setNewSubject(e.target.value)}
                  onKeyDown={e => { if (e.key === 'Enter') addSubject() }}
                  placeholder="Add custom subject…"
                  className="flex-1 px-3 py-2 rounded-lg text-sm outline-none border"
                  style={{ backgroundColor: 'var(--color-bg-secondary)', borderColor: 'var(--color-border)', color: 'var(--color-text-primary)' }}
                />
                <button
                  onClick={addSubject}
                  disabled={!newSubject.trim()}
                  className="px-3 py-2 rounded-lg text-sm font-medium transition-colors disabled:opacity-30"
                  style={{ backgroundColor: 'var(--color-accent)', color: '#fff' }}
                >
                  Add
                </button>
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex justify-between px-5 py-4" style={{ borderTop: '1px solid var(--color-border)' }}>
          <button
            onClick={onClose}
            className="px-4 py-2 rounded-lg text-sm font-medium transition-colors"
            style={{ color: 'var(--color-text-secondary)' }}
          >
            Cancel
          </button>
          {step < totalSteps - 1 ? (
            <button
              onClick={() => setStep(step + 1)}
              disabled={!canNext}
              className="inline-flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-colors disabled:opacity-30"
              style={{ backgroundColor: 'var(--color-accent)', color: '#fff' }}
            >
              Next <ChevronRight size={14} />
            </button>
          ) : (
            <button
              onClick={handleSubmit}
              disabled={!canNext || submitting}
              className="inline-flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-colors disabled:opacity-30"
              style={{ backgroundColor: 'var(--color-accent)', color: '#fff' }}
            >
              {submitting ? 'Creating…' : 'Create Space'}
            </button>
          )}
        </div>
      </div>
    </div>
  )
}
