import { authExchange } from '@urql/exchange-auth'

const AUTH_TOKEN_KEY = 'mario-kart-auth-token'

const getToken = (): string | null => {
  const stored = localStorage.getItem(AUTH_TOKEN_KEY)
  if (!stored) return null
  return JSON.parse(stored)
}

const clearToken = (): void => {
  localStorage.removeItem(AUTH_TOKEN_KEY)
}

export const createAuthExchange = () => {
  return authExchange(async (utils) => {
    return {
      addAuthToOperation: (operation) => {
        const token = getToken()
        if (!token) {
          return operation
        }

        return utils.appendHeaders(operation, {
          Authorization: `Bearer ${token}`,
        })
      },
      didAuthError: (error) => {
        return error.graphQLErrors.some((e) => e.extensions?.code === 'UNAUTHORIZED')
      },
      refreshAuth: async () => {
        clearToken()
      },
    }
  })
}
