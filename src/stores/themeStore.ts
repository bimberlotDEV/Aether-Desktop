import { create } from 'zustand'

export type Theme = 'light' | 'dark' | 'system'

interface ThemeState {
  theme: Theme
  resolved: 'light' | 'dark'
  setTheme: (theme: Theme) => void
  _apply: (theme: Theme) => void
  _isReady: boolean
  _setReady: () => void
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

// Try to save to database (Tauri), fall back to localStorage
async function persistTheme(theme: Theme) {
  // Always save to localStorage as fallback
  try {
    localStorage.setItem('aether-theme', theme)
  } catch {}

  // Try Tauri database persistence
  try {
    const { setSetting } = await import('@/lib/db/tauri')
    await setSetting('theme', theme, 'string')
  } catch {
    // Not running in Tauri — that's fine
  }
}

export const useThemeStore = create<ThemeState>((set) => ({
  theme: 'system',
  resolved: 'dark',
  _isReady: false,

  _setReady: () => set({ _isReady: true }),

  setTheme: (theme: Theme) => {
    const resolved = applyTheme(theme)
    set({ theme, resolved })
    persistTheme(theme)
  },

  _apply: (theme: Theme) => {
    const resolved = applyTheme(theme)
    set({ theme, resolved })
  },
}))

// Listen for system preference changes
if (typeof window !== 'undefined' && typeof window.matchMedia === 'function') {
  try {
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
      const { theme, _apply } = useThemeStore.getState()
      if (theme === 'system') _apply('system')
    })
  } catch {
    // matchMedia not available
  }
}

// Initialize theme — tries DB first, then localStorage, then system
export async function initTheme() {
  let theme: Theme = 'system'

  // Try Tauri database first
  try {
    const { getSetting } = await import('@/lib/db/tauri')
    const setting = await getSetting('theme')
    if (setting && ['light', 'dark', 'system'].includes(setting.value)) {
      theme = setting.value as Theme
    }
  } catch {
    // Not in Tauri — try localStorage
    try {
      const stored = localStorage.getItem('aether-theme')
      if (stored === 'light' || stored === 'dark' || stored === 'system') {
        theme = stored
      }
    } catch {}
  }

  useThemeStore.getState()._apply(theme)
  useThemeStore.getState()._setReady()
}
