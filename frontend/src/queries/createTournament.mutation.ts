import { graphql } from 'gql.tada'

export const createTournamentMutation = graphql(`
  mutation CreateTournament($startDate: String, $endDate: String) {
    createTournament(startDate: $startDate, endDate: $endDate) {
      id
      startDate
      endDate
      winnerId
    }
  }
`)
