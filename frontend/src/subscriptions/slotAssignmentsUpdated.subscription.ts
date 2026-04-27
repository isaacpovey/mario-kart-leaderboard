import { graphql } from 'gql.tada'

export const slotAssignmentsUpdatedSubscription = graphql(`
  subscription SlotAssignmentsUpdated($matchId: ID!, $roundNumber: Int!) {
    slotAssignmentsUpdated(matchId: $matchId, roundNumber: $roundNumber) {
      slotNumber
      playerId
      sourceClientId
    }
  }
`)
