import { useCallback } from 'react'
import { createMatchWithRoundsMutation } from '@/queries/createMatchWithRounds.mutation.ts'
import { recordRoundResultsMutation } from '@/queries/recordRoundResults.mutation.ts'
import { useMutationWithRefresh } from '../patterns/useMutationWithRefresh'

type CreateMatchInput = {
  tournamentId: string
  playerIds: string[]
  numRaces: number
  playersPerRace: number
}

type RecordRoundResultsInput = {
  matchId: string
  roundNumber: number
  results: Array<{ playerId: string; position: number }>
}

export const useMatchManagement = () => {
  const { execute: createMatch, isLoading: isCreatingMatch, error: createMatchError } = useMutationWithRefresh(createMatchWithRoundsMutation)

  const { execute: recordRoundResults, isLoading: isRecordingResults, error: recordResultsError } = useMutationWithRefresh(recordRoundResultsMutation)

  const createMatchWithRounds = useCallback(
    async (input: CreateMatchInput) => {
      const result = await createMatch(input)
      return result.data?.createMatchWithRounds || null
    },
    [createMatch]
  )

  const recordResults = useCallback(
    async (input: RecordRoundResultsInput) => {
      const result = await recordRoundResults(input)
      return result.data?.recordRoundResults || null
    },
    [recordRoundResults]
  )

  return {
    createMatchWithRounds,
    isCreatingMatch,
    createMatchError,
    recordResults,
    isRecordingResults,
    recordResultsError,
  }
}
