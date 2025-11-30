import { Box, Button, Dialog, HStack, Portal, Text, VStack } from '@chakra-ui/react'
import { useState } from 'react'
import { Avatar } from './common/Avatar'

type Player = {
  id: string
  name: string
  avatarFilename?: string | null
}

type Team = {
  id: string
  name: string
  players: Player[]
}

type SwapPlayerModalProps = {
  open: boolean
  onOpenChange: (open: boolean) => void
  currentPlayer: Player & { teamId: string }
  roundNumber: number
  teams: Team[]
  roundPlayerIds: string[]
  onSwap: (newPlayerId: string) => Promise<void>
  isSwapping: boolean
  error: string | null
}

export const SwapPlayerModal = (props: SwapPlayerModalProps) => {
  const { open, onOpenChange, currentPlayer, roundNumber, teams, roundPlayerIds, onSwap, isSwapping, error } = props
  const [selectedPlayerId, setSelectedPlayerId] = useState<string | null>(null)

  const currentTeam = teams.find((t) => t.id === currentPlayer.teamId)
  const eligiblePlayers = currentTeam?.players.filter(
    (p) => p.id !== currentPlayer.id && !roundPlayerIds.includes(p.id)
  ) ?? []

  const handleConfirm = async () => {
    if (!selectedPlayerId) return
    await onSwap(selectedPlayerId)
  }

  const handleClose = () => {
    setSelectedPlayerId(null)
    onOpenChange(false)
  }

  return (
    <Dialog.Root open={open} onOpenChange={(details) => {
      if (!details.open) {
        setSelectedPlayerId(null)
      }
      onOpenChange(details.open)
    }}>
      <Portal>
        <Dialog.Backdrop />
        <Dialog.Positioner>
          <Dialog.Content maxW={{ base: '90vw', md: '450px' }}>
            <Dialog.Header>
              <Dialog.Title>Swap Player - Race {roundNumber}</Dialog.Title>
            </Dialog.Header>
            <Dialog.Body>
              <VStack gap={4} align="stretch">
                <Box>
                  <Text fontSize="sm" color="gray.600" mb={2}>Current player:</Text>
                  <HStack gap={2} p={2} bg="gray.50" borderRadius="md">
                    <Avatar name={currentPlayer.name} avatarFilename={currentPlayer.avatarFilename} size="sm" />
                    <Text fontWeight="medium">{currentPlayer.name}</Text>
                  </HStack>
                </Box>

                <Box>
                  <Text fontSize="sm" color="gray.600" mb={2}>
                    Select replacement from {currentTeam?.name ?? 'team'}:
                  </Text>
                  {eligiblePlayers.length === 0 ? (
                    <Box p={3} bg="yellow.50" borderRadius="md" borderWidth="1px" borderColor="yellow.200">
                      <Text color="yellow.700" fontSize="sm">
                        No eligible teammates available. All other team members are already in this race.
                      </Text>
                    </Box>
                  ) : (
                    <VStack gap={2} align="stretch">
                      {eligiblePlayers.map((player) => (
                        <Box
                          key={player.id}
                          p={3}
                          bg={selectedPlayerId === player.id ? 'brand.50' : 'white'}
                          borderRadius="md"
                          borderWidth="2px"
                          borderColor={selectedPlayerId === player.id ? 'brand.400' : 'gray.200'}
                          cursor="pointer"
                          onClick={() => setSelectedPlayerId(player.id)}
                          _hover={{ borderColor: 'brand.300' }}
                          transition="all 0.2s"
                        >
                          <HStack gap={2}>
                            <Avatar name={player.name} avatarFilename={player.avatarFilename} size="sm" />
                            <Text fontWeight="medium">{player.name}</Text>
                          </HStack>
                        </Box>
                      ))}
                    </VStack>
                  )}
                </Box>

                {error && (
                  <Box p={3} bg="red.50" borderRadius="md" borderWidth="1px" borderColor="red.300">
                    <Text color="red.700" fontSize="sm" fontWeight="medium">
                      {error}
                    </Text>
                  </Box>
                )}

                <HStack gap={3} justify="flex-end">
                  <Button variant="outline" onClick={handleClose} disabled={isSwapping}>
                    Cancel
                  </Button>
                  <Button
                    colorScheme="yellow"
                    bg="brand.400"
                    color="gray.900"
                    onClick={handleConfirm}
                    loading={isSwapping}
                    disabled={!selectedPlayerId || eligiblePlayers.length === 0}
                  >
                    Swap Player
                  </Button>
                </HStack>
              </VStack>
            </Dialog.Body>
          </Dialog.Content>
        </Dialog.Positioner>
      </Portal>
    </Dialog.Root>
  )
}
