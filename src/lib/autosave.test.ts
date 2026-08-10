import { afterEach, describe, expect, it, vi } from 'vitest'
import { createAutosaveCoordinator } from '@/lib/autosave'

function deferred() {
  let resolve!: () => void
  const promise = new Promise<void>((done) => {
    resolve = done
  })
  return { promise, resolve }
}

describe('createAutosaveCoordinator', () => {
  afterEach(() => {
    vi.useRealTimers()
  })

  it('debounces drafts and persists only the newest value', async () => {
    vi.useFakeTimers()
    const persist = vi.fn(async () => undefined)
    const autosave = createAutosaveCoordinator(persist, 100)

    autosave.schedule('first')
    autosave.schedule('latest')
    await vi.advanceTimersByTimeAsync(100)

    expect(persist).toHaveBeenCalledTimes(1)
    expect(persist).toHaveBeenCalledWith('latest')
    expect(autosave.hasPending()).toBe(false)
  })

  it('serializes writes so a newer draft cannot race an in-flight save', async () => {
    vi.useFakeTimers()
    const first = deferred()
    const persist = vi
      .fn()
      .mockImplementationOnce(() => first.promise)
      .mockResolvedValueOnce(undefined)
    const autosave = createAutosaveCoordinator(persist, 100)

    autosave.schedule('first')
    await vi.advanceTimersByTimeAsync(100)
    autosave.schedule('second')
    await vi.advanceTimersByTimeAsync(100)

    expect(persist).toHaveBeenCalledTimes(1)
    first.resolve()
    await Promise.resolve()
    await vi.advanceTimersByTimeAsync(100)

    expect(persist.mock.calls).toEqual([['first'], ['second']])
  })

  it('flushes a pending draft immediately during editor teardown', async () => {
    vi.useFakeTimers()
    const persist = vi.fn(async () => undefined)
    const autosave = createAutosaveCoordinator(persist, 10_000)

    autosave.schedule({ title: 'Draft', content: 'Keep me' })
    await autosave.flush()

    expect(persist).toHaveBeenCalledWith({ title: 'Draft', content: 'Keep me' })
    expect(autosave.hasPending()).toBe(false)
  })
})
