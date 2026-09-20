import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { Space, UserProfile } from '@/lib/db/types'

const mocks = vi.hoisted(() => ({
  open: vi.fn(),
  listTopLevelSpaces: vi.fn(),
  createSpaceWithModules: vi.fn(),
  createSource: vi.fn(),
  updateProfile: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({ open: mocks.open }))
vi.mock('@/lib/db/tauri', () => ({
  listTopLevelSpaces: mocks.listTopLevelSpaces,
  createSpaceWithModules: mocks.createSpaceWithModules,
  createSource: mocks.createSource,
  updateProfile: mocks.updateProfile,
}))
vi.mock('@/components/ai/AiSettings', () => ({
  AiSettings: () => <p>Secure provider settings</p>,
}))

import { Onboarding } from '@/components/Onboarding'

const profile: UserProfile = {
  id: 'profile-1',
  display_name: null,
  onboarding_completed: false,
  created_at: '2026-08-29T00:00:00Z',
  updated_at: '2026-08-29T00:00:00Z',
}

const space: Space = {
  id: 'space-1',
  name: 'Projects',
  description: null,
  icon: null,
  accent: null,
  template_type: 'developer',
  favourite: false,
  archived_at: null,
  sort_order: 0,
  settings_json: null,
  parent_space_id: null,
  last_opened_at: null,
  created_at: '2026-08-29T00:00:00Z',
  updated_at: '2026-08-29T00:00:00Z',
}

function enableDesktop() {
  Object.defineProperty(window, '__TAURI_INTERNALS__', {
    configurable: true,
    value: {},
  })
}

async function continueToSpace() {
  fireEvent.click(screen.getByRole('button', { name: /continue/i }))
  await screen.findByRole('heading', { name: /what will aether help/i })
  fireEvent.click(screen.getByRole('button', { name: /developer/i }))
  fireEvent.click(screen.getByRole('button', { name: /continue/i }))
  await screen.findByRole('heading', { name: /create a useful home/i })
}

async function continueToReady() {
  fireEvent.click(screen.getByRole('button', { name: /continue/i }))
  await screen.findByRole('heading', { name: /choose what aether may observe/i })
  fireEvent.click(screen.getByRole('button', { name: /continue/i }))
  await screen.findByRole('heading', { name: /ai works only/i })
  fireEvent.click(screen.getByRole('button', { name: /continue/i }))
  await screen.findByRole('heading', { name: /workspace is ready/i })
}

describe('Onboarding', () => {
  beforeEach(() => {
    enableDesktop()
    Object.values(mocks).forEach((mock) => mock.mockReset())
    mocks.listTopLevelSpaces.mockResolvedValue([])
    mocks.createSpaceWithModules.mockResolvedValue({ space, modules: [] })
    mocks.updateProfile.mockResolvedValue({ ...profile, onboarding_completed: true })
  })

  it('creates one editable template Space and completes the local profile', async () => {
    const onComplete = vi.fn()
    render(<Onboarding profile={profile} onComplete={onComplete} />)
    await waitFor(() => expect(mocks.listTopLevelSpaces).toHaveBeenCalled())

    await continueToSpace()
    fireEvent.change(screen.getByLabelText(/space name/i), {
      target: { value: 'Aether project' },
    })
    await continueToReady()

    expect(mocks.createSpaceWithModules).toHaveBeenCalledTimes(1)
    expect(mocks.createSpaceWithModules).toHaveBeenCalledWith(
      expect.objectContaining({
        name: 'Aether project',
        templateType: 'developer',
        moduleTypes: ['files', 'tasks', 'notes', 'ai', 'activity'],
      }),
    )

    fireEvent.click(screen.getByRole('button', { name: /enter pulse/i }))
    await waitFor(() =>
      expect(mocks.updateProfile).toHaveBeenCalledWith('profile-1', undefined, true),
    )
    expect(onComplete).toHaveBeenCalledWith(
      expect.objectContaining({ onboarding_completed: true }),
    )
  })

  it('resumes an interrupted setup without creating a duplicate Space', async () => {
    mocks.listTopLevelSpaces.mockResolvedValue([space])
    render(<Onboarding profile={profile} onComplete={vi.fn()} />)

    await waitFor(() =>
      expect(screen.getByRole('button', { name: /continue/i })).toBeEnabled(),
    )
    fireEvent.click(screen.getByRole('button', { name: /continue/i }))
    await screen.findByRole('heading', { name: /what will aether help/i })
    fireEvent.click(screen.getByRole('button', { name: /continue/i }))
    await screen.findByRole('heading', { name: /continue with projects/i })
    fireEvent.click(screen.getByRole('button', { name: /continue/i }))
    await screen.findByRole('heading', { name: /choose what aether may observe/i })

    expect(mocks.createSpaceWithModules).not.toHaveBeenCalled()
  })

  it('keeps required Space failures visible and retryable', async () => {
    mocks.createSpaceWithModules
      .mockRejectedValueOnce(new Error('Database is busy'))
      .mockResolvedValueOnce({ space, modules: [] })
    render(<Onboarding profile={profile} onComplete={vi.fn()} />)
    await waitFor(() => expect(mocks.listTopLevelSpaces).toHaveBeenCalled())
    await continueToSpace()

    fireEvent.click(screen.getByRole('button', { name: /continue/i }))
    expect(await screen.findByRole('alert')).toHaveTextContent('Database is busy')
    expect(screen.getByRole('heading', { name: /create a useful home/i })).toBeVisible()

    fireEvent.click(screen.getByRole('button', { name: /continue/i }))
    await screen.findByRole('heading', { name: /choose what aether may observe/i })
    expect(mocks.createSpaceWithModules).toHaveBeenCalledTimes(2)
  })

  it('authorizes only the selected folder and does not start a scan', async () => {
    mocks.open.mockResolvedValue('C:\\Work')
    mocks.createSource.mockResolvedValue({ id: 'source-1' })
    render(<Onboarding profile={profile} onComplete={vi.fn()} />)
    await waitFor(() => expect(mocks.listTopLevelSpaces).toHaveBeenCalled())
    await continueToSpace()
    fireEvent.click(screen.getByRole('button', { name: /continue/i }))
    await screen.findByRole('heading', { name: /choose what aether may observe/i })

    fireEvent.click(screen.getByRole('button', { name: /choose one folder/i }))
    await waitFor(() =>
      expect(mocks.createSource).toHaveBeenCalledWith({
        rootPath: 'C:\\Work',
        displayName: 'Work',
        spaceId: 'space-1',
      }),
    )
    expect(screen.getByText(/work is authorized/i)).toBeVisible()
  })

  it('runs a non-destructive browser tour without profile or Space mutations', async () => {
    delete (window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__
    const onComplete = vi.fn()
    render(
      <Onboarding
        profile={{ ...profile, onboarding_completed: true }}
        tourMode
        onComplete={onComplete}
      />,
    )

    for (let index = 0; index < 5; index += 1) {
      fireEvent.click(screen.getByRole('button', { name: /continue/i }))
    }
    await screen.findByRole('heading', { name: /aether rhythm/i })
    fireEvent.click(screen.getByRole('button', { name: /return to aether/i }))

    expect(mocks.createSpaceWithModules).not.toHaveBeenCalled()
    expect(mocks.createSource).not.toHaveBeenCalled()
    expect(mocks.updateProfile).not.toHaveBeenCalled()
    expect(onComplete).toHaveBeenCalled()
  })
})
