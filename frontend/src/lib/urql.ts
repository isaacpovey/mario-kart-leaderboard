import { cacheExchange } from '@urql/exchange-graphcache'
import { atom } from 'jotai'
import { createClient, fetchExchange } from 'urql'
import { createAuthExchange } from './auth-exchange'

const graphqlUrl = import.meta.env.VITE_GRAPHQL_URL || 'http://localhost:8080/graphql'

export const urqlClient = createClient({
  url: graphqlUrl,
  preferGetMethod: false,
  fetchOptions: {
    method: 'POST',
  },
  exchanges: [
    cacheExchange({
      keys: {
        // Add custom key configurations as needed
      },
    }),
    createAuthExchange(),
    fetchExchange,
  ],
})

export const clientAtom = atom(urqlClient)
