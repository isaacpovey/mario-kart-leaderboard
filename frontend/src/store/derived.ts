import { atom } from 'jotai'
import { tournamentsQueryAtom } from './queries'

export const currentTournamentAtom = atom(async (get) => {
  const tournamentsResult = await get(tournamentsQueryAtom)

  if (!tournamentsResult?.data?.tournaments) {
    return null
  }

  const tournaments = tournamentsResult.data.tournaments
    // biome-ignore lint/suspicious/noDoubleEquals: != is fine for undefined as this will only check !== undefined and !== null
    .filter((tournament: { startDate?: string | null }) => tournament.startDate != undefined)
    .sort((a: { startDate?: string | null }, b: { startDate?: string | null }) => {
      if (!a.startDate || !b.startDate) return 0
      return b.startDate.localeCompare(a.startDate)
    })

  return tournaments[0] || null
})

export const sortedPlayersAtom = atom(async (get) => {
  const currentTournament = await get(currentTournamentAtom)

  if (!currentTournament?.leaderboard) {
    return []
  }

  return currentTournament.leaderboard.slice().sort((a: { totalScore: number }, b: { totalScore: number }) => b.totalScore - a.totalScore)
})
