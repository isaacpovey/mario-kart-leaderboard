import { atomWithQuery } from 'jotai-urql'
import { activeTournamentQuery } from '../queries/activeTournament.query'
import { currentGroupQuery } from '../queries/currentGroup.query'
import { matchQuery } from '../queries/match.query'
import { playerByIdQuery } from '../queries/playerById.query'
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
  getContext: () => ({ requestPolicy: 'cache-and-network' }),
})

export const matchQueryAtom = (matchId: string) =>
  atomWithQuery({
    query: matchQuery,
    getVariables: () => ({ matchId }),
    getContext: () => ({ requestPolicy: 'cache-and-network' }),
  })

export const playerByIdQueryAtom = (playerId: string) =>
  atomWithQuery({
    query: playerByIdQuery,
    getVariables: () => ({ playerId }),
    getContext: () => ({ requestPolicy: 'cache-and-network' }),
  })

export const currentGroupQueryAtom = atomWithQuery({
  query: currentGroupQuery,
  getContext: () => ({ requestPolicy: 'cache-first' }),
})
