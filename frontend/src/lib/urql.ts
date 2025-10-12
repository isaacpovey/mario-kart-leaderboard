import { cacheExchange } from '@urql/exchange-graphcache'
import { createClient, fetchExchange } from 'urql'

export const urqlClient = createClient({
  url: 'http://localhost:8080/graphql',
  exchanges: [
    cacheExchange({
      keys: {
        // Add custom key configurations as needed
      },
    }),
    fetchExchange,
  ],
})
