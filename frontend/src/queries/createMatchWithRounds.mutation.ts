import { graphql } from 'gql.tada'

export const createMatchWithRoundsMutation = graphql(`
  mutation CreateMatchWithRounds(
    $tournamentId: ID!
    $playerIds: [ID!]!
    $numRaces: Int!
    $playersPerRace: Int
  ) {
    createMatchWithRounds(
      tournamentId: $tournamentId
      playerIds: $playerIds
      numRaces: $numRaces
      playersPerRace: $playersPerRace
    ) {
      id
      tournamentId
      time
      numOfRounds
      completed
    }
  }
`)
