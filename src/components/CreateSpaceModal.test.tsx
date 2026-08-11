import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'

vi.mock('@/hooks/useSpaces', () => ({
  useSpaces: () => ({ create: vi.fn() }),
}))

import { CreateSpaceModal } from '@/components/CreateSpaceModal'

describe('CreateSpaceModal accessibility', () => {
  it('provides dialog semantics, labelled fields, named controls, and Escape close', async () => {
    const onClose = vi.fn()
    render(<CreateSpaceModal onClose={onClose} />)

    expect(screen.getByRole('dialog', { name: 'New Space' })).toHaveAttribute(
      'aria-modal',
      'true',
    )
    expect(
      screen.getByRole('button', { name: 'Close new Space dialog' }),
    ).toBeInTheDocument()

    await userEvent.click(
      screen.getByRole('button', {
        name: /Blank Space Start with an empty Space/,
      }),
    )
    expect(screen.getByRole('textbox', { name: 'Name *' })).toBeInTheDocument()
    expect(screen.getByRole('textbox', { name: 'Description' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Previous step' })).toBeInTheDocument()

    await userEvent.keyboard('{Escape}')
    expect(onClose).toHaveBeenCalledOnce()
  })
})
