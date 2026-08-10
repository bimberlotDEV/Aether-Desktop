export interface AutosaveCoordinator<T> {
  schedule(value: T): void
  flush(): Promise<void>
  hasPending(): boolean
}

export function createAutosaveCoordinator<T>(
  persist: (value: T) => Promise<void>,
  delayMs = 800,
): AutosaveCoordinator<T> {
  let timer: ReturnType<typeof setTimeout> | null = null
  let pending: T | null = null
  let inFlight: Promise<void> | null = null

  const clearTimer = () => {
    if (timer) clearTimeout(timer)
    timer = null
  }

  const runNext = async (): Promise<void> => {
    if (inFlight || pending === null) return

    const value = pending
    pending = null
    const operation = persist(value)
    inFlight = operation
    try {
      await operation
    } catch {
      // The persistence callback owns user-facing error reporting.
    } finally {
      if (inFlight === operation) inFlight = null
      if (pending !== null && timer === null) {
        timer = setTimeout(() => {
          timer = null
          void runNext()
        }, delayMs)
      }
    }
  }

  return {
    schedule(value) {
      pending = value
      clearTimer()
      timer = setTimeout(() => {
        timer = null
        void runNext()
      }, delayMs)
    },

    async flush() {
      clearTimer()
      while (inFlight || pending !== null) {
        if (inFlight) {
          try {
            await inFlight
          } catch {
            // The persistence callback owns error reporting; continue with any newer draft.
          }
        } else {
          clearTimer()
          await runNext()
        }
      }
    },

    hasPending() {
      return pending !== null || inFlight !== null
    },
  }
}
