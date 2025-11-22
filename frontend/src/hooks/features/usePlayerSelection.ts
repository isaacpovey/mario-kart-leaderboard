import { useAtomValue } from 'jotai'
import { useCallback, useMemo, useState } from 'react'
import { createPlayerMutation } from '@/queries/createPlayer.mutation.ts'
import { playersQueryAtom } from '@/store/queries.ts'
import { useMutationWithRefresh } from '../patterns/useMutationWithRefresh'

type Player = {
  id: string
  name: string
  currentTournamentElo: number | null
}

export const usePlayerSelection = () => {
  const playersResult = useAtomValue(playersQueryAtom)
  const [selectedPlayerIds, setSelectedPlayerIds] = useState<string[]>([])
  const [searchTerm, setSearchTerm] = useState('')
  const [showDropdown, setShowDropdown] = useState(false)

  const { execute: createPlayer, isLoading: isCreatingPlayer, error: createPlayerError } = useMutationWithRefresh(createPlayerMutation)

  const players = (playersResult.data?.players || []) as Player[]
  const selectedPlayers = useMemo(() => players.filter((player: Player) => selectedPlayerIds.includes(player.id)), [players, selectedPlayerIds])

  const filteredPlayers = useMemo(
    () => players.filter((player: Player) => player.name.toLowerCase().includes(searchTerm.toLowerCase()) && !selectedPlayerIds.includes(player.id)),
    [players, searchTerm, selectedPlayerIds]
  )

  const canCreateNewPlayer = useMemo(() => searchTerm.trim() !== '' && !players.some((p: Player) => p.name.toLowerCase() === searchTerm.toLowerCase()), [searchTerm, players])

  const togglePlayer = useCallback((playerId: string) => {
    setSelectedPlayerIds((prev) => (prev.includes(playerId) ? prev.filter((id) => id !== playerId) : [...prev, playerId]))
  }, [])

  const createAndSelectPlayer = useCallback(async () => {
    if (!canCreateNewPlayer || isCreatingPlayer) return null

    const result = await createPlayer({ name: searchTerm.trim() })

    if (result.data?.createPlayer) {
      const newPlayerId = result.data.createPlayer.id
      setSelectedPlayerIds((prev) => [...prev, newPlayerId])
      setSearchTerm('')
      setShowDropdown(false)
      return result.data.createPlayer
    }

    return null
  }, [canCreateNewPlayer, isCreatingPlayer, createPlayer, searchTerm])

  const clearSelection = useCallback(() => {
    setSelectedPlayerIds([])
  }, [])

  const setSelection = useCallback((playerIds: string[]) => {
    setSelectedPlayerIds(playerIds)
  }, [])

  return {
    players,
    selectedPlayerIds,
    selectedPlayers,
    filteredPlayers,
    searchTerm,
    setSearchTerm,
    showDropdown,
    setShowDropdown,
    canCreateNewPlayer,
    isCreatingPlayer,
    createPlayerError,
    togglePlayer,
    createAndSelectPlayer,
    clearSelection,
    setSelection,
  }
}
