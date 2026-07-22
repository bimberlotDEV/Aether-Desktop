import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useThemeStore } from './themeStore'

// Mock the dynamic import for tests
vi.mock('@/lib/db/tauri', () => ({
  getSetting: async () => null,
  setSetting: async () => {},
}))

describe('themeStore', () => {
  beforeEach(() => {
    useThemeStore.setState({ theme: 'system', resolved: 'dark', _isReady: false })
    localStorage.clear()
  })

  it('starts with system theme', () => {
    const { theme } = useThemeStore.getState()
    expect(theme).toBe('system')
  })

  it('can set to light', () => {
    useThemeStore.getState().setTheme('light')
    const { theme, resolved } = useThemeStore.getState()
    expect(theme).toBe('light')
    expect(resolved).toBe('light')
  })

  it('can set to dark', () => {
    useThemeStore.getState().setTheme('dark')
    const { theme, resolved } = useThemeStore.getState()
    expect(theme).toBe('dark')
    expect(resolved).toBe('dark')
  })

  it('persists to localStorage', () => {
    useThemeStore.getState().setTheme('light')
    expect(localStorage.getItem('aether-theme')).toBe('light')
  })
})
