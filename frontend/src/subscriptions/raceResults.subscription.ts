import { graphql } from 'gql.tada'

export const raceResultsUpdatedSubscription = graphql(`
  subscription RaceResultsUpdated($tournamentId: ID!) {
    raceResultsUpdated(tournamentId: $tournamentId) {
      matchId
      tournamentId
      roundNumber
      roundCompleted
      matchCompleted
      raceResults {
        player {
          id
          name
          avatarFilename
        }
        position
        tournamentEloChange
      }
      playerAggregates {
        player {
          id
          name
          avatarFilename
        }
        position
        eloChange
        tournamentEloChange
        tournamentEloFromRaces
        tournamentEloFromContributions
      }
      leaderboard {
        playerId
        playerName
        allTimeEloRating
        allTimeElo
        avatarFilename
      }
      teams {
        id
        name
        score
      }
    }
  }
`)
