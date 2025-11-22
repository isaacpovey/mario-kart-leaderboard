import { graphql } from 'gql.tada'

export const activeTournamentQuery = graphql(`
  query ActiveTournament {
    activeTournament {
      id
      startDate
      endDate
      winnerId
      leaderboard {
        playerId
        playerName
        totalScore
        eloRating
        allTimeElo
        avatarFilename
      }
      matches {
        id
        time
        completed
      }
    }
  }
`)
