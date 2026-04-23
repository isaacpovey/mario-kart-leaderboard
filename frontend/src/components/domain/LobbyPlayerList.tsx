import { Box, Button, Heading, HStack, Text, VStack } from '@chakra-ui/react'
import { useMemo } from 'react'
import { useLobby } from '../../hooks/features/useLobby'

type Player = {
  id: string
  name: string
  avatarFilename?: string | null
  currentTournamentElo: number | null
}

type LobbyPlayerListProps = {
  allPlayers: Player[]
  checkedInPlayers: Player[]
  onAddNewPlayer: () => void
}

export const LobbyPlayerList = (props: LobbyPlayerListProps) => {
  const { allPlayers, checkedInPlayers, onAddNewPlayer } = props
  const checkedInPlayerIds = useMemo(() => checkedInPlayers.map((p) => p.id), [checkedInPlayers])
  const { toggle, isLoading } = useLobby(checkedInPlayerIds)

  const others = useMemo(
    () => allPlayers.filter((p) => !checkedInPlayerIds.includes(p.id)).sort((a, b) => a.name.localeCompare(b.name)),
    [allPlayers, checkedInPlayerIds]
  )

  return (
    <VStack align="stretch" gap={4}>
      <Box>
        <Heading size="sm" mb={2}>
          In the Lobby ({checkedInPlayers.length})
        </Heading>
        {checkedInPlayers.length === 0 ? (
          <Text color="gray.500" fontSize="sm">
            No one checked in yet.
          </Text>
        ) : (
          <VStack align="stretch" gap={1}>
            {checkedInPlayers.map((p) => (
              <HStack
                key={p.id}
                as="button"
                onClick={() => toggle(p.id)}
                justify="space-between"
                px={3}
                py={2}
                borderWidth="1px"
                borderRadius="md"
                borderColor="green.200"
                bg="green.50"
                opacity={isLoading ? 0.6 : 1}
                _hover={{ bg: 'green.100' }}
              >
                <Text fontWeight="medium">{p.name}</Text>
                <Text color="green.600" fontWeight="bold">
                  ✓
                </Text>
              </HStack>
            ))}
          </VStack>
        )}
      </Box>

      <Box>
        <Heading size="sm" mb={2}>
          Other players
        </Heading>
        {others.length === 0 ? (
          <Text color="gray.500" fontSize="sm">
            Everyone's in the lobby.
          </Text>
        ) : (
          <VStack align="stretch" gap={1}>
            {others.map((p) => (
              <HStack
                key={p.id}
                as="button"
                onClick={() => toggle(p.id)}
                justify="space-between"
                px={3}
                py={2}
                borderWidth="1px"
                borderRadius="md"
                borderColor="gray.200"
                opacity={isLoading ? 0.6 : 1}
                _hover={{ bg: 'gray.50' }}
              >
                <Text>{p.name}</Text>
                <Text color="gray.400">○</Text>
              </HStack>
            ))}
          </VStack>
        )}
        <Button variant="ghost" mt={2} onClick={onAddNewPlayer} size="sm">
          + Add new player
        </Button>
      </Box>
    </VStack>
  )
}
