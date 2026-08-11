import { Settings as SettingsIcon, Sun, Moon, Monitor } from 'lucide-react'
import { useThemeStore } from '@/stores/themeStore'
import type { Theme } from '@/stores/themeStore'
import { AiSettings } from '@/components/ai/AiSettings'
import { NativeSettings } from '@/components/NativeSettings'
import { BackupSettings } from '@/components/BackupSettings'

const themeOptions: { value: Theme; label: string; icon: typeof Sun }[] = [
  { value: 'light', label: 'Light', icon: Sun },
  { value: 'dark', label: 'Dark', icon: Moon },
  { value: 'system', label: 'System', icon: Monitor },
]

export function Settings() {
  const { theme, setTheme } = useThemeStore()

  return (
    <div className="flex-1 flex flex-col min-h-0 overflow-y-auto">
      {/* Header */}
      <div
        className="px-8 py-6 shrink-0"
        style={{ borderBottom: `1px solid var(--color-border)` }}
      >
        <div className="flex items-center gap-3">
          <SettingsIcon
            size={20}
            strokeWidth={1.75}
            style={{ color: 'var(--color-text-secondary)' }}
          />
          <h1
            className="text-xl font-semibold tracking-tight"
            style={{ color: 'var(--color-text-primary)' }}
          >
            Settings
          </h1>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 px-8 py-6 space-y-8 max-w-[560px]">
        {/* Appearance */}
        <section>
          <h2
            className="text-sm font-semibold mb-4"
            style={{ color: 'var(--color-text-primary)' }}
          >
            Appearance
          </h2>

          <div className="space-y-1">
            {themeOptions.map((opt) => {
              const Icon = opt.icon
              const isActive = theme === opt.value
              return (
                <button
                  key={opt.value}
                  type="button"
                  aria-pressed={isActive}
                  onClick={() => setTheme(opt.value)}
                  className="flex items-center gap-3 w-full px-4 py-3 rounded-lg text-left transition-colors duration-100"
                  style={{
                    backgroundColor: isActive
                      ? 'var(--color-accent-muted)'
                      : 'transparent',
                    color: isActive ? 'var(--color-accent)' : 'var(--color-text-primary)',
                  }}
                >
                  <Icon size={17} strokeWidth={1.75} />
                  <span className="text-sm font-medium flex-1">{opt.label}</span>
                  {isActive && (
                    <div
                      className="w-2 h-2 rounded-full"
                      style={{ backgroundColor: 'var(--color-accent)' }}
                    />
                  )}
                </button>
              )
            })}
          </div>
        </section>

        <AiSettings />

        <NativeSettings />

        <BackupSettings />

        {/* About */}
        <section>
          <h2
            className="text-sm font-semibold mb-4"
            style={{ color: 'var(--color-text-primary)' }}
          >
            About
          </h2>
          <div
            className="rounded-xl p-5 space-y-2"
            style={{
              backgroundColor: 'var(--color-bg-secondary)',
              border: `1px solid var(--color-border)`,
            }}
          >
            <div className="flex justify-between items-center">
              <span className="text-sm" style={{ color: 'var(--color-text-secondary)' }}>
                Version
              </span>
              <span
                className="text-sm font-medium"
                style={{ color: 'var(--color-text-primary)' }}
              >
                Alpha 0.3.1
              </span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-sm" style={{ color: 'var(--color-text-secondary)' }}>
                Storage
              </span>
              <span
                className="text-sm font-medium"
                style={{ color: 'var(--color-text-primary)' }}
              >
                Local
              </span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-sm" style={{ color: 'var(--color-text-secondary)' }}>
                Data directory
              </span>
              <span
                className="text-sm font-medium text-right max-w-[200px] truncate"
                style={{ color: 'var(--color-text-tertiary)' }}
              >
                %APPDATA%/Aether
              </span>
            </div>
          </div>
        </section>
      </div>
    </div>
  )
}
