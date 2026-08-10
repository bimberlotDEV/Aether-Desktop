import { useCallback, useEffect, useMemo, useState } from 'react'
import type { Task, TaskFilter, TaskInput } from '@/lib/db/types'
import * as db from '@/lib/db/tauri'

const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
const taskChangeListeners = new Set<() => void>()
let mockTasks: Task[] = []
let mockIdCounter = 0

function notifyTasksChanged() {
  taskChangeListeners.forEach((listener) => listener())
}

function mockDescendants(id: string): Set<string> {
  const ids = new Set([id])
  let changed = true
  while (changed) {
    changed = false
    for (const task of mockTasks) {
      if (task.parent_task_id && ids.has(task.parent_task_id) && !ids.has(task.id)) {
        ids.add(task.id)
        changed = true
      }
    }
  }
  return ids
}

function useTaskChangeSubscription(load: () => Promise<void>) {
  useEffect(() => {
    const listener = () => void load()
    taskChangeListeners.add(listener)
    return () => {
      taskChangeListeners.delete(listener)
    }
  }, [load])
}

export function taskToInput(task: Task, changes: Partial<TaskInput> = {}): TaskInput {
  return {
    spaceId: task.space_id,
    parentTaskId: task.parent_task_id,
    title: task.title,
    description: task.description,
    status: task.status,
    priority: task.priority,
    dueDate: task.due_date,
    tags: task.tags,
    ...changes,
  }
}

function createMockTask(input: TaskInput): Task {
  const now = new Date().toISOString()
  mockIdCounter += 1
  return {
    id: `mock-task-${mockIdCounter}`,
    space_id: input.spaceId,
    parent_task_id: input.parentTaskId,
    title: input.title.trim(),
    description: input.description,
    status: input.status,
    priority: input.priority,
    due_date: input.dueDate,
    tags: input.tags,
    completed_at: input.status === 'done' ? now : null,
    archived_at: null,
    created_at: now,
    updated_at: now,
  }
}

function filterMockTasks(filter: TaskFilter): Task[] {
  const search = filter.search?.trim().toLocaleLowerCase()
  return mockTasks
    .filter((task) => (filter.includeArchived ? true : !task.archived_at))
    .filter((task) => (filter.spaceId ? task.space_id === filter.spaceId : true))
    .filter((task) => (filter.unassignedOnly ? task.space_id === null : true))
    .filter((task) => (filter.status ? task.status === filter.status : true))
    .filter((task) => (filter.priority ? task.priority === filter.priority : true))
    .filter((task) =>
      search
        ? `${task.title} ${task.description} ${task.tags.join(' ')}`
            .toLocaleLowerCase()
            .includes(search)
        : true,
    )
    .sort((a, b) => b.updated_at.localeCompare(a.updated_at))
    .slice(0, filter.limit ?? 500)
}

