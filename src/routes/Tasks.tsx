import { TaskView } from '@/components/tasks/TaskView'
import { useSpaces } from '@/hooks/useSpaces'

export function Tasks() {
  const { spaces } = useSpaces()
  return (
    <div className="flex-1 min-h-0 overflow-y-auto">
      <TaskView spaces={spaces.filter((space) => !space.archived_at)} />
    </div>
  )
}
