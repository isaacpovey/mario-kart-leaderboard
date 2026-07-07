import { useAtom } from 'jotai'
import { useCallback, useMemo } from 'react'
import { mePlayerIdByGroupAtom } from '@/store/me'

export const useMe = (groupId: string | null | undefined) => {
  const [map, setMap] = useAtom(mePlayerIdByGroupAtom)

  const playerId = useMemo(() => (groupId ? (map[groupId] ?? null) : null), [groupId, map])

  const setMe = useCallback(
    (nextPlayerId: string) => {
      if (!groupId) {
        return
      }
      setMap((prev) => ({ ...prev, [groupId]: nextPlayerId }))
    },
    [groupId, setMap]
  )

  const clearMe = useCallback(() => {
    if (!groupId) {
      return
    }
    setMap((prev) => {
      const { [groupId]: _removed, ...next } = prev
      return next
    })
  }, [groupId, setMap])

  return { clearMe, playerId, setMe }
}
