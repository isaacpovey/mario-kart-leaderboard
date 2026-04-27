import { graphql } from 'gql.tada'

export const checkInPlayerMutation = graphql(`
  mutation CheckInPlayer($playerId: ID!) {
    checkInPlayer(playerId: $playerId) {
      id
      name
      avatarFilename
      currentTournamentElo
    }
  }
`)
