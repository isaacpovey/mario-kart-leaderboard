import { atom } from 'jotai'
import { atomWithStorage } from 'jotai/utils'

const AUTH_TOKEN_KEY = 'mario-kart-auth-token'

export const tokenAtom = atomWithStorage<string | null>(AUTH_TOKEN_KEY, null)

export const isAuthenticatedAtom = atom((get) => {
  const token = get(tokenAtom)
  return token !== null && token !== ''
})
