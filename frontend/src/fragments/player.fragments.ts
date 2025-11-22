import { graphql } from 'gql.tada'

export const PlayerBasicFragment = graphql(`
  fragment PlayerBasic on Player {
    id
    name
    avatarFilename
  }
`)

export const PlayerWithTournamentEloFragment = graphql(`
  fragment PlayerWithTournamentElo on Player {
    id
    name
    avatarFilename
    currentTournamentElo
  }
`)
