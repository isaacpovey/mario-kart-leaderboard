import { graphql } from 'gql.tada'

export const completeTournamentMutation = graphql(`
  mutation CompleteTournament($tournamentId: ID!) {
    completeTournament(tournamentId: $tournamentId) {
      id
      startDate
      endDate
      winnerId
    }
  }
`)
