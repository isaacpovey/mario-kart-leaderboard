import { atomWithQuery } from 'jotai-urql'
import { activeTournamentQuery } from '../queries/activeTournament.query'
import { currentGroupQuery } from '../queries/currentGroup.query'
import { lobbyQuery } from '../queries/lobby.query'
import { matchQuery } from '../queries/match.query'
import { playerByIdQuery } from '../queries/playerById.query'
import { playersQuery } from '../queries/players.query'
import { tournamentsQuery } from '../queries/tournaments.query'

export const playersQueryAtom = atomWithQuery({
  getContext: () => ({ requestPolicy: 'cache-first' }),
  query: playersQuery,
})

export const tournamentsQueryAtom = atomWithQuery({
  getContext: () => ({ requestPolicy: 'cache-first' }),
  query: tournamentsQuery,
})

export const activeTournamentQueryAtom = atomWithQuery({
  getContext: () => ({ requestPolicy: 'cache-and-network' }),
  query: activeTournamentQuery,
})

export const matchQueryAtom = (matchId: string) =>
  atomWithQuery({
    getContext: () => ({ requestPolicy: 'cache-and-network' }),
    getVariables: () => ({ matchId }),
    query: matchQuery,
  })

export const playerByIdQueryAtom = (playerId: string) =>
  atomWithQuery({
    getContext: () => ({ requestPolicy: 'cache-and-network' }),
    getVariables: () => ({ playerId }),
    query: playerByIdQuery,
  })

export const currentGroupQueryAtom = atomWithQuery({
  getContext: () => ({ requestPolicy: 'cache-first' }),
  query: currentGroupQuery,
})

export const lobbyQueryAtom = atomWithQuery({
  getContext: () => ({ requestPolicy: 'cache-and-network' }),
  query: lobbyQuery,
})
