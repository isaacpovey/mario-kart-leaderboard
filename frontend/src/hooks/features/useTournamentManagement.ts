import { useCallback } from 'react'
import { completeTournamentMutation } from '@/queries/completeTournament.mutation.ts'
import { createTournamentMutation } from '@/queries/createTournament.mutation.ts'
import { useMutationWithRefresh } from '../patterns/useMutationWithRefresh'

type CreateTournamentInput = {
  startDate?: string | null
  endDate?: string | null
}

export const useTournamentManagement = () => {
  const { execute: createTournamentAction, isLoading: isCreating, error: createError } = useMutationWithRefresh(createTournamentMutation)
  const { execute: completeTournamentAction, isLoading: isCompleting, error: completeError } = useMutationWithRefresh(completeTournamentMutation)

  const createTournament = useCallback(
    async (input: CreateTournamentInput) => {
      const result = await createTournamentAction(input)
      return result.data?.createTournament ?? null
    },
    [createTournamentAction]
  )

  const completeTournament = useCallback(
    async (tournamentId: string) => {
      const result = await completeTournamentAction({ tournamentId })
      return result.data?.completeTournament ?? null
    },
    [completeTournamentAction]
  )

  return {
    createTournament,
    isCreating,
    createError,
    completeTournament,
    isCompleting,
    completeError,
  }
}
