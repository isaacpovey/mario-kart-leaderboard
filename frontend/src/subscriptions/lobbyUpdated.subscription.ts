import { graphql } from 'gql.tada'

export const lobbyUpdatedSubscription = graphql(`
  subscription LobbyUpdated {
    lobbyUpdated {
      id
      name
      avatarFilename
      currentTournamentElo
    }
  }
`)
