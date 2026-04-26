import { useCallback, useEffect, useRef } from 'react'
import { useMutation, useSubscription } from 'urql'
import { assertSlotAssignmentMutation } from '../queries/assertSlotAssignment.mutation'
import { slotAssignmentsUpdatedSubscription } from '../subscriptions/slotAssignmentsUpdated.subscription'

type UseSlotAssignmentSyncArgs = {
  matchId: string | null
  roundNumber: number | null
  onAssignment: (slotNumber: number, playerId: string | null) => void
}

type UseSlotAssignmentSyncResult = {
  publish: (slotNumber: number, playerId: string | null) => Promise<{ ok: boolean }>
}

export const useSlotAssignmentSync = ({ matchId, roundNumber, onAssignment }: UseSlotAssignmentSyncArgs): UseSlotAssignmentSyncResult => {
  const clientIdRef = useRef<string>('')
  if (clientIdRef.current === '') {
    clientIdRef.current = crypto.randomUUID()
  }

  const onAssignmentRef = useRef(onAssignment)
  useEffect(() => {
    onAssignmentRef.current = onAssignment
  }, [onAssignment])

  const canSync = matchId !== null && roundNumber !== null

  // Process every event inside the urql subscription handler so back-to-back
  // messages aren't coalesced by React's render batching (only the latest
  // `subscription.data` would be visible to a `useEffect` watcher under
  // contention).
  useSubscription(
    {
      query: slotAssignmentsUpdatedSubscription,
      variables: { matchId: matchId ?? '', roundNumber: roundNumber ?? 0 },
      pause: !canSync,
    },
    (_acc, data) => {
      const event = data.slotAssignmentsUpdated
      if (event && event.sourceClientId !== clientIdRef.current) {
        onAssignmentRef.current(event.slotNumber, event.playerId ?? null)
      }
      return data
    }
  )

  const [, executeMutation] = useMutation(assertSlotAssignmentMutation)

  const publish = useCallback(
    async (slotNumber: number, playerId: string | null) => {
      if (!canSync || matchId === null || roundNumber === null) return { ok: false }
      const result = await executeMutation({
        matchId,
        roundNumber,
        slotNumber,
        playerId,
        clientId: clientIdRef.current,
      })
      if (result.error) {
        console.error('Failed to sync slot assignment', { matchId, roundNumber, slotNumber, playerId, error: result.error.message })
        return { ok: false }
      }
      return { ok: true }
    },
    [executeMutation, canSync, matchId, roundNumber]
  )

  return { publish }
}
