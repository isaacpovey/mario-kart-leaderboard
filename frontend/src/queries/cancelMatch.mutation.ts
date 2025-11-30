import { graphql } from 'gql.tada'

export const cancelMatchMutation = graphql(`
  mutation CancelMatch($matchId: ID!) {
    cancelMatch(matchId: $matchId)
  }
`)
