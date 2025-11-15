import { HStack, Text, VStack } from '@chakra-ui/react'

type LeaderboardEntry = {
  playerId: string
  playerName: string
  totalScore: number
  eloRating: number
}

type LeaderboardListProps = {
  entries: LeaderboardEntry[]
}

export const LeaderboardList = ({ entries }: LeaderboardListProps) => {
  if (entries.length === 0) {
    return <Text color="fg.subtle">No players yet</Text>
  }

  return (
    <>
      {entries.map((entry, index) => (
        <HStack
          key={entry.playerId}
          p={4}
          bg={index === 0 ? 'yellow.50' : 'bg.panel'}
          borderRadius="md"
          borderWidth="1px"
          borderColor={index === 0 ? 'yellow.300' : 'border'}
          justify="space-between"
        >
          <HStack gap={4} flex={1} minW={0}>
            <Text fontWeight="bold" fontSize={{ base: 'lg', md: 'xl' }} minW="8">
              #{index + 1}
            </Text>
            <VStack align="start" gap={0} flex={1} minW={0}>
              <Text fontWeight="semibold" fontSize={{ base: 'md', md: 'lg' }} truncate>
                {entry.playerName}
              </Text>
              <Text fontSize="sm" color="fg.subtle">
                ELO: {entry.eloRating}
              </Text>
            </VStack>
          </HStack>
          <Text fontWeight="bold" fontSize={{ base: 'lg', md: 'xl' }} flexShrink={0}>
            {entry.totalScore} pts
          </Text>
        </HStack>
      ))}
    </>
  )
}
