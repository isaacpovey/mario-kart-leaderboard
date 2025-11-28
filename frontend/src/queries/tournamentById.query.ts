import { graphql } from '../lib/graphql'

export const tournamentByIdQuery = graphql(`
  query TournamentById($id: ID!) {
    tournamentById(id: $id) {
      id
      startDate
      endDate
      winnerId
      leaderboard {
        playerId
        playerName
        allTimeEloRating
        allTimeElo
        avatarFilename
      }
      stats {
        id
        statType
        playerId
        value
        extraData
      }
      playerEloHistory {
        playerId
        playerName
        dataPoints {
          timestamp
          elo
        }
      }
    }
  }
`)
