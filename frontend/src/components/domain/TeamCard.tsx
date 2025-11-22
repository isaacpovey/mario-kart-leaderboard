import { Box, HStack, Text, VStack } from '@chakra-ui/react'
import { LuFlag, LuUsers } from 'react-icons/lu'
import { Avatar } from '../common/Avatar'

type Player = {
  id: string
  name: string
  currentTournamentElo: number | null
}

type Team = {
  id: string
  name: string
  score: number | null
  players: Player[]
}

type PlayerResult = {
  player: {
    id: string
    name: string
    currentTournamentElo: number | null
  }
  tournamentEloChange: number
  tournamentEloFromRaces: number
  tournamentEloFromContributions: number
  teammateContribution: number
}

type TeamCardProps = {
  team: Team
  playerResults?: PlayerResult[]
}

const getPlayerResult = (playerId: string, playerResults?: PlayerResult[]) => playerResults?.find((result) => result.player.id === playerId)

const formatEloChange = (change: number): string => {
  if (change > 0) return `+${change}`
  return String(change)
}

export const TeamCard = ({ team, playerResults }: TeamCardProps) => (
  <Box
    p={{ base: 4, md: 5 }}
    bg="bg.panel"
    borderRadius="card"
    borderWidth="1px"
    borderColor="gray.200"
    boxShadow="card"
    _hover={{ boxShadow: 'card-hover' }}
    transition="all 0.2s"
  >
    <VStack align="stretch" gap={{ base: 3, md: 4 }}>
      <Text fontWeight="bold" fontSize={{ base: 'lg', md: 'xl' }} color="gray.900" pb={{ base: 2, md: 3 }} borderBottomWidth="1px" borderBottomColor="gray.200">
        {team.name}
      </Text>

      <VStack align="stretch" gap={{ base: 2, md: 3 }}>
        {team.players.map((player) => {
          const result = getPlayerResult(player.id, playerResults)
          const ownContribution = result?.tournamentEloFromRaces ?? 0
          const teammateContribution = result?.tournamentEloFromContributions ?? 0
          const totalChange = result?.tournamentEloChange ?? 0

          return (
            <HStack key={player.id} gap={{ base: 3, md: 4 }} justify="space-between">
              <HStack gap={{ base: 2, md: 3 }} flex={1} minW={0}>
                <Avatar name={player.name} size="sm" />
                <VStack align="start" gap={0} flex={1} minW={0}>
                  <Text fontSize={{ base: 'sm', md: 'md' }} fontWeight="medium" truncate>
                    {player.name}
                  </Text>
                  {result && (
                    <HStack gap={2} fontSize={{ base: 'xs', md: 'sm' }}>
                      <Text color="gray.600">{player.currentTournamentElo ?? 1200}</Text>
                      <HStack gap={1} color={ownContribution >= 0 ? 'green.600' : 'red.600'} fontWeight="medium">
                        <LuFlag size={12} />
                        <Text>{formatEloChange(ownContribution)}</Text>
                      </HStack>
                      <HStack gap={1} color={teammateContribution >= 0 ? 'green.600' : 'red.600'} fontWeight="medium">
                        <LuUsers size={12} />
                        <Text>{formatEloChange(teammateContribution)}</Text>
                      </HStack>
                      <Text color={totalChange >= 0 ? 'green.600' : 'red.600'} fontWeight="bold">
                        = {formatEloChange(totalChange)}
                      </Text>
                    </HStack>
                  )}
                  {!result && (
                    <Text fontSize={{ base: 'xs', md: 'sm' }} color="gray.600">
                      {player.currentTournamentElo ?? 1200}
                    </Text>
                  )}
                </VStack>
              </HStack>
            </HStack>
          )
        })}
      </VStack>
    </VStack>
  </Box>
)
