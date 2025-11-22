import { graphql } from 'gql.tada'
import { PlayerBasicFragment, PlayerWithTournamentEloFragment } from '../fragments/player.fragments'

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
          ...PlayerBasic
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
          ...PlayerBasic
        }
        results {
          player {
            ...PlayerWithTournamentElo
          }
          position
          tournamentEloChange
        }
      }
      playerResults {
        player {
          ...PlayerWithTournamentElo
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
`, [PlayerBasicFragment, PlayerWithTournamentEloFragment])
