import { Box, Button, Drawer, HStack, Input, Portal, Text, VStack } from '@chakra-ui/react'
import { useMemo, useState } from 'react'
import { Avatar } from './Avatar'

type Player = {
  id: string
  name: string
  currentTournamentElo: number | null
  avatarFilename?: string | null
}

type PlayerSelectorDrawerProps = {
  open: boolean
  onOpenChange: (open: boolean) => void
  allPlayers: Player[]
  selectedPlayerIds: string[]
  onTogglePlayer: (playerId: string) => void
  isCreatingPlayer: boolean
  onCreatePlayer: (name: string) => Promise<unknown>
}

export const PlayerSelectorDrawer = (props: PlayerSelectorDrawerProps) => {
  const { open, onOpenChange, allPlayers, selectedPlayerIds, onTogglePlayer, isCreatingPlayer, onCreatePlayer } = props

  const [searchTerm, setSearchTerm] = useState('')

  const { selectedPlayers, availablePlayers, canCreate } = useMemo(() => {
    const lowercaseSearch = searchTerm.toLowerCase()
    const trimmedSearch = searchTerm.trim()
    const filteredPlayers = trimmedSearch ? allPlayers.filter((p) => p.name.toLowerCase().includes(lowercaseSearch)) : allPlayers

    const selected = filteredPlayers.filter((p) => selectedPlayerIds.includes(p.id))
    const available = filteredPlayers.filter((p) => !selectedPlayerIds.includes(p.id))

    const exactMatchExists = allPlayers.some((p) => p.name.toLowerCase() === lowercaseSearch)
    const showCreateOption = trimmedSearch !== '' && !exactMatchExists

    return {
      selectedPlayers: selected,
      availablePlayers: available,
      canCreate: showCreateOption,
    }
  }, [allPlayers, selectedPlayerIds, searchTerm])

  const handleCreatePlayer = async () => {
    await onCreatePlayer(searchTerm.trim())
    setSearchTerm('')
  }

  const handleClose = () => {
    setSearchTerm('')
    onOpenChange(false)
  }

  return (
    <Drawer.Root open={open} onOpenChange={(details) => onOpenChange(details.open)} placement="bottom" size="full">
      <Portal>
        <Drawer.Backdrop />
        <Drawer.Positioner>
          <Drawer.Content height="100dvh">
            <Drawer.Header borderBottomWidth="1px">
              <VStack align="stretch" gap={3} width="100%">
                <HStack justify="space-between">
                  <Drawer.Title>Select Players</Drawer.Title>
                  <Drawer.CloseTrigger asChild>
                    <Button variant="ghost" size="sm">
                      ✕
                    </Button>
                  </Drawer.CloseTrigger>
                </HStack>
                <Input placeholder="Search players..." value={searchTerm} onChange={(e) => setSearchTerm(e.target.value)} size="lg" />
              </VStack>
            </Drawer.Header>

            <Drawer.Body overflowY="auto" p={0}>
              <VStack align="stretch" gap={0}>
                {canCreate && (
                  <Box
                    px={4}
                    py={3}
                    borderBottomWidth="1px"
                    cursor={isCreatingPlayer ? 'not-allowed' : 'pointer'}
                    onClick={isCreatingPlayer ? undefined : handleCreatePlayer}
                    bg="blue.50"
                    _hover={{ bg: isCreatingPlayer ? undefined : 'blue.100' }}
                  >
                    <HStack gap={3}>
                      <Box width="32px" height="32px" borderRadius="full" bg="blue.500" color="white" display="flex" alignItems="center" justifyContent="center" fontWeight="bold">
                        +
                      </Box>
                      <Text color="blue.700" fontWeight="semibold">
                        {isCreatingPlayer ? 'Creating...' : `Create "${searchTerm}"`}
                      </Text>
                    </HStack>
                  </Box>
                )}

                {selectedPlayers.length > 0 && (
                  <Box>
                    <Box px={4} py={2} bg="gray.50" position="sticky" top={0} zIndex={1}>
                      <Text fontSize="sm" fontWeight="semibold" color="gray.600">
                        Selected ({selectedPlayers.length})
                      </Text>
                    </Box>
                    {selectedPlayers.map((player) => (
                      <PlayerRow key={player.id} player={player} isSelected={true} onToggle={() => onTogglePlayer(player.id)} />
                    ))}
                  </Box>
                )}

                <Box>
                  <Box px={4} py={2} bg="gray.50" position="sticky" top={0} zIndex={1}>
                    <Text fontSize="sm" fontWeight="semibold" color="gray.600">
                      Available Players ({availablePlayers.length})
                    </Text>
                  </Box>
                  {availablePlayers.map((player) => (
                    <PlayerRow key={player.id} player={player} isSelected={false} onToggle={() => onTogglePlayer(player.id)} />
                  ))}

                  {availablePlayers.length === 0 && !canCreate && searchTerm.trim() && (
                    <Box px={4} py={4}>
                      <Text color="gray.500">No players found</Text>
                    </Box>
                  )}
                </Box>
              </VStack>
            </Drawer.Body>

            <Drawer.Footer borderTopWidth="1px">
              <Button colorScheme="yellow" bg="brand.400" color="gray.900" width="100%" size="lg" onClick={handleClose} _hover={{ bg: 'brand.500' }}>
                Done ({selectedPlayerIds.length} selected)
              </Button>
            </Drawer.Footer>
          </Drawer.Content>
        </Drawer.Positioner>
      </Portal>
    </Drawer.Root>
  )
}

const PlayerRow = (props: { player: Player; isSelected: boolean; onToggle: () => void }) => {
  const { player, isSelected, onToggle } = props

  return (
    <Box
      px={4}
      py={3}
      borderBottomWidth="1px"
      cursor="pointer"
      onClick={onToggle}
      bg={isSelected ? 'brand.50' : 'white'}
      _hover={{ bg: isSelected ? 'brand.100' : 'gray.50' }}
      transition="background 0.15s"
    >
      <HStack gap={3}>
        <Avatar name={player.name} avatarFilename={player.avatarFilename} size="sm" />
        <VStack align="start" gap={0} flex={1}>
          <Text fontWeight="medium">{player.name}</Text>
          <Text fontSize="sm" color="gray.500">
            ELO: {player.currentTournamentElo ?? 1200}
          </Text>
        </VStack>
        {isSelected && (
          <Box color="brand.500" fontWeight="bold">
            ✓
          </Box>
        )}
      </HStack>
    </Box>
  )
}
