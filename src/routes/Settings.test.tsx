import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

vi.mock('@/stores/themeStore', () => ({
  useThemeStore: () => ({ theme: 'dark', setTheme: vi.fn() }),
}))
vi.mock('@/components/ai/AiSettings', () => ({ AiSettings: () => null }))
vi.mock('@/components/NativeSettings', () => ({ NativeSettings: () => null }))
vi.mock('@/components/BackupSettings', () => ({ BackupSettings: () => null }))
vi.mock('@/components/Onboarding', () => ({
  START_ONBOARDING_TOUR_EVENT: 'aether:start-onboarding-tour',
}))

import { Settings } from '@/routes/Settings'

describe('Settings onboarding tour', () => {
  it('opens the non-destructive tour through the shared shell event', () => {
    const listener = vi.fn()
    window.addEventListener('aether:start-onboarding-tour', listener)
    render(<Settings />)

    fireEvent.click(screen.getByRole('button', { name: /open tour/i }))

    expect(listener).toHaveBeenCalledTimes(1)
    window.removeEventListener('aether:start-onboarding-tour', listener)
  })
})
