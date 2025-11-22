import { atomWithQuery } from 'jotai-urql'
import { activeTournamentQuery } from '../queries/activeTournament.query'
import { currentGroupQuery } from '../queries/currentGroup.query'
import { matchQuery } from '../queries/match.query'
import { playersQuery } from '../queries/players.query'
import { tournamentsQuery } from '../queries/tournaments.query'

export const playersQueryAtom = atomWithQuery({
  query: playersQuery,
  getContext: () => ({ requestPolicy: 'cache-first' }),
})

export const tournamentsQueryAtom = atomWithQuery({
  query: tournamentsQuery,
  getContext: () => ({ requestPolicy: 'cache-first' }),
})

export const activeTournamentQueryAtom = atomWithQuery({
  query: activeTournamentQuery,
  getContext: () => ({ requestPolicy: 'cache-first' }),
})

export const matchQueryAtom = (matchId: string) =>
  atomWithQuery({
    query: matchQuery,
    getVariables: () => ({ matchId }),
    getContext: () => ({ requestPolicy: 'cache-first' }),
  })

export const currentGroupQueryAtom = atomWithQuery({
  query: currentGroupQuery,
  getContext: () => ({ requestPolicy: 'cache-first' }),
})
