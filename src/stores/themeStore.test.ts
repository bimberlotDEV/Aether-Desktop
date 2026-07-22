import { describe, it, expect, beforeEach } from 'vitest'
import { useThemeStore } from './themeStore'

describe('themeStore', () => {
  beforeEach(() => {
    useThemeStore.setState({ theme: 'system', resolved: 'dark' })
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
})
