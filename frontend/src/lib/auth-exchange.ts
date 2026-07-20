import { authExchange } from '@urql/exchange-auth'
import { getDefaultStore } from 'jotai'
import { tokenAtom } from '../store/auth'

const AUTH_TOKEN_KEY = 'mario-kart-auth-token'

const getToken = (): string | null => {
  const stored = localStorage.getItem(AUTH_TOKEN_KEY)
  if (!stored) {
    return null
  }
  try {
    return JSON.parse(stored)
  } catch {
    return null
  }
}

const clearAuth = (): void => {
  // Clear jotai state so ProtectedRoute redirects; atomWithStorage mirrors to localStorage.
  getDefaultStore().set(tokenAtom, null)
  // Ensure localStorage is cleared even if the jotai write is a no-op (already null).
  localStorage.removeItem(AUTH_TOKEN_KEY)
}

export const isAuthError = (error: { graphQLErrors: readonly { message: string; extensions?: Record<string, unknown> | null }[]; response?: Response }): boolean => {
  if (error.response?.status === 401) {
    return true
  }

  return error.graphQLErrors.some((e) => {
    const code = e.extensions?.code
    if (code === 'UNAUTHORIZED' || code === 'UNAUTHENTICATED') {
      return true
    }
    return e.message === 'Authentication required' || e.message === 'Unauthorized' || e.message.startsWith('Unauthorized:')
  })
}

export const createAuthExchange = () =>
  authExchange(async (utils) => ({
    addAuthToOperation: (operation) => {
      const token = getToken()
      if (!token) {
        return operation
      }

      return utils.appendHeaders(operation, {
        Authorization: `Bearer ${token}`,
      })
    },
    didAuthError: (error) => isAuthError(error),
    refreshAuth: async () => {
      clearAuth()
    },
  }))
