import { useEffect, useMemo, useRef, useState } from 'react'
import { open } from '@tauri-apps/plugin-dialog'
import {
  ArrowLeft,
  ArrowRight,
  BriefcaseBusiness,
  Check,
  Code2,
  Command,
  FolderOpen,
  GraduationCap,
  LoaderCircle,
  ShieldCheck,
  Square,
  UserRound,
} from 'lucide-react'
import { AiSettings } from '@/components/ai/AiSettings'
import { SPACE_MODULES } from '@/components/spaceOptions'
import * as db from '@/lib/db/tauri'
import type { Space, UserProfile } from '@/lib/db/types'
import { cn } from '@/lib/utils'

export const START_ONBOARDING_TOUR_EVENT = 'aether:start-onboarding-tour'

type PersonaId = 'student' | 'developer' | 'professional' | 'personal' | 'blank'

type Persona = {
  id: PersonaId
  label: string
  description: string
  spaceName: string
  templateType: string
  modules: string[]
  icon: typeof GraduationCap
}

const PERSONAS: Persona[] = [
  {
    id: 'student',
    label: 'Student',
    description: 'Study material, assignments and durable learning context.',
    spaceName: 'Study',
    templateType: 'school',
    modules: ['notes', 'tasks', 'files', 'memory', 'ai', 'activity'],
    icon: GraduationCap,
  },
  {
    id: 'developer',
    label: 'Developer',
    description: 'Projects, source material, tasks and technical conversations.',
    spaceName: 'Projects',
    templateType: 'developer',
    modules: ['files', 'tasks', 'notes', 'ai', 'activity'],
    icon: Code2,
  },
  {
    id: 'professional',
    label: 'Professional',
    description: 'Focused work, decisions, documents and follow-through.',
    spaceName: 'Work',
    templateType: 'professional',
    modules: ['tasks', 'notes', 'files', 'ai', 'activity'],
    icon: BriefcaseBusiness,
  },
  {
    id: 'personal',
    label: 'Personal',
    description: 'A private home for notes, plans, files and memory.',
    spaceName: 'Personal',
    templateType: 'personal',
    modules: ['notes', 'tasks', 'files', 'memory'],
    icon: UserRound,
  },
  {
    id: 'blank',
    label: 'Blank',
    description: 'Start quietly and choose only what you need.',
    spaceName: 'My Space',
    templateType: 'blank',
    modules: ['notes', 'tasks', 'files'],
    icon: Square,
  },
]

const STEPS = ['Welcome', 'Use', 'Space', 'Sources', 'AI', 'Ready'] as const

