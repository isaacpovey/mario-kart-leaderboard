import { atom } from 'jotai'
import { atomWithStorage } from 'jotai/utils'

const AUTH_TOKEN_KEY = 'mario-kart-auth-token'

// Read localStorage on first access so ProtectedRoute sees the real auth state
// Before the first paint (avoids treating a stored session as logged-out).
export const tokenAtom = atomWithStorage<string | null>(AUTH_TOKEN_KEY, null, undefined, { getOnInit: true })

export const isAuthenticatedAtom = atom((get) => {
  const token = get(tokenAtom)
  return token !== null && token !== ''
})
