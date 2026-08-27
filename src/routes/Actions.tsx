import { useMemo, useState, type FormEvent, type ReactNode } from 'react'
import { useNavigate } from 'react-router-dom'
import {
  ArrowRight,
  CheckCircle2,
  Clock3,
  Database,
  FolderInput,
  ShieldCheck,
  X,
} from 'lucide-react'
import { useActions } from '@/hooks/useActions'
import type { ActionRequest } from '@/lib/db/types'
import {
  Button,
  EmptyState,
  Page,
  PageHeader,
  SectionLabel,
  Surface,
} from '@/components/ui/AetherUI'

type ActionType = ActionRequest['type']

const choices: { value: ActionType; label: string; detail: string }[] = [
  { value: 'createTask', label: 'Create Task', detail: 'Add a Task to your workspace' },
  { value: 'createNote', label: 'Create Note', detail: 'Add a Note inside a Space' },
  {
    value: 'createFolder',
    label: 'Create folder',
    detail: 'Inside one authorized Source',
  },
  { value: 'copyFile', label: 'Copy file', detail: 'Without replacing an existing file' },
  { value: 'moveFile', label: 'Move file', detail: 'Within the same authorized Source' },
  {
    value: 'renameFile',
    label: 'Rename file',
    detail: 'Within the same authorized Source',
  },
  { value: 'openFile', label: 'Open file', detail: 'An indexed file in its Windows app' },
  {
    value: 'openFolder',
    label: 'Open Source folder',
    detail: 'Open an authorized root in Explorer',
  },
]

export function Actions() {
  const navigate = useNavigate()
  const actions = useActions()

  return (
    <Page width="default" className="aether-actions">
      <PageHeader
        eyebrow="Safe Actions"
        icon={ShieldCheck}
        title="Act with you in control"
        description="Aether validates the exact change first. Nothing runs until you approve its preview."
      />

      {!actions.isDesktop ? (
        <Surface className="aether-action-disclosure" role="status">
          <Database size={18} aria-hidden="true" />
          <div>
            <strong>Actions are available in the installed desktop app</strong>
            <p>The browser preview cannot access or change your local workspace.</p>
          </div>
        </Surface>
      ) : actions.loading ? (
        <Surface className="aether-action-disclosure" role="status" aria-live="polite">
          <Clock3 size={18} aria-hidden="true" />
          <p>Loading your approved Spaces and Sources…</p>
        </Surface>
      ) : actions.preview ? (
        <ActionReview
          preview={actions.preview}
          busy={actions.busy}
          error={actions.error}
          onApprove={() => void actions.approve()}
          onCancel={() => void actions.cancel()}
        />
      ) : actions.result ? (
        <ActionComplete
          title={actions.result.title}
          detail={actions.result.detail}
          destination={actions.result.destination}
          onOpen={() => navigate(actions.result!.destination)}
          onReset={actions.reset}
        />
      ) : (
        <ActionComposer actions={actions} />
      )}
    </Page>
  )
}

