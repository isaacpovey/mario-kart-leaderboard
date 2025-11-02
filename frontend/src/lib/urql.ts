import { cacheExchange } from '@urql/exchange-graphcache'
import { atom } from 'jotai'
import { createClient, fetchExchange } from 'urql'
import { createAuthExchange } from './auth-exchange'

export const urqlClient = createClient({
  url: 'http://localhost:8080/graphql',
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
