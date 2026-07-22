import { create } from 'zustand'

export type Theme = 'light' | 'dark' | 'system'

interface ThemeState {
  theme: Theme
  resolved: 'light' | 'dark'
  setTheme: (theme: Theme) => void
  _apply: (theme: Theme) => void
}

function getSystemPreference(): 'light' | 'dark' {
  if (typeof window === 'undefined') return 'dark'
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

function applyTheme(theme: Theme): 'light' | 'dark' {
  const root = document.documentElement
  const resolved = theme === 'system' ? getSystemPreference() : theme
  root.classList.toggle('dark', resolved === 'dark')
  return resolved
}

export const useThemeStore = create<ThemeState>((set) => ({
  theme: 'system',
  resolved: 'dark',

  setTheme: (theme: Theme) => {
    const resolved = applyTheme(theme)
    set({ theme, resolved })
    try { localStorage.setItem('aether-theme', theme) } catch {}
  },

  _apply: (theme: Theme) => {
    const resolved = applyTheme(theme)
    set({ theme, resolved })
  },
}))

// Listen for system preference changes (guarded for SSR/test environments)
if (typeof window !== 'undefined' && typeof window.matchMedia === 'function') {
  try {
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
      const { theme, _apply } = useThemeStore.getState()
      if (theme === 'system') _apply('system')
    })
  } catch {
    // matchMedia not available (test environment)
  }
}

// Initialize from localStorage
export function initTheme() {
  let theme: Theme = 'system'
  try {
    const stored = localStorage.getItem('aether-theme')
    if (stored === 'light' || stored === 'dark' || stored === 'system') {
      theme = stored
    }
  } catch {}
  useThemeStore.getState()._apply(theme)
}
