import { graphql } from 'gql.tada'

export const checkOutPlayerMutation = graphql(`
  mutation CheckOutPlayer($playerId: ID!) {
    checkOutPlayer(playerId: $playerId) {
      id
      name
      avatarFilename
      currentTournamentElo
    }
  }
`)
