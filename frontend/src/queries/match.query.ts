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
          eloRating
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
          eloRating
        }
        results {
          player {
            id
            name
            eloRating
          }
          position
          tournamentEloChange
        }
      }
      playerResults {
        player {
          id
          name
          eloRating
        }
        position
        eloChange
        tournamentEloChange
        teammateContribution
      }
    }
  }
`)
