import { graphql } from 'gql.tada'

export const completedTournamentsQuery = graphql(`
  query CompletedTournaments($limit: Int!, $offset: Int!) {
    completedTournaments(limit: $limit, offset: $offset) {
      totalCount
      hasMore
      items {
        id
        startDate
        endDate
        winnerId
        winnerName
        winnerAvatarFilename
        participantCount
      }
    }
  }
`)
