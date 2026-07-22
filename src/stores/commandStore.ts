import { create } from 'zustand'

export interface Command {
  id: string
  label: string
  description?: string
  icon?: string
  keywords?: string[]
  action: () => void
}

interface CommandState {
  isOpen: boolean
  query: string
  commands: Command[]
  open: () => void
  close: () => void
  toggle: () => void
  setQuery: (q: string) => void
  registerCommands: (cmds: Command[]) => void
}

export const useCommandStore = create<CommandState>((set) => ({
  isOpen: false,
  query: '',
  commands: [],

  open: () => set({ isOpen: true, query: '' }),
  close: () => set({ isOpen: false, query: '' }),
  toggle: () => set((s) => ({ isOpen: !s.isOpen, query: '' })),

  setQuery: (query: string) => set({ query }),

  registerCommands: (commands: Command[]) => set({ commands }),
}))
