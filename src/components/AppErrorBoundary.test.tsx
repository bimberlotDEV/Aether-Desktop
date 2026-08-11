import { render, screen } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { AppErrorBoundary } from '@/components/AppErrorBoundary'

function BrokenView(): never {
  throw new Error('render failed')
}

describe('AppErrorBoundary', () => {
  beforeEach(() => {
    vi.spyOn(console, 'error').mockImplementation(() => undefined)
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('replaces a blank root with a recoverable error screen', () => {
    render(
      <AppErrorBoundary>
        <BrokenView />
      </AppErrorBoundary>,
    )

    expect(screen.getByRole('alert')).toHaveTextContent('Your local data is safe')
    expect(screen.getByRole('button', { name: 'Reload Aether' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Go to Pulse' })).toBeInTheDocument()
  })
})
