import { graphql } from 'gql.tada'

export const recordRoundResultsMutation = graphql(`
  mutation RecordRoundResults($matchId: ID!, $roundNumber: Int!, $results: [PlayerResultInput!]!) {
    recordRoundResults(matchId: $matchId, roundNumber: $roundNumber, results: $results) {
      id
      completed
      rounds {
        roundNumber
        completed
      }
    }
  }
`)
