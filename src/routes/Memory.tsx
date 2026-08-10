import { MemoryView } from '@/components/memory/MemoryView'
import { useSpaces } from '@/hooks/useSpaces'

export function Memory() {
  const { spaces } = useSpaces()
  return <MemoryView spaces={spaces.filter((space) => !space.archived_at)} />
}
