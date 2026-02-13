import { graphql } from 'gql.tada'

export const createMatchWithRoundsMutation = graphql(`
  mutation CreateMatchWithRounds(
    $tournamentId: ID!
    $playerIds: [ID!]!
    $numRaces: Int!
    $playersPerRace: Int
    $randomTeams: Boolean
  ) {
    createMatchWithRounds(
      tournamentId: $tournamentId
      playerIds: $playerIds
      numRaces: $numRaces
      playersPerRace: $playersPerRace
      randomTeams: $randomTeams
    ) {
      id
      tournamentId
      time
      numOfRounds
      completed
    }
  }
`)
