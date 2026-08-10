import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it } from 'vitest'
import { CommandPalette } from '@/components/CommandPalette'
import { useCommandStore } from '@/stores/commandStore'

describe('CommandPalette navigation', () => {
  beforeEach(() => {
    window.history.replaceState({}, '', '/')
    useCommandStore.setState({ isOpen: true, query: 'Go to Tasks' })
  })

  it('navigates BrowserRouter routes without a stale hash URL', async () => {
    const user = userEvent.setup()
    render(<CommandPalette />)

    await user.click(screen.getByRole('button', { name: /Go to Tasks/ }))

    expect(window.location.pathname).toBe('/tasks')
    expect(window.location.hash).toBe('')
    expect(useCommandStore.getState().isOpen).toBe(false)
  })
})
