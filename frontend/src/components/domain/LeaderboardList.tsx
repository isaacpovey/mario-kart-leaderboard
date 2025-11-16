import { Badge, Box, HStack, Text, VStack } from '@chakra-ui/react'
import { LuCrown } from 'react-icons/lu'
import { Avatar } from '../common/Avatar'

type LeaderboardEntry = {
  playerId: string
  playerName: string
  totalScore: number
  eloRating: number
  allTimeElo: number
}

type LeaderboardListProps = {
  entries: LeaderboardEntry[]
}

type LeaderboardEntryCardProps = {
  entry: LeaderboardEntry
  index: number
}

const getCrownColor = (position: number): string => {
  if (position === 1) return 'yellow.500'
  if (position === 2) return 'gray.500'
  if (position === 3) return 'orange.500'
  return 'gray.500'
}

const LeaderboardEntryCard = ({ entry, index }: LeaderboardEntryCardProps) => (
  <Box
    key={entry.playerId}
    p={{ base: 4, md: 5 }}
    bg={index === 0 ? 'brand.50' : 'bg.panel'}
    borderRadius="card"
    borderWidth={index === 0 ? '2px' : '1px'}
    borderColor={index === 0 ? 'brand.400' : 'gray.200'}
    boxShadow="card"
    _hover={{ boxShadow: 'card-hover', transform: 'translateY(-2px)' }}
    transition="all 0.2s"
  >
    <HStack justify="space-between" gap={{ base: 3, md: 4 }}>
      <HStack gap={{ base: 3, md: 4 }} flex={1} minW={0}>
        <Badge
          colorScheme={index === 0 ? 'yellow' : index === 1 ? 'gray' : index === 2 ? 'orange' : 'gray'}
          fontSize={{ base: 'lg', md: 'xl' }}
          px={{ base: 3, md: 4 }}
          py={{ base: 1, md: 2 }}
          borderRadius="md"
          fontWeight="bold"
          display="flex"
          alignItems="center"
          justifyContent="center"
        >
          {index < 3 ? (
            <Box color={getCrownColor(index + 1)}>
              <LuCrown size={24} fill="currentColor" />
            </Box>
          ) : (
            `#${index + 1}`
          )}
        </Badge>

        <Avatar name={entry.playerName} size="md" />

        <VStack align="start" gap={0} flex={1} minW={0}>
          <Text fontWeight="bold" fontSize={{ base: 'md', md: 'lg' }} truncate>
            {entry.playerName}
          </Text>
          <Text fontSize={{ base: 'xs', md: 'sm' }} color="gray.600">
            All Time: {entry.allTimeElo}
          </Text>
        </VStack>
      </HStack>

      <VStack align="end" gap={0}>
        <Text fontSize={{ base: 'xs', md: 'sm' }} color="gray.600" fontWeight="medium">
          Score
        </Text>
        <Text fontSize={{ base: 'xl', md: '2xl' }} fontWeight="bold" color={index === 0 ? 'brand.600' : 'gray.900'}>
          {entry.totalScore}
        </Text>
      </VStack>
    </HStack>
  </Box>
)

export const LeaderboardList = ({ entries }: LeaderboardListProps) => {
  if (entries.length === 0) {
    return <Text color="fg.subtle">No players yet</Text>
  }

  return (
    <VStack gap={{ base: 3, md: 4 }} align="stretch">
      {entries.map((entry, index) => (
        <LeaderboardEntryCard key={entry.playerId} entry={entry} index={index} />
      ))}
    </VStack>
  )
}
