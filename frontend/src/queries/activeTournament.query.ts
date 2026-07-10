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
        allTimeEloRating
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
