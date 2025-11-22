import { graphql } from 'gql.tada'

export const currentGroupQuery = graphql(`
    query CurrentGroup {
        currentGroup {
            id
            name
            players {
                id
                name
                currentTournamentElo
                avatarFilename
            }
        }
    }
`)