export function useTasks(filter: TaskFilter = {}) {
  const filterKey = JSON.stringify(filter)
  const stableFilter = useMemo<TaskFilter>(() => JSON.parse(filterKey), [filterKey])
  const [tasks, setTasks] = useState<Task[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      setTasks(isTauri ? await db.listTasks(stableFilter) : filterMockTasks(stableFilter))
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Failed to load Tasks')
    } finally {
      setLoading(false)
    }
  }, [stableFilter])

  useEffect(() => {
    void load()
  }, [load])
  useTaskChangeSubscription(load)

  const mutate = useCallback(async <T>(operation: () => Promise<T>): Promise<T> => {
    setError(null)
    try {
      const result = await operation()
      notifyTasksChanged()
      return result
    } catch (cause) {
      const message = cause instanceof Error ? cause.message : 'Task update failed'
      setError(message)
      throw cause
    }
  }, [])

  const create = useCallback(
    (input: TaskInput) =>
      mutate(async () => {
        if (isTauri) return db.createTask(input)
        const task = createMockTask(input)
        mockTasks = [task, ...mockTasks]
        return task
      }),
    [mutate],
  )

  const update = useCallback(
    (task: Task, changes: Partial<TaskInput>) =>
      mutate(async () => {
        const input = taskToInput(task, changes)
        if (isTauri) {
          const updated = await db.updateTask(task.id, input)
          if (!updated) throw new Error('This Task no longer exists.')
          return updated
        }
        const now = new Date().toISOString()
        let updated: Task | null = null
        const descendants = mockDescendants(task.id)
        mockTasks = mockTasks.map((candidate) => {
          if (candidate.id !== task.id) return candidate
          updated = {
            ...candidate,
            space_id: input.spaceId,
            parent_task_id: input.parentTaskId,
            title: input.title.trim(),
            description: input.description,
            status: input.status,
            priority: input.priority,
            due_date: input.dueDate,
            tags: input.tags,
            completed_at:
              input.status === 'done' ? (candidate.completed_at ?? now) : null,
            updated_at: now,
          }
          return updated
        })
        if (input.spaceId !== task.space_id) {
          mockTasks = mockTasks.map((candidate) =>
            candidate.id !== task.id && descendants.has(candidate.id)
              ? { ...candidate, space_id: input.spaceId, updated_at: now }
              : candidate,
          )
        }
        return updated
      }),
    [mutate],
  )

  const archive = useCallback(
    (id: string) =>
      mutate(async () => {
        if (isTauri) {
          const archived = await db.archiveTask(id)
          if (!archived) throw new Error('This Task no longer exists.')
          return archived
        }
        const now = new Date().toISOString()
        const descendants = mockDescendants(id)
        mockTasks = mockTasks.map((task) =>
          descendants.has(task.id)
            ? { ...task, archived_at: now, updated_at: now }
            : task,
        )
        return true
      }),
    [mutate],
  )

  const restore = useCallback(
    (id: string) =>
      mutate(async () => {
        if (isTauri) {
          const restored = await db.restoreTask(id)
          if (!restored) throw new Error('This Task no longer exists.')
          return restored
        }
        const now = new Date().toISOString()
        const descendants = mockDescendants(id)
        mockTasks = mockTasks.map((task) =>
          descendants.has(task.id)
            ? { ...task, archived_at: null, updated_at: now }
            : task,
        )
        return true
      }),
    [mutate],
  )

  const remove = useCallback(
    (id: string) =>
      mutate(async () => {
        if (isTauri) {
          const removed = await db.deleteTask(id)
          if (!removed) throw new Error('This Task no longer exists.')
          return removed
        }
        const removed = mockDescendants(id)
        mockTasks = mockTasks.filter((task) => !removed.has(task.id))
        return true
      }),
    [mutate],
  )

  const toggleComplete = useCallback(
    (task: Task) => update(task, { status: task.status === 'done' ? 'inbox' : 'done' }),
    [update],
  )

  return {
    tasks,
    loading,
    error,
    load,
    create,
    update,
    toggleComplete,
    archive,
    restore,
    remove,
    isTauri,
  }
}

function formatLocalDate(date: Date): string {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

export function useTaskAttention(days = 7, limit = 12) {
  const [tasks, setTasks] = useState<Task[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(async () => {
    setLoading(true)
    setError(null)
    const now = new Date()
    const horizon = new Date(now)
    horizon.setDate(horizon.getDate() + days)
    const today = formatLocalDate(now)
    const horizonDate = formatLocalDate(horizon)
    try {
      const result = isTauri
        ? await db.listTaskAttention(today, horizonDate, limit)
        : filterMockTasks({ limit: 500 })
            .filter(
              (task) =>
                task.status !== 'done' && !!task.due_date && task.due_date <= horizonDate,
            )
            .slice(0, limit)
      setTasks(result)
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Failed to load Task attention')
    } finally {
      setLoading(false)
    }
  }, [days, limit])

  useEffect(() => {
    void load()
  }, [load])
  useTaskChangeSubscription(load)

  const toggleComplete = useCallback(async (task: Task) => {
    setError(null)
    const input = taskToInput(task, {
      status: task.status === 'done' ? 'inbox' : 'done',
    })
    try {
      if (isTauri) {
        const updated = await db.updateTask(task.id, input)
        if (!updated) throw new Error('This Task no longer exists.')
      } else {
        const now = new Date().toISOString()
        mockTasks = mockTasks.map((candidate) =>
          candidate.id === task.id
            ? {
                ...candidate,
                status: input.status,
                completed_at: input.status === 'done' ? now : null,
                updated_at: now,
              }
            : candidate,
        )
      }
      notifyTasksChanged()
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Failed to update Task')
      throw cause
    }
  }, [])

  return { tasks, loading, error, load, toggleComplete }
}
