import { useCallback } from 'react'
import { useMutation } from 'urql'
import { checkInPlayerMutation } from '@/queries/checkInPlayer.mutation'
import { checkOutPlayerMutation } from '@/queries/checkOutPlayer.mutation'

export const useLobby = (checkedInPlayerIds: string[]) => {
  const [checkInResult, executeCheckIn] = useMutation(checkInPlayerMutation)
  const [checkOutResult, executeCheckOut] = useMutation(checkOutPlayerMutation)

  const checkIn = useCallback((playerId: string) => executeCheckIn({ playerId }), [executeCheckIn])

  const checkOut = useCallback((playerId: string) => executeCheckOut({ playerId }), [executeCheckOut])

  const toggle = useCallback(
    (playerId: string) => {
      if (checkedInPlayerIds.includes(playerId)) {
        return executeCheckOut({ playerId })
      }
      return executeCheckIn({ playerId })
    },
    [checkedInPlayerIds, executeCheckIn, executeCheckOut]
  )

  const isLoading = checkInResult.fetching || checkOutResult.fetching
  const error = checkInResult.error ?? checkOutResult.error

  return { checkIn, checkOut, error, isLoading, toggle }
}
