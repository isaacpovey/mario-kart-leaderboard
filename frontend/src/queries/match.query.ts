import { graphql } from 'gql.tada'

export const matchQuery = graphql(`
  query Match($matchId: ID!) {
    matchById(matchId: $matchId) {
      id
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
          currentTournamentElo
          avatarFilename
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
          currentTournamentElo
          avatarFilename
        }
        results {
          player {
            id
            name
            currentTournamentElo
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
          currentTournamentElo
          avatarFilename
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
