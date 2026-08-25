import { fireEvent, render, screen } from '@testing-library/react'
import { Layers } from 'lucide-react'
import { Button, EmptyState, PageHeader } from '@/components/ui/AetherUI'

describe('Aether UI primitives', () => {
  it('keeps headings, actions, and empty-state controls semantic', () => {
    const onHeaderAction = vi.fn()
    const onEmptyAction = vi.fn()

    render(
      <>
        <PageHeader
          eyebrow="Structure"
          icon={Layers}
          title="Spaces"
          description="Focused contexts"
          actions={<Button onClick={onHeaderAction}>New Space</Button>}
        />
        <EmptyState
          icon={Layers}
          title="Create a place"
          description="Keep related work together."
          action={{ label: 'Create Space', onClick: onEmptyAction }}
        />
      </>,
    )

    expect(screen.getByRole('heading', { level: 1, name: 'Spaces' })).toBeVisible()
    expect(
      screen.getByRole('heading', { level: 2, name: 'Create a place' }),
    ).toBeVisible()

    fireEvent.click(screen.getByRole('button', { name: 'New Space' }))
    fireEvent.click(screen.getByRole('button', { name: 'Create Space' }))

    expect(onHeaderAction).toHaveBeenCalledOnce()
    expect(onEmptyAction).toHaveBeenCalledOnce()
  })
})