function ActionComposer({ actions }: { actions: ReturnType<typeof useActions> }) {
  const [type, setType] = useState<ActionType>('createTask')
  const [title, setTitle] = useState('')
  const [content, setContent] = useState('')
  const [dueDate, setDueDate] = useState('')
  const [spaceId, setSpaceId] = useState('')
  const [sourceId, setSourceId] = useState('')
  const [fromPath, setFromPath] = useState('')
  const [toPath, setToPath] = useState('')
  const [newName, setNewName] = useState('')
  const selected = useMemo(() => choices.find((choice) => choice.value === type)!, [type])
  const needsSpace = type === 'createTask' || type === 'createNote'
  const needsSource = !needsSpace
  const submit = (event: FormEvent) => {
    event.preventDefault()
    const request = buildRequest({
      type,
      title,
      content,
      dueDate,
      spaceId,
      sourceId,
      fromPath,
      toPath,
      newName,
    })
    if (request) void actions.propose(request)
  }

  return (
    <div className="aether-action-layout">
      <Surface className="aether-action-composer">
        <SectionLabel meta="Step 1 of 2">Choose and describe</SectionLabel>
        <form onSubmit={submit} className="aether-action-form">
          <Field label="Action">
            <select
              className="aether-field"
              value={type}
              onChange={(event) => setType(event.target.value as ActionType)}
            >
              {choices.map((choice) => (
                <option key={choice.value} value={choice.value}>
                  {choice.label} — {choice.detail}
                </option>
              ))}
            </select>
          </Field>

          {needsSpace && (
            <Field label={type === 'createNote' ? 'Space' : 'Space (optional)'}>
              <select
                className="aether-field"
                value={spaceId}
                required={type === 'createNote'}
                onChange={(event) => setSpaceId(event.target.value)}
              >
                <option value="">
                  {type === 'createNote' ? 'Choose a Space' : 'Task Inbox'}
                </option>
                {actions.spaces.map((space) => (
                  <option key={space.id} value={space.id}>
                    {space.name}
                  </option>
                ))}
              </select>
            </Field>
          )}

          {needsSource && (
            <Field label="Authorized Source">
              <select
                className="aether-field"
                value={sourceId}
                required
                onChange={(event) => setSourceId(event.target.value)}
              >
                <option value="">Choose a Source</option>
                {actions.sources.map((source) => (
                  <option key={source.id} value={source.id}>
                    {source.displayName}
                  </option>
                ))}
              </select>
            </Field>
          )}

          {(type === 'createTask' || type === 'createNote') && (
            <Field label="Title">
              <input
                className="aether-field"
                value={title}
                maxLength={200}
                required
                onChange={(event) => setTitle(event.target.value)}
              />
            </Field>
          )}
          {type === 'createTask' && (
            <>
              <Field label="Description (optional)">
                <textarea
                  className="aether-field"
                  value={content}
                  maxLength={10_000}
                  onChange={(event) => setContent(event.target.value)}
                />
              </Field>
              <Field label="Due date (optional)">
                <input
                  className="aether-field"
                  type="date"
                  value={dueDate}
                  onChange={(event) => setDueDate(event.target.value)}
                />
              </Field>
            </>
          )}
          {type === 'createNote' && (
            <Field label="Content (optional)">
              <textarea
                className="aether-field"
                value={content}
                maxLength={20_000}
                onChange={(event) => setContent(event.target.value)}
              />
            </Field>
          )}
          {(type === 'createFolder' || type === 'openFile') && (
            <Field
              label={
                type === 'createFolder'
                  ? 'New relative folder path'
                  : 'Indexed relative file path'
              }
              hint="Relative to the selected Source; absolute paths and .. are rejected."
            >
              <input
                className="aether-field"
                value={type === 'createFolder' ? toPath : fromPath}
                required
                onChange={(event) =>
                  type === 'createFolder'
                    ? setToPath(event.target.value)
                    : setFromPath(event.target.value)
                }
              />
            </Field>
          )}
          {(type === 'copyFile' || type === 'moveFile') && (
            <>
              <Field label="Indexed source file">
                <input
                  className="aether-field"
                  value={fromPath}
                  required
                  onChange={(event) => setFromPath(event.target.value)}
                />
              </Field>
              <Field label="New relative destination">
                <input
                  className="aether-field"
                  value={toPath}
                  required
                  onChange={(event) => setToPath(event.target.value)}
                />
              </Field>
            </>
          )}
          {type === 'renameFile' && (
            <>
              <Field label="Indexed source file">
                <input
                  className="aether-field"
                  value={fromPath}
                  required
                  onChange={(event) => setFromPath(event.target.value)}
                />
              </Field>
              <Field label="New file name">
                <input
                  className="aether-field"
                  value={newName}
                  required
                  onChange={(event) => setNewName(event.target.value)}
                />
              </Field>
            </>
          )}

          <div role="alert" aria-live="assertive" className="aether-action-error">
            {actions.error}
          </div>
          <Button
            type="submit"
            variant="primary"
            icon={ArrowRight}
            disabled={actions.busy}
          >
            {actions.busy ? 'Validating…' : `Preview ${selected.label}`}
          </Button>
        </form>
      </Surface>

      <aside className="aether-action-guardrails" aria-label="Action safeguards">
        <ShieldCheck size={20} aria-hidden="true" />
        <div>
          <strong>Bounded by design</strong>
          <p>
            No shell commands, deletion, overwrites, cross-Source moves, or silent
            execution.
          </p>
        </div>
        <div>
          <strong>Files stay contained</strong>
          <p>
            File changes are limited to one Source you already authorized. Rescan Sources
            after a file change to refresh its index.
          </p>
        </div>
      </aside>
    </div>
  )
}

