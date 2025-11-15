import { HStack, Text, VStack } from '@chakra-ui/react'

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
  <VStack p={{ base: 3, md: 4 }} bg="bg.panel" borderRadius="md" borderWidth="1px" align="stretch" gap={2}>
    <HStack justify="space-between">
      <Text fontWeight="bold" fontSize={{ base: 'md', md: 'lg' }} truncate>
        {team.name}
      </Text>
      <Text fontWeight="semibold" fontSize={{ base: 'md', md: 'lg' }} flexShrink={0}>
        {team.score} pts
      </Text>
    </HStack>
    <VStack align="stretch" gap={1}>
      {team.players.map((player) => (
        <HStack key={player.id} justify="space-between" gap={2}>
          <Text fontSize={{ base: 'sm', md: 'md' }} truncate>
            {player.name}
          </Text>
          <Text fontSize="sm" color="fg.subtle" flexShrink={0}>
            ELO: {player.eloRating}
          </Text>
        </HStack>
      ))}
    </VStack>
  </VStack>
)
