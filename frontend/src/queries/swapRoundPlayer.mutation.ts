import { graphql } from 'gql.tada'

export const swapRoundPlayerMutation = graphql(`
  mutation SwapRoundPlayer($matchId: ID!, $roundNumber: Int!, $currentPlayerId: ID!, $newPlayerId: ID!) {
    swapRoundPlayer(
      matchId: $matchId
      roundNumber: $roundNumber
      currentPlayerId: $currentPlayerId
      newPlayerId: $newPlayerId
    ) {
      id
      rounds {
        roundNumber
        completed
        players {
          id
          name
          avatarFilename
          teamId
        }
      }
    }
  }
`)
