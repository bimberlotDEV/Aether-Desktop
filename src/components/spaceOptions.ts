export const SPACE_ACCENTS = [
  { id: 'indigo', value: '#6366f1', label: 'Indigo' },
  { id: 'blue', value: '#3b82f6', label: 'Blue' },
  { id: 'emerald', value: '#10b981', label: 'Emerald' },
  { id: 'amber', value: '#f59e0b', label: 'Amber' },
  { id: 'rose', value: '#f43f5e', label: 'Rose' },
  { id: 'violet', value: '#8b5cf6', label: 'Violet' },
  { id: 'teal', value: '#14b8a6', label: 'Teal' },
  { id: 'slate', value: '#64748b', label: 'Slate' },
]

export const SPACE_MODULES = [
  { id: 'notes', label: 'Notes', description: 'Write, organize, and search notes' },
  {
    id: 'tasks',
    label: 'Tasks',
    description: 'Track to-dos with priorities and due dates',
  },
  { id: 'files', label: 'Files', description: 'Import and manage documents' },
  { id: 'ai', label: 'AI', description: 'AI-powered assistance and summarization' },
  { id: 'memory', label: 'Memory', description: 'Store explicit durable context' },
  { id: 'activity', label: 'Activity', description: 'Timeline of meaningful actions' },
] as const
