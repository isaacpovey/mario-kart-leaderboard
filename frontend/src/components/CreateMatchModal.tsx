import { Box, Button, Checkbox, Dialog, Field, HStack, Input, Portal, Tag, Text, VStack } from '@chakra-ui/react'
import { useEffect, useMemo, useRef, useState } from 'react'
import { useNavigate } from 'react-router'
import { useClient, useMutation, useQuery } from 'urql'
import { createMatchWithRoundsMutation } from '../queries/createMatchWithRounds.mutation'
import { createPlayerMutation } from '../queries/createPlayer.mutation'
import { playersQuery } from '../queries/players.query'
import { tournamentsQuery } from '../queries/tournaments.query'

const DEFAULT_NUM_RACES = 6
const DEFAULT_PLAYERS_PER_RACE = 4
const FALLBACK_NUM_RACES = 4
const FALLBACK_PLAYERS_PER_RACE = 4

export const CreateMatchModal = (dependencies: { open: boolean; onOpenChange: (open: boolean) => void; tournamentId: string }) => {
  const { open, onOpenChange, tournamentId } = dependencies
  const navigate = useNavigate()
  const [form, setForm] = useState({
    numRaces: String(DEFAULT_NUM_RACES),
    playersPerRace: String(DEFAULT_PLAYERS_PER_RACE),
  })
  const [selectedPlayerIds, setSelectedPlayerIds] = useState<string[]>([])
  const [searchTerm, setSearchTerm] = useState('')
  const [showDropdown, setShowDropdown] = useState(false)
  const [error, setError] = useState('')
  const [isCreatingPlayer, setIsCreatingPlayer] = useState(false)
  const [isCreatingMatch, setIsCreatingMatch] = useState(false)
  const blurTimeoutRef = useRef<number | null>(null)
  const dropdownRef = useRef<HTMLDivElement>(null)

  const [playersResult] = useQuery({ query: playersQuery })
  const [, executeCreateMatch] = useMutation(createMatchWithRoundsMutation)
  const [, executeCreatePlayer] = useMutation(createPlayerMutation)
  const client = useClient()

  const players = playersResult.data?.players || []
  const selectedPlayers = players.filter((player) => selectedPlayerIds.includes(player.id))

  const filteredPlayers = useMemo(
    () => players.filter((player) => player.name.toLowerCase().includes(searchTerm.toLowerCase()) && !selectedPlayerIds.includes(player.id)),
    [players, searchTerm, selectedPlayerIds]
  )

  const canCreateNewPlayer = searchTerm.trim() !== '' && !players.some((p) => p.name.toLowerCase() === searchTerm.toLowerCase())

  const totalSlots = (Number.parseInt(form.numRaces, 10) || FALLBACK_NUM_RACES) * (Number.parseInt(form.playersPerRace, 10) || FALLBACK_PLAYERS_PER_RACE)
  const selectedCount = selectedPlayerIds.length
  const isValidAllocation = selectedCount > 0 && totalSlots >= selectedCount

  const validationMessage = useMemo(() => {
    if (selectedCount === 0) return 'Select at least one player'
    if (totalSlots < selectedCount) {
      return `Total slots (${totalSlots}) must be at least equal to player count (${selectedCount})`
    }
    return ''
  }, [totalSlots, selectedCount])

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setShowDropdown(false)
      }
    }

    if (showDropdown) {
      document.addEventListener('mousedown', handleClickOutside)
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside)
    }
  }, [showDropdown])

  const handleTogglePlayer = (playerId: string) => {
    setSelectedPlayerIds((prev) => (prev.includes(playerId) ? prev.filter((id) => id !== playerId) : [...prev, playerId]))
  }

  const handleCreateAndSelectPlayer = async () => {
    if (!canCreateNewPlayer || isCreatingPlayer) return

    setIsCreatingPlayer(true)
    const result = await executeCreatePlayer({ name: searchTerm.trim() })
    setIsCreatingPlayer(false)

    if (result.error) {
      setError(result.error.message)
      return
    }

    if (result.data?.createPlayer) {
      setSelectedPlayerIds((prev) => [...prev, result.data?.createPlayer.id ?? ''])
      setSearchTerm('')
      setShowDropdown(false)
    }
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')

    if (!isValidAllocation) {
      setError(validationMessage)
      return
    }

    setIsCreatingMatch(true)
    const result = await executeCreateMatch({
      tournamentId,
      playerIds: selectedPlayerIds,
      numRaces: Number.parseInt(form.numRaces, 10) || FALLBACK_NUM_RACES,
      playersPerRace: Number.parseInt(form.playersPerRace, 10) || FALLBACK_PLAYERS_PER_RACE,
    })
    setIsCreatingMatch(false)

    if (result.error) {
      setError(result.error.message)
      return
    }

    if (result.data?.createMatchWithRounds) {
      const matchId = result.data.createMatchWithRounds.id
      setForm({ numRaces: String(DEFAULT_NUM_RACES), playersPerRace: String(DEFAULT_PLAYERS_PER_RACE) })
      setSelectedPlayerIds([])
      setSearchTerm('')
      onOpenChange(false)

      await client.query(tournamentsQuery, {}, { requestPolicy: 'network-only' }).toPromise()
      navigate(`/match/${matchId}`)
    }
  }

  const handleClose = () => {
    setForm({ numRaces: String(DEFAULT_NUM_RACES), playersPerRace: String(DEFAULT_PLAYERS_PER_RACE) })
    setSelectedPlayerIds([])
    setSearchTerm('')
    setError('')
    onOpenChange(false)
  }

  return (
    <Dialog.Root open={open} onOpenChange={(details) => onOpenChange(details.open)}>
      <Portal>
        <Dialog.Backdrop />
        <Dialog.Positioner>
          <Dialog.Content>
            <Dialog.Header>
              <Dialog.Title>Create New Match</Dialog.Title>
            </Dialog.Header>
            <Dialog.Body>
              <form onSubmit={handleSubmit}>
                <VStack gap={4} align="stretch">
                  <Field.Root>
                    <Field.Label>Players</Field.Label>
                    <Box position="relative" ref={dropdownRef}>
                      <Input
                        placeholder="Search or type to create player..."
                        value={searchTerm}
                        onChange={(e) => {
                          setSearchTerm(e.target.value)
                          setShowDropdown(true)
                        }}
                        onFocus={() => setShowDropdown(true)}
                        onBlur={() => {
                          blurTimeoutRef.current = window.setTimeout(() => setShowDropdown(false), 300)
                        }}
                        onKeyDown={(e) => {
                          if (e.key === 'Enter') {
                            e.preventDefault()
                            if (filteredPlayers.length > 0) {
                              handleTogglePlayer(filteredPlayers[0].id)
                              setSearchTerm('')
                              setShowDropdown(false)
                            } else if (canCreateNewPlayer) {
                              handleCreateAndSelectPlayer()
                            }
                          }
                        }}
                      />
                      {showDropdown && searchTerm.trim() && (filteredPlayers.length > 0 || canCreateNewPlayer) && (
                        <Box position="absolute" zIndex={10} bg="bg.panel" borderWidth="1px" borderRadius="md" mt={1} maxH="200px" overflowY="auto" width="100%">
                        {filteredPlayers.map((player) => (
                          <Box
                            key={player.id}
                            p={2}
                            cursor="pointer"
                            _hover={{ bg: 'bg.subtle' }}
                            onClick={() => {
                              if (blurTimeoutRef.current) {
                                clearTimeout(blurTimeoutRef.current)
                              }
                              handleTogglePlayer(player.id)
                              setSearchTerm('')
                              setShowDropdown(false)
                            }}
                          >
                            <Checkbox.Root checked={selectedPlayerIds.includes(player.id)}>
                              <Checkbox.HiddenInput />
                              <Checkbox.Control />
                              <Checkbox.Label>
                                {player.name} (ELO: {player.eloRating})
                              </Checkbox.Label>
                            </Checkbox.Root>
                          </Box>
                        ))}
                        {canCreateNewPlayer && (
                          <Box
                            p={2}
                            cursor={isCreatingPlayer ? 'not-allowed' : 'pointer'}
                            _hover={{ bg: isCreatingPlayer ? undefined : 'bg.subtle' }}
                            onClick={() => {
                              if (blurTimeoutRef.current) {
                                clearTimeout(blurTimeoutRef.current)
                              }
                              handleCreateAndSelectPlayer()
                            }}
                            borderTopWidth={filteredPlayers.length > 0 ? '1px' : '0'}
                          >
                            <Text color="blue.500" fontWeight="semibold">
                              {isCreatingPlayer ? 'Creating...' : `Create "${searchTerm}"`}
                            </Text>
                          </Box>
                        )}
                        {filteredPlayers.length === 0 && !canCreateNewPlayer && (
                          <Box p={2}>
                            <Text color="fg.subtle">No players found</Text>
                          </Box>
                        )}
                      </Box>
                      )}
                    </Box>
                    {selectedPlayers.length > 0 && (
                      <HStack mt={2} flexWrap="wrap" gap={2}>
                        {selectedPlayers.map((player) => (
                          <Tag.Root key={player.id} size="sm">
                            <Tag.Label>{player.name}</Tag.Label>
                            <Tag.CloseTrigger onClick={() => handleTogglePlayer(player.id)} />
                          </Tag.Root>
                        ))}
                      </HStack>
                    )}
                  </Field.Root>

                  <Field.Root>
                    <Field.Label>Number of Races</Field.Label>
                    <Input type="number" min={1} max={20} value={form.numRaces} onChange={(e) => setForm((prev) => ({ ...prev, numRaces: e.target.value }))} disabled={isCreatingMatch} />
                  </Field.Root>

                  <Field.Root>
                    <Field.Label>Players per Race</Field.Label>
                    <Input type="number" min={2} max={12} value={form.playersPerRace} onChange={(e) => setForm((prev) => ({ ...prev, playersPerRace: e.target.value }))} disabled={isCreatingMatch} />
                  </Field.Root>

                  {validationMessage && (
                    <Text color="orange.500" fontSize="sm">
                      {validationMessage}
                    </Text>
                  )}

                  {error && (
                    <Text color="red.500" fontSize="sm">
                      {error}
                    </Text>
                  )}

                  <VStack gap={2}>
                    <Button type="submit" colorScheme="blue" width="full" disabled={!isValidAllocation} loading={isCreatingMatch}>
                      {isCreatingMatch ? 'Creating...' : 'Create Match'}
                    </Button>
                    <Button type="button" variant="outline" width="full" onClick={handleClose} disabled={isCreatingMatch}>
                      Cancel
                    </Button>
                  </VStack>
                </VStack>
              </form>
            </Dialog.Body>
          </Dialog.Content>
        </Dialog.Positioner>
      </Portal>
    </Dialog.Root>
  )
}
