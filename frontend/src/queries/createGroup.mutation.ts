import { graphql } from 'gql.tada'

export const createGroupMutation = graphql(`
    mutation CreateGroup($name: String!, $password: String!) {
        createGroup(name: $name, password: $password)
    }
`)
