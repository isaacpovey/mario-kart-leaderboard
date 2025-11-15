import { atom } from 'jotai'

export const createTournamentModalOpenAtom = atom(false)
export const createMatchModalOpenAtom = atom(false)
export const selectedRoundAtom = atom<number | null>(null)