function ActionReview({
  preview,
  busy,
  error,
  onApprove,
  onCancel,
}: {
  preview: ReturnType<typeof useActions>['preview'] & {}
  busy: boolean
  error: string | null
  onApprove: () => void
  onCancel: () => void
}) {
  return (
    <Surface className="aether-action-review">
      <SectionLabel meta="Step 2 of 2">Review before execution</SectionLabel>
      <span className="aether-icon-frame aether-icon-frame--accent">
        <FolderInput size={19} aria-hidden="true" />
      </span>
      <h2>{preview.title}</h2>
      <p>{preview.summary}</p>
      <div className="aether-action-consequence">
        <strong>What will change</strong>
        <p>{preview.consequence}</p>
      </div>
      <p className="aether-action-expiry">
        This one-time approval expires at{' '}
        {new Date(preview.expiresAt).toLocaleTimeString()}.
      </p>
      {error && (
        <p role="alert" className="aether-action-error">
          {error}
        </p>
      )}
      <div className="aether-action-buttons">
        <Button variant="quiet" icon={X} disabled={busy} onClick={onCancel}>
          Cancel
        </Button>
        <Button variant="primary" icon={ShieldCheck} disabled={busy} onClick={onApprove}>
          {busy ? 'Executing…' : 'Approve and execute'}
        </Button>
      </div>
    </Surface>
  )
}

function ActionComplete({
  title,
  detail,
  destination,
  onOpen,
  onReset,
}: {
  title: string
  detail: string
  destination: string
  onOpen: () => void
  onReset: () => void
}) {
  return (
    <EmptyState
      icon={CheckCircle2}
      eyebrow="Action completed"
      title={title}
      description={detail}
      action={{
        label: destination === '/sources' ? 'Open Sources' : 'Open result',
        onClick: onOpen,
      }}
      secondary={
        <Button variant="quiet" onClick={onReset}>
          New Action
        </Button>
      }
    />
  )
}

function Field({
  label,
  hint,
  children,
}: {
  label: string
  hint?: string
  children: ReactNode
}) {
  return (
    <label className="aether-action-field">
      <span>{label}</span>
      {children}
      {hint && <small>{hint}</small>}
    </label>
  )
}

function buildRequest(values: {
  type: ActionType
  title: string
  content: string
  dueDate: string
  spaceId: string
  sourceId: string
  fromPath: string
  toPath: string
  newName: string
}): ActionRequest | null {
  switch (values.type) {
    case 'createTask':
      return {
        type: 'createTask',
        title: values.title,
        description: values.content,
        dueDate: values.dueDate || null,
        spaceId: values.spaceId || null,
      }
    case 'createNote':
      return {
        type: 'createNote',
        title: values.title,
        content: values.content,
        spaceId: values.spaceId,
      }
    case 'createFolder':
      return {
        type: 'createFolder',
        sourceId: values.sourceId,
        relativePath: values.toPath,
      }
    case 'copyFile':
      return {
        type: 'copyFile',
        sourceId: values.sourceId,
        fromRelativePath: values.fromPath,
        toRelativePath: values.toPath,
      }
    case 'moveFile':
      return {
        type: 'moveFile',
        sourceId: values.sourceId,
        fromRelativePath: values.fromPath,
        toRelativePath: values.toPath,
      }
    case 'renameFile':
      return {
        type: 'renameFile',
        sourceId: values.sourceId,
        fromRelativePath: values.fromPath,
        newName: values.newName,
      }
    case 'openFile':
      return {
        type: 'openFile',
        sourceId: values.sourceId,
        relativePath: values.fromPath,
      }
    case 'openFolder':
      return { type: 'openFolder', sourceId: values.sourceId }
  }
}
