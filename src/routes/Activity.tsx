import { Activity as ActivityIcon } from 'lucide-react'
import { EmptyState, Page, PageHeader } from '@/components/ui/AetherUI'

export function Activity() {
  return (
    <Page>
      <PageHeader
        eyebrow="Timeline"
        icon={ActivityIcon}
        title="Activity"
        description="A quiet, local history of meaningful changes across your workspace."
      />
      <EmptyState
        icon={ActivityIcon}
        eyebrow="Nothing to review"
        title="Your timeline is quiet"
        description="As you create and organize things, meaningful actions will appear here without analytics, noise, or surveillance."
      />
    </Page>
  )
}
