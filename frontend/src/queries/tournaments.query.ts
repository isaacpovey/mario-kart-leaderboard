import { graphql } from 'gql.tada'

export const tournamentsQuery = graphql(`
    query Tournaments {
        tournaments {
            id
            startDate
            endDate
            winnerId
            leaderboard {
                playerId
                playerName
                totalScore
                eloRating
            }
            matches {
                id
                time
                completed
            }
        }
    }
`)
