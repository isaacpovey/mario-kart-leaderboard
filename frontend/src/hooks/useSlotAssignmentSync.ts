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
  publish: (slotNumber: number, playerId: string | null) => void
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

  const [subscription] = useSubscription({
    query: slotAssignmentsUpdatedSubscription,
    variables: { matchId: matchId ?? '', roundNumber: roundNumber ?? 0 },
    pause: !canSync,
  })

  useEffect(() => {
    const event = subscription.data?.slotAssignmentsUpdated
    if (!event) return
    if (event.sourceClientId === clientIdRef.current) return
    onAssignmentRef.current(event.slotNumber, event.playerId ?? null)
  }, [subscription.data])

  const [, executeMutation] = useMutation(assertSlotAssignmentMutation)

  const publish = useCallback(
    (slotNumber: number, playerId: string | null) => {
      if (!canSync || matchId === null || roundNumber === null) return
      void executeMutation({
        matchId,
        roundNumber,
        slotNumber,
        playerId,
        clientId: clientIdRef.current,
      })
    },
    [executeMutation, canSync, matchId, roundNumber]
  )

  return { publish }
}
