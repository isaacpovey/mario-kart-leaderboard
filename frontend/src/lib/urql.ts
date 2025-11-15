import { cacheExchange } from '@urql/exchange-graphcache'
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
        LeaderboardEntry: () => null,
      },
      updates: {
        Mutation: {
          recordRoundResults: (_result, _args, cache, _info) => {
            // Invalidate all relevant queries to force refetch
            cache
              .inspectFields('Query')
              .filter((field) => field.fieldName === 'tournaments' || field.fieldName === 'players' || field.fieldName === 'matchById')
              .forEach((field) => {
                cache.invalidate('Query', field.fieldName, field.arguments)
              })
          },
          createMatchWithRounds: (_result, _args, cache, _info) => {
            cache
              .inspectFields('Query')
              .filter((field) => field.fieldName === 'tournaments')
              .forEach((field) => {
                cache.invalidate('Query', field.fieldName, field.arguments)
              })
          },
          createTournament: (_result, _args, cache, _info) => {
            cache
              .inspectFields('Query')
              .filter((field) => field.fieldName === 'tournaments')
              .forEach((field) => {
                cache.invalidate('Query', field.fieldName, field.arguments)
              })
          },
          createPlayer: (_result, _args, cache, _info) => {
            cache
              .inspectFields('Query')
              .filter((field) => field.fieldName === 'players')
              .forEach((field) => {
                cache.invalidate('Query', field.fieldName, field.arguments)
              })
          },
        },
      },
    }),
    createAuthExchange(),
    fetchExchange,
  ],
})
