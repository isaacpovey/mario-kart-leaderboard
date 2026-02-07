import { graphql } from 'gql.tada'

export const playerByIdQuery = graphql(`
  query PlayerById($playerId: ID!) {
    playerById(playerId: $playerId) {
      id
      name
      avatarFilename
      eloRating
      currentTournamentElo
      matchHistory {
        matchId
        matchTime
        position
        eloChange
        tournamentEloChange
      }
    }
  }
`)
