import { graphql } from 'gql.tada'

export const playerByIdQuery = graphql(`
  query PlayerById($playerId: ID!) {
    playerById(playerId: $playerId) {
      id
      name
      avatarFilename
      eloRating
      currentTournamentElo
      trackStats {
        trackName
        averagePosition
        racesPlayed
      }
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