export function Onboarding({
  profile,
  tourMode = false,
  onComplete,
}: {
  profile: UserProfile | null
  tourMode?: boolean
  onComplete: (profile: UserProfile | null) => void
}) {
  const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  const [step, setStep] = useState(0)
  const [personaId, setPersonaId] = useState<PersonaId>('blank')
  const persona = useMemo(
    () => PERSONAS.find((candidate) => candidate.id === personaId) ?? PERSONAS[4],
    [personaId],
  )
  const [spaceName, setSpaceName] = useState(persona.spaceName)
  const [modules, setModules] = useState<string[]>(persona.modules)
  const [workspaceSpace, setWorkspaceSpace] = useState<Space | null>(null)
  const [authorizedSource, setAuthorizedSource] = useState<string | null>(null)
  const [loadingExisting, setLoadingExisting] = useState(isTauri && !tourMode)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const headingRef = useRef<HTMLHeadingElement>(null)
  const spaceCreationRef = useRef<Promise<Space | null> | null>(null)

  useEffect(() => {
    headingRef.current?.focus()
  }, [step])

  useEffect(() => {
    if (!isTauri || tourMode) {
      setLoadingExisting(false)
      return
    }
    void db
      .listTopLevelSpaces()
      .then((spaces) =>
        setWorkspaceSpace(spaces.find((space) => !space.archived_at) ?? null),
      )
      .catch((cause) => setError(message(cause, 'Could not inspect existing Spaces.')))
      .finally(() => setLoadingExisting(false))
  }, [isTauri, tourMode])

  function choosePersona(nextPersona: Persona) {
    setPersonaId(nextPersona.id)
    setSpaceName(nextPersona.spaceName)
    setModules(nextPersona.modules)
  }

  function toggleModule(moduleId: string) {
    setModules((current) =>
      current.includes(moduleId)
        ? current.filter((candidate) => candidate !== moduleId)
        : [...current, moduleId],
    )
  }

  async function ensureSpace(): Promise<Space | null> {
    if (workspaceSpace || tourMode || !isTauri) return workspaceSpace
    if (spaceCreationRef.current) return spaceCreationRef.current
    if (!spaceName.trim()) throw new Error('Give your first Space a name.')
    if (modules.length === 0) throw new Error('Choose at least one Space module.')

    spaceCreationRef.current = db
      .createSpaceWithModules({
        name: spaceName.trim(),
        description: persona.description,
        templateType: persona.templateType,
        moduleTypes: modules,
      })
      .then((result) => {
        setWorkspaceSpace(result.space)
        return result.space
      })
    try {
      return await spaceCreationRef.current
    } finally {
      spaceCreationRef.current = null
    }
  }

  async function next() {
    setError(null)
    if (step === 2 && !tourMode) {
      setBusy(true)
      try {
        await ensureSpace()
      } catch (cause) {
        setError(message(cause, 'Could not create your first Space.'))
        return
      } finally {
        setBusy(false)
      }
    }
    setStep((current) => Math.min(current + 1, STEPS.length - 1))
  }

  async function authorizeFolder() {
    setError(null)
    if (!isTauri) {
      setError('Folder authorization is available in the Windows desktop app.')
      return
    }
    setBusy(true)
    try {
      const space = await ensureSpace()
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Choose one folder Aether may observe',
      })
      if (!selected) return
      const displayName =
        selected
          .replace(/[\\/]+$/, '')
          .split(/[\\/]/)
          .pop() || 'Local source'
      await db.createSource({
        rootPath: selected,
        displayName,
        spaceId: space?.id ?? null,
      })
      setAuthorizedSource(displayName)
    } catch (cause) {
      setError(message(cause, 'Could not authorize this folder.'))
    } finally {
      setBusy(false)
    }
  }

  async function finish() {
    setBusy(true)
    setError(null)
    try {
      if (tourMode || !isTauri) {
        onComplete(profile)
        return
      }
      await ensureSpace()
      if (!profile) throw new Error('The local profile is unavailable.')
      const completed = await db.updateProfile(profile.id, undefined, true)
      if (!completed) throw new Error('The local profile could not be completed.')
      onComplete(completed)
    } catch (cause) {
      setError(message(cause, 'Could not finish setup.'))
    } finally {
      setBusy(false)
    }
  }

  const canContinue = step !== 2 || tourMode || (!!spaceName.trim() && modules.length > 0)

  return (
    <main className="onboarding-shell">
      <div className="onboarding-frame">
        <header className="onboarding-brand" aria-label="Aether onboarding">
          <div>
            <span className="onboarding-wordmark">Aether</span>
            <span className="onboarding-caption">
              {tourMode ? 'Workspace tour' : 'First-run setup'}
            </span>
          </div>
          <ol className="onboarding-progress" aria-label="Setup progress">
            {STEPS.map((label, index) => (
              <li
                key={label}
                aria-current={index === step ? 'step' : undefined}
                className={cn(index <= step && 'is-active')}
              >
                <span>{index + 1}</span>
                <span className="sr-only">{label}</span>
              </li>
            ))}
          </ol>
        </header>

        <section className="onboarding-content" aria-busy={busy || loadingExisting}>
          {step === 0 && (
            <Step
              headingRef={headingRef}
              eyebrow="Welcome"
              title="Bring your work back together"
            >
              <p className="onboarding-lead">
                Aether keeps your Spaces, notes, tasks, files, Memory and conversations in
                one calm Windows workspace.
              </p>
              <div className="onboarding-assurance">
                <ShieldCheck size={20} aria-hidden="true" />
                <div>
                  <strong>Local by default</strong>
                  <p>
                    Your workspace stays on this PC. Nothing is sent to AI unless you
                    explicitly ask and choose the context.
                  </p>
                </div>
              </div>
              {!isTauri && (
                <p role="status" className="onboarding-note">
                  This browser preview does not create data or request folder access.
                </p>
              )}
              {tourMode && (
                <p role="status" className="onboarding-note">
                  Tour mode is non-destructive. Your existing workspace will not be reset.
                </p>
              )}
            </Step>
          )}

          {step === 1 && (
            <Step
              headingRef={headingRef}
              eyebrow="Starting point"
              title="What will Aether help you organize?"
              description="This only chooses editable modules for your first Space."
            >
              <div className="onboarding-personas">
                {PERSONAS.map((candidate) => {
                  const Icon = candidate.icon
                  return (
                    <button
                      type="button"
                      key={candidate.id}
                      aria-pressed={personaId === candidate.id}
                      onClick={() => choosePersona(candidate)}
                      className="onboarding-choice focus-ring"
                    >
                      <Icon size={19} aria-hidden="true" />
                      <span>
                        <strong>{candidate.label}</strong>
                        <small>{candidate.description}</small>
                      </span>
                      {personaId === candidate.id && (
                        <Check size={17} aria-hidden="true" />
                      )}
                    </button>
                  )
                })}
              </div>
            </Step>
          )}

          {step === 2 && (
            <Step
              headingRef={headingRef}
              eyebrow="Your first Space"
              title={
                workspaceSpace
                  ? 'Continue with ' + workspaceSpace.name
                  : 'Create a useful home'
              }
              description={
                workspaceSpace
                  ? 'A Space already exists, so setup will resume without creating a duplicate.'
                  : tourMode
                    ? 'In first-run mode, this creates one normal editable Space.'
                    : 'You can rename it and change every module later.'
              }
            >
              {!workspaceSpace && !tourMode && (
                <label className="onboarding-field">
                  <span>Space name</span>
                  <input
                    value={spaceName}
                    onChange={(event) => setSpaceName(event.target.value)}
                    maxLength={80}
                    autoFocus
                  />
                </label>
              )}
              <fieldset
                className="onboarding-modules"
                disabled={!!workspaceSpace || tourMode}
              >
                <legend>Modules</legend>
                <div>
                  {SPACE_MODULES.map((module) => (
                    <label key={module.id}>
                      <input
                        type="checkbox"
                        checked={modules.includes(module.id)}
                        onChange={() => toggleModule(module.id)}
                      />
                      <span>
                        <strong>{module.label}</strong>
                        <small>{module.description}</small>
                      </span>
                    </label>
                  ))}
                </div>
              </fieldset>
            </Step>
          )}

          {step === 3 && (
            <Step
              headingRef={headingRef}
              eyebrow="Optional Source"
              title="Choose what Aether may observe"
              description="Authorize one folder now, or add Sources later. Aether never scans your whole PC."
            >
              <div className="onboarding-assurance">
                <FolderOpen size={20} aria-hidden="true" />
                <div>
                  <strong>
                    {authorizedSource
                      ? authorizedSource + ' is authorized'
                      : 'Folder access is explicit and revocable'}
                  </strong>
                  <p>
                    Aether records local file metadata only. It does not reorganize files,
                    read their contents, or send them to AI.
                  </p>
                </div>
              </div>
              {!tourMode && (
                <button
                  type="button"
                  className="aether-button aether-button--secondary focus-ring"
                  onClick={() => void authorizeFolder()}
                  disabled={busy || !!authorizedSource}
                >
                  <FolderOpen size={15} aria-hidden="true" />
                  {authorizedSource ? 'Folder authorized' : 'Choose one folder'}
                </button>
              )}
              <p className="onboarding-note">
                Skipping this does not limit Notes or Tasks.
              </p>
            </Step>
          )}

          {step === 4 && (
            <Step
              headingRef={headingRef}
              eyebrow="Optional intelligence"
              title="AI works only when you configure it"
              description="You can add a provider key now or continue without AI."
            >
              <div className="onboarding-ai">
                <AiSettings />
              </div>
            </Step>
          )}

          {step === 5 && (
            <Step
              headingRef={headingRef}
              eyebrow="Ready"
              title={tourMode ? 'That is the Aether rhythm' : 'Your workspace is ready'}
              description="Start in Pulse, then use Search whenever you need to move quickly."
            >
              <div className="onboarding-command">
                <Command size={21} aria-hidden="true" />
                <div>
                  <strong>Search anything with Ctrl+K</strong>
                  <p>
                    Open Spaces, notes, tasks, files, Memory, conversations or commands.
                  </p>
                </div>
                <kbd>Ctrl</kbd>
                <span>+</span>
                <kbd>K</kbd>
              </div>
              <ul className="onboarding-summary">
                <li>
                  <Check size={15} /> Local workspace
                </li>
                <li>
                  <Check size={15} /> Editable Spaces
                </li>
                <li>
                  <Check size={15} /> User-controlled context
                </li>
              </ul>
            </Step>
          )}

          {error && (
            <p role="alert" className="onboarding-error">
              {error}
            </p>
          )}
        </section>

        <footer className="onboarding-actions">
          <button
            type="button"
            className="aether-button aether-button--secondary focus-ring"
            onClick={() => setStep((current) => Math.max(0, current - 1))}
            disabled={step === 0 || busy}
          >
            <ArrowLeft size={15} aria-hidden="true" /> Back
          </button>
          <span className="onboarding-step-label">
            {loadingExisting
              ? 'Checking local workspace…'
              : String(step + 1) + ' of ' + String(STEPS.length)}
          </span>
          {step < STEPS.length - 1 ? (
            <button
              type="button"
              className="aether-button aether-button--primary focus-ring"
              onClick={() => void next()}
              disabled={!canContinue || busy || loadingExisting}
            >
              {busy && <LoaderCircle size={15} className="animate-spin" />}
              Continue <ArrowRight size={15} aria-hidden="true" />
            </button>
          ) : (
            <button
              type="button"
              className="aether-button aether-button--primary focus-ring"
              onClick={() => void finish()}
              disabled={busy || loadingExisting}
            >
              {busy && <LoaderCircle size={15} className="animate-spin" />}
              {tourMode ? 'Return to Aether' : 'Enter Pulse'}
            </button>
          )}
        </footer>
      </div>
    </main>
  )
}

function Step({
  headingRef,
  eyebrow,
  title,
  description,
  children,
}: {
  headingRef: React.RefObject<HTMLHeadingElement | null>
  eyebrow: string
  title: string
  description?: string
  children: React.ReactNode
}) {
  return (
    <div className="onboarding-step">
      <p className="onboarding-eyebrow">{eyebrow}</p>
      <h1 ref={headingRef} tabIndex={-1}>
        {title}
      </h1>
      {description && <p className="onboarding-description">{description}</p>}
      <div className="onboarding-step-body">{children}</div>
    </div>
  )
}

function message(cause: unknown, fallback: string) {
  return cause instanceof Error && cause.message ? cause.message : fallback
}
