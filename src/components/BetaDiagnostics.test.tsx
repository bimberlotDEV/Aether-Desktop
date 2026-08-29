import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({ getBetaDiagnostics: vi.fn() }))
vi.mock('@/lib/db/tauri', () => mocks)

import { BetaDiagnostics } from '@/components/BetaDiagnostics'

const report = {
  appVersion: '0.5.0',
  databaseSchema: '010_ai_route_provenance',
  databaseIntegrity: 'ok' as const,
  platform: 'Windows x86_64',
  updaterConfigured: false,
  trayAvailable: true,
  shortcutRegistered: true,
  notificationsAvailable: true,
}

describe('Beta diagnostics', () => {
  const writeText = vi.fn()

  beforeEach(() => {
    mocks.getBetaDiagnostics.mockReset().mockResolvedValue(report)
    writeText.mockReset().mockResolvedValue(undefined)
    Object.defineProperty(navigator, 'clipboard', {
      value: { writeText },
      configurable: true,
    })
  })

  afterEach(() => {
    Reflect.deleteProperty(window, '__TAURI_INTERNALS__')
  })

  it('is honest and inactive outside the installed desktop app', () => {
    render(<BetaDiagnostics />)

    expect(screen.getByRole('button', { name: 'Create report' })).toBeDisabled()
    expect(screen.getByRole('status')).toHaveTextContent(/installed Aether desktop app/i)
    expect(mocks.getBetaDiagnostics).not.toHaveBeenCalled()
  })

  it('shows the exact sanitized report before an explicit copy', async () => {
    Object.defineProperty(window, '__TAURI_INTERNALS__', {
      value: {},
      configurable: true,
    })
    const user = userEvent.setup()
    Object.defineProperty(navigator, 'clipboard', {
      value: { writeText },
      configurable: true,
    })
    render(<BetaDiagnostics />)

    await user.click(screen.getByRole('button', { name: 'Create report' }))
    const preview = await screen.findByLabelText('Sanitized diagnostic report')
    expect(preview).toHaveTextContent('010_ai_route_provenance')
    expect(preview).not.toHaveTextContent(/path|prompt|filename|apiKey/i)
    expect(writeText).not.toHaveBeenCalled()

    await user.click(screen.getByRole('button', { name: 'Copy report' }))
    expect(writeText).toHaveBeenCalledWith(JSON.stringify(report, null, 2))
    expect(await screen.findByRole('status')).toHaveTextContent(
      /review it before sharing/i,
    )
  })

  it('announces generation and clipboard failures without stale output', async () => {
    Object.defineProperty(window, '__TAURI_INTERNALS__', {
      value: {},
      configurable: true,
    })
    mocks.getBetaDiagnostics.mockRejectedValueOnce(new Error('Integrity check failed'))
    const user = userEvent.setup()
    render(<BetaDiagnostics />)

    await user.click(screen.getByRole('button', { name: 'Create report' }))
    expect(await screen.findByRole('alert')).toHaveTextContent('Integrity check failed')
    expect(screen.queryByLabelText('Sanitized diagnostic report')).not.toBeInTheDocument()
  })

  it('keeps loading visible and prevents duplicate generation', async () => {
    Object.defineProperty(window, '__TAURI_INTERNALS__', {
      value: {},
      configurable: true,
    })
    let resolveReport: (value: typeof report) => void = () => {}
    mocks.getBetaDiagnostics.mockReturnValueOnce(
      new Promise((resolve) => {
        resolveReport = resolve
      }),
    )
    const user = userEvent.setup()
    render(<BetaDiagnostics />)

    await user.click(screen.getByRole('button', { name: 'Create report' }))
    expect(screen.getByRole('button', { name: 'Checking…' })).toBeDisabled()
    resolveReport(report)
    expect(
      await screen.findByLabelText('Sanitized diagnostic report'),
    ).toBeInTheDocument()
  })

  it('announces a clipboard denial and leaves the preview visible', async () => {
    Object.defineProperty(window, '__TAURI_INTERNALS__', {
      value: {},
      configurable: true,
    })
    const user = userEvent.setup()
    writeText.mockRejectedValueOnce(new Error('Clipboard permission denied'))
    Object.defineProperty(navigator, 'clipboard', {
      value: { writeText },
      configurable: true,
    })
    render(<BetaDiagnostics />)

    await user.click(screen.getByRole('button', { name: 'Create report' }))
    await user.click(await screen.findByRole('button', { name: 'Copy report' }))
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Clipboard permission denied',
    )
    expect(screen.getByLabelText('Sanitized diagnostic report')).toBeInTheDocument()
  })
})
