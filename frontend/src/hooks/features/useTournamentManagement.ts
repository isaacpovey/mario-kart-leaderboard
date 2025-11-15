import { useCallback } from 'react'
import { createTournamentMutation } from '@/queries/createTournament.mutation.ts'
import { useMutationWithRefresh } from '../patterns/useMutationWithRefresh'

type CreateTournamentInput = {
  startDate?: string | null
  endDate?: string | null
}

export const useTournamentManagement = () => {
  const { execute: createTournamentAction, isLoading: isCreating, error: createError } = useMutationWithRefresh(createTournamentMutation)

  const createTournament = useCallback(
    async (input: CreateTournamentInput) => {
      const result = await createTournamentAction(input)
      return result.data?.createTournament || null
    },
    [createTournamentAction]
  )

  return {
    createTournament,
    isCreating,
    createError,
  }
}
