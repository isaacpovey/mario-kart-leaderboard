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
                allTimeEloRating
                allTimeElo
                avatarFilename
            }
            matches {
                id
                time
                completed
            }
        }
    }
`)
