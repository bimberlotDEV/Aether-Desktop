import { Settings as SettingsIcon, Sun, Moon, Monitor } from 'lucide-react'
import { useThemeStore } from '@/stores/themeStore'
import type { Theme } from '@/stores/themeStore'
import { AiSettings } from '@/components/ai/AiSettings'
import { NativeSettings } from '@/components/NativeSettings'
import { BackupSettings } from '@/components/BackupSettings'
import { Page, PageHeader, SectionLabel, Surface } from '@/components/ui/AetherUI'

const themeOptions: { value: Theme; label: string; icon: typeof Sun }[] = [
  { value: 'light', label: 'Light', icon: Sun },
  { value: 'dark', label: 'Dark', icon: Moon },
  { value: 'system', label: 'System', icon: Monitor },
]

export function Settings() {
  const { theme, setTheme } = useThemeStore()

  return (
    <Page width="compact">
      <PageHeader
        eyebrow="Aether control center"
        icon={SettingsIcon}
        title="Settings"
        description="Tune your workspace, private intelligence, and Windows integration."
      />
      <div className="space-y-7">
        {/* Appearance */}
        <section>
          <SectionLabel>Appearance</SectionLabel>
          <div className="grid grid-cols-3 gap-2">
            {themeOptions.map((opt) => {
              const Icon = opt.icon
              const isActive = theme === opt.value
              return (
                <button
                  key={opt.value}
                  type="button"
                  aria-pressed={isActive}
                  onClick={() => setTheme(opt.value)}
                  className="aether-theme-option focus-ring"
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
          <SectionLabel>About</SectionLabel>
          <Surface className="p-5 space-y-3">
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
          </Surface>
        </section>
      </div>
    </Page>
  )
}
