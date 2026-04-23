import { graphql } from 'gql.tada'

export const lobbyQuery = graphql(`
  query Lobby {
    currentGroup {
      id
      players {
        id
        name
        avatarFilename
        currentTournamentElo
      }
      lobby {
        id
        name
        avatarFilename
        currentTournamentElo
      }
    }
  }
`)
