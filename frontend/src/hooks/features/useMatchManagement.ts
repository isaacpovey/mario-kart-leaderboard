import { useCallback } from 'react'
import { cancelMatchMutation } from '@/queries/cancelMatch.mutation.ts'
import { createMatchWithRoundsMutation } from '@/queries/createMatchWithRounds.mutation.ts'
import { recordRoundResultsMutation } from '@/queries/recordRoundResults.mutation.ts'
import { swapRoundPlayerMutation } from '@/queries/swapRoundPlayer.mutation.ts'
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

type SwapRoundPlayerInput = {
  matchId: string
  roundNumber: number
  currentPlayerId: string
  newPlayerId: string
}

export const useMatchManagement = () => {
  const { execute: createMatch, isLoading: isCreatingMatch, error: createMatchError } = useMutationWithRefresh(createMatchWithRoundsMutation)

  const { execute: recordRoundResults, isLoading: isRecordingResults, error: recordResultsError } = useMutationWithRefresh(recordRoundResultsMutation)

  const { execute: executeCancelMatch, isLoading: isCancellingMatch, error: cancelMatchError } = useMutationWithRefresh(cancelMatchMutation)

  const { execute: executeSwapRoundPlayer, isLoading: isSwappingPlayer, error: swapPlayerError } = useMutationWithRefresh(swapRoundPlayerMutation)

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

  const cancelMatch = useCallback(
    async (matchId: string) => {
      const result = await executeCancelMatch({ matchId })
      return result.data?.cancelMatch ?? false
    },
    [executeCancelMatch]
  )

  const swapRoundPlayer = useCallback(
    async (input: SwapRoundPlayerInput) => {
      const result = await executeSwapRoundPlayer(input)
      return result.data?.swapRoundPlayer ?? null
    },
    [executeSwapRoundPlayer]
  )

  return {
    createMatchWithRounds,
    isCreatingMatch,
    createMatchError,
    recordResults,
    isRecordingResults,
    recordResultsError,
    cancelMatch,
    isCancellingMatch,
    cancelMatchError,
    swapRoundPlayer,
    isSwappingPlayer,
    swapPlayerError,
  }
}
