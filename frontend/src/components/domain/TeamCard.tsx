import { Box, HStack, Text, VStack } from '@chakra-ui/react'
import { Avatar } from '../common/Avatar'

type Player = {
  id: string
  name: string
  eloRating: number
}

type Team = {
  id: string
  name: string
  score: number | null
  players: Player[]
}

type TeamCardProps = {
  team: Team
}

export const TeamCard = ({ team }: TeamCardProps) => (
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
      <HStack justify="space-between" gap={4} pb={{ base: 2, md: 3 }} borderBottomWidth="1px" borderBottomColor="gray.200">
        <Text fontWeight="bold" fontSize={{ base: 'lg', md: 'xl' }} color="gray.900" flex={1} truncate>
          {team.name}
        </Text>
        <VStack align="end" gap={0}>
          <Text fontSize={{ base: 'xs', md: 'sm' }} color="gray.600" fontWeight="medium">
            Score
          </Text>
          <Text fontWeight="bold" fontSize={{ base: 'xl', md: '2xl' }} color="brand.600">
            {team.score ?? 0}
          </Text>
        </VStack>
      </HStack>

      <VStack align="stretch" gap={{ base: 2, md: 3 }}>
        {team.players.map((player) => (
          <HStack key={player.id} gap={{ base: 3, md: 4 }} justify="space-between">
            <HStack gap={{ base: 2, md: 3 }} flex={1} minW={0}>
              <Avatar name={player.name} size="sm" />
              <Text fontSize={{ base: 'sm', md: 'md' }} fontWeight="medium" truncate>
                {player.name}
              </Text>
            </HStack>
            <Text fontSize={{ base: 'xs', md: 'sm' }} color="gray.600" flexShrink={0}>
              ELO: {player.eloRating}
            </Text>
          </HStack>
        ))}
      </VStack>
    </VStack>
  </Box>
)
