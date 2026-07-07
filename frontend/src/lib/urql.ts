import { cacheExchange } from '@urql/exchange-graphcache'
import { createClient as createSSEClient } from 'graphql-sse'
import { createClient, fetchExchange, subscriptionExchange } from 'urql'
import { createAuthExchange } from './auth-exchange'

const graphqlUrl = import.meta.env.VITE_GRAPHQL_URL || 'http://localhost:8080/graphql'

// Get auth token for SSE subscriptions
const getAuthToken = () => {
  const stored = localStorage.getItem('mario-kart-auth-token')
  if (!stored) {
    return null
  }
  try {
    return JSON.parse(stored)
  } catch {
    return null
  }
}

// Create SSE client for subscriptions
const sseClient = createSSEClient({
  credentials: 'include',
  headers: (): Record<string, string> => {
    const token = getAuthToken()
    if (token) {
      return { Authorization: `Bearer ${token}` }
    }
    return {}
  },
  singleConnection: false,
  url: graphqlUrl,
})

export const urqlClient = createClient({
  exchanges: [
    cacheExchange({
      keys: {
        LeaderboardEntry: () => null,
      },
      updates: {
        Mutation: {
          checkInPlayer: (_result, _args, cache, _info) => {
            cache
              .inspectFields('Query')
              .filter((field) => field.fieldName === 'currentGroup')
              .forEach((field) => {
                cache.invalidate('Query', field.fieldName, field.arguments)
              })
          },
          checkOutPlayer: (_result, _args, cache, _info) => {
            cache
              .inspectFields('Query')
              .filter((field) => field.fieldName === 'currentGroup')
              .forEach((field) => {
                cache.invalidate('Query', field.fieldName, field.arguments)
              })
          },
          createMatchWithRounds: (_result, _args, cache, _info) => {
            cache
              .inspectFields('Query')
              .filter((field) => field.fieldName === 'tournaments' || field.fieldName === 'activeTournament')
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
          createTournament: (_result, _args, cache, _info) => {
            cache
              .inspectFields('Query')
              .filter((field) => field.fieldName === 'tournaments' || field.fieldName === 'activeTournament')
              .forEach((field) => {
                cache.invalidate('Query', field.fieldName, field.arguments)
              })
          },
          recordRoundResults: (_result, _args, cache, _info) => {
            // Invalidate all relevant queries to force refetch
            cache
              .inspectFields('Query')
              .filter((field) => field.fieldName === 'tournaments' || field.fieldName === 'activeTournament' || field.fieldName === 'players' || field.fieldName === 'matchById')
              .forEach((field) => {
                cache.invalidate('Query', field.fieldName, field.arguments)
              })
          },
        },
        Subscription: {
          lobbyUpdated: (_result, _args, cache, _info) => {
            cache
              .inspectFields('Query')
              .filter((field) => field.fieldName === 'currentGroup')
              .forEach((field) => {
                cache.invalidate('Query', field.fieldName, field.arguments)
              })
          },
          raceResultsUpdated: (_result, _args, cache, _info) => {
            // Invalidate all relevant queries when subscription data arrives
            cache
              .inspectFields('Query')
              .filter((field) => field.fieldName === 'tournaments' || field.fieldName === 'activeTournament' || field.fieldName === 'players' || field.fieldName === 'matchById')
              .forEach((field) => {
                cache.invalidate('Query', field.fieldName, field.arguments)
              })
          },
        },
      },
    }),
    subscriptionExchange({
      enableAllOperations: false,
      forwardSubscription: (operation) => ({
        subscribe: (sink) => ({
          unsubscribe: sseClient.subscribe(
            {
              ...operation,
              query: operation.query ?? '',
            },
            sink
          ),
        }),
      }),
      isSubscriptionOperation: (op) => op.kind === 'subscription',
    }),
    createAuthExchange(),
    fetchExchange,
  ],
  fetchOptions: {
    credentials: 'include',
    method: 'POST',
  },
  preferGetMethod: false,
  url: graphqlUrl,
})
