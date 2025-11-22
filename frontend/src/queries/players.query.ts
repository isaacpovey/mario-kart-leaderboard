import { graphql } from 'gql.tada'

export const playersQuery = graphql(`
  query Players {
    players {
      id
      name
      currentTournamentElo
      avatarFilename
    }
  }
`)
