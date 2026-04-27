import { graphql } from 'gql.tada'

export const assertSlotAssignmentMutation = graphql(`
  mutation AssertSlotAssignment(
    $matchId: ID!
    $roundNumber: Int!
    $slotNumber: Int!
    $playerId: ID
    $clientId: String!
  ) {
    assertSlotAssignment(
      matchId: $matchId
      roundNumber: $roundNumber
      slotNumber: $slotNumber
      playerId: $playerId
      clientId: $clientId
    )
  }
`)
