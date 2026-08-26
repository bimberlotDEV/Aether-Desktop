import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({ universalSearch: vi.fn() }))
vi.mock('@/lib/db/tauri', () => mocks)
vi.mock('@/hooks/useSpaces', () => ({ useSpaces: () => ({ spaces: [] }) }))

import { CommandPalette } from '@/components/CommandPalette'
import type { UniversalSearchResult } from '@/lib/db/types'
import { useCommandStore } from '@/stores/commandStore'

const noteResult: UniversalSearchResult = {
  kind: 'note',
  entityId: 'note-1',
  spaceId: 'space-1',
  title: 'Gradient descent',
  subtitle: 'Optimization notes',
  provenance: 'Note',
  score: 900,
  updatedAt: '2026-08-26 12:00:00',
  sourceId: null,
  relativePath: null,
}

describe('Universal Search palette', () => {
  beforeEach(() => {
    window.history.replaceState({}, '', '/')
    Reflect.deleteProperty(window, '__TAURI_INTERNALS__')
    useCommandStore.setState({ isOpen: true, query: '' })
    mocks.universalSearch.mockReset()
    mocks.universalSearch.mockResolvedValue([])
  })

  it('navigates BrowserRouter commands without a stale hash URL', async () => {
    const user = userEvent.setup()
    useCommandStore.setState({ query: 'Go to Tasks' })
    render(<CommandPalette />)

    await user.click(screen.getByRole('option', { name: /Go to Tasks/ }))
    expect(window.location.pathname).toBe('/tasks')
    expect(window.location.hash).toBe('')
    expect(useCommandStore.getState().isOpen).toBe(false)
  })

  it('is honest in browser mode and never fabricates workspace results', () => {
    useCommandStore.setState({ query: 'gradient' })
    render(<CommandPalette />)
    expect(screen.getByText(/Browser mode searches commands only/)).toBeInTheDocument()
    expect(mocks.universalSearch).not.toHaveBeenCalled()
  })

  it('debounces desktop search, shows provenance, and opens a specific domain', async () => {
    Object.defineProperty(window, '__TAURI_INTERNALS__', {
      value: {},
      configurable: true,
    })
    mocks.universalSearch.mockResolvedValue([noteResult])
    useCommandStore.setState({ query: 'gradient' })
    render(<CommandPalette />)

    expect(
      await screen.findByRole('option', { name: /Gradient descent/ }),
    ).toBeInTheDocument()
    expect(mocks.universalSearch).toHaveBeenCalledWith('gradient', null, 30)
    fireEvent.keyDown(screen.getByRole('searchbox'), { key: 'Enter' })
    expect(window.location.pathname).toBe('/spaces/space-1/notes')
  })

  it('suppresses stale search responses', async () => {
    Object.defineProperty(window, '__TAURI_INTERNALS__', {
      value: {},
      configurable: true,
    })
    const alpha = deferred<UniversalSearchResult[]>()
    const beta = deferred<UniversalSearchResult[]>()
    mocks.universalSearch
      .mockReturnValueOnce(alpha.promise)
      .mockReturnValueOnce(beta.promise)
    useCommandStore.setState({ query: 'alpha' })
    render(<CommandPalette />)
    await waitFor(() => expect(mocks.universalSearch).toHaveBeenCalledTimes(1))

    fireEvent.change(screen.getByRole('searchbox'), { target: { value: 'beta' } })
    await waitFor(() => expect(mocks.universalSearch).toHaveBeenCalledTimes(2))
    beta.resolve([{ ...noteResult, entityId: 'beta', title: 'Beta result' }])
    expect(await screen.findByText('Beta result')).toBeInTheDocument()
    alpha.resolve([{ ...noteResult, entityId: 'alpha', title: 'Alpha stale' }])
    await Promise.resolve()
    expect(screen.queryByText('Alpha stale')).not.toBeInTheDocument()
  })
})

function deferred<T>() {
  let resolve!: (value: T) => void
  const promise = new Promise<T>((next) => {
    resolve = next
  })
  return { promise, resolve }
}
