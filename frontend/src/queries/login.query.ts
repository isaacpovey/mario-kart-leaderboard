import { graphql } from 'gql.tada'

export const loginQuery = graphql(`
    query Login($groupId: ID!, $password: String!) {
        login(groupId: $groupId, password: $password)
    }
`)
