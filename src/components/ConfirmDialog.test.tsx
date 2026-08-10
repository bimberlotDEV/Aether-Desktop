import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { ConfirmDialog } from '@/components/ConfirmDialog'

describe('ConfirmDialog', () => {
  it('exposes dialog semantics and supports keyboard cancellation', async () => {
    const onCancel = vi.fn()
    render(
      <ConfirmDialog
        title="Delete note?"
        message="This cannot be undone."
        danger
        onConfirm={vi.fn()}
        onCancel={onCancel}
      />,
    )

    expect(screen.getByRole('alertdialog', { name: 'Delete note?' })).toHaveAttribute(
      'aria-modal',
      'true',
    )
    await userEvent.keyboard('{Escape}')
    expect(onCancel).toHaveBeenCalledOnce()
  })
})
