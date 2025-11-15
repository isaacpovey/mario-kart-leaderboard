import { atomWithQuery } from 'jotai-urql'
import { currentGroupQuery } from '../queries/currentGroup.query'
import { matchQuery } from '../queries/match.query'
import { playersQuery } from '../queries/players.query'
import { tournamentsQuery } from '../queries/tournaments.query'

export const playersQueryAtom = atomWithQuery({
  query: playersQuery,
  getContext: () => ({ requestPolicy: 'cache-and-network' }),
})

export const tournamentsQueryAtom = atomWithQuery({
  query: tournamentsQuery,
  getContext: () => ({ requestPolicy: 'cache-and-network' }),
})

export const matchQueryAtom = (matchId: string) =>
  atomWithQuery({
    query: matchQuery,
    getVariables: () => ({ matchId }),
    getContext: () => ({ requestPolicy: 'cache-and-network' }),
  })

export const currentGroupQueryAtom = atomWithQuery({
  query: currentGroupQuery,
  getContext: () => ({ requestPolicy: 'cache-and-network' }),
})
