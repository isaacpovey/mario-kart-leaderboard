import { graphql } from 'gql.tada'

export const matchQuery = graphql(`
  query Match($matchId: ID!) {
    matchById(matchId: $matchId) {
      id
      tournamentId
      time
      numOfRounds
      completed
      teams {
        id
        name
        score
        players {
          id
          name
          avatarFilename
          currentTournamentElo
        }
      }
      rounds {
        roundNumber
        track {
          id
          name
        }
        completed
        players {
          id
          name
          avatarFilename
          teamId
        }
        results {
          player {
            id
            name
            avatarFilename
          }
          position
          tournamentEloChange
        }
      }
      playerResults {
        player {
          id
          name
          avatarFilename
          currentTournamentElo
        }
        position
        eloChange
        tournamentEloChange
        tournamentEloFromRaces
        tournamentEloFromContributions
        teammateContribution
      }
    }
  }
`)
