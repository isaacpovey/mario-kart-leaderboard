import { graphql } from 'gql.tada'

export const createPlayerMutation = graphql(`
  mutation CreatePlayer($name: String!) {
    createPlayer(name: $name) {
      id
      name
      currentTournamentElo
    }
  }
`)
