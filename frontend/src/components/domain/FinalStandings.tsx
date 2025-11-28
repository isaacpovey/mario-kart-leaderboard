import { Badge, Box, HStack, Text, VStack } from '@chakra-ui/react'
import { LuCrown } from 'react-icons/lu'
import { Avatar } from '../common/Avatar'

type LeaderboardEntry = {
  playerId: string
  playerName: string
  allTimeEloRating: number
  allTimeElo: number
  avatarFilename?: string | null
}

type FinalStandingsProps = {
  leaderboard: LeaderboardEntry[]
  winnerId?: string | null
}

const getCrownColor = (position: number): string => {
  if (position === 1) return 'yellow.500'
  if (position === 2) return 'gray.500'
  if (position === 3) return 'orange.700'
  return 'gray.500'
}

type StandingCardProps = {
  entry: LeaderboardEntry
  position: number
  isWinner: boolean
}

const StandingCard = ({ entry, position, isWinner }: StandingCardProps) => (
  <Box
    p={{ base: 4, md: 5 }}
    bg="bg.panel"
    borderRadius="card"
    borderWidth={isWinner ? '3px' : '1px'}
    borderColor={isWinner ? 'yellow.400' : 'gray.200'}
    boxShadow="card"
    _hover={{ boxShadow: 'card-hover', transform: 'translateY(-2px)' }}
    transition="all 0.2s"
  >
    <HStack justify="space-between" gap={{ base: 3, md: 4 }}>
      <HStack gap={{ base: 3, md: 4 }} flex={1} minW={0}>
        <Badge
          colorScheme={position === 1 ? 'yellow' : position === 2 ? 'gray' : position === 3 ? 'orange' : 'gray'}
          fontSize={{ base: 'lg', md: 'xl' }}
          px={{ base: 3, md: 4 }}
          py={{ base: 1, md: 2 }}
          borderRadius="md"
          fontWeight="bold"
          display="flex"
          alignItems="center"
          justifyContent="center"
        >
          {position <= 3 ? (
            <Box color={getCrownColor(position)}>
              <LuCrown size={24} fill="currentColor" />
            </Box>
          ) : (
            `#${position}`
          )}
        </Badge>

        <Avatar name={entry.playerName} avatarFilename={entry.avatarFilename} size="md" />

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
        <Text fontSize={{ base: 'xl', md: '2xl' }} fontWeight="bold" color={isWinner ? 'brand.500' : 'gray.900'}>
          {entry.allTimeEloRating}
        </Text>
      </VStack>
    </HStack>
  </Box>
)

export const FinalStandings = ({ leaderboard, winnerId }: FinalStandingsProps) => {
  if (leaderboard.length === 0) {
    return <Text color="fg.subtle">No standings available</Text>
  }

  return (
    <VStack gap={{ base: 3, md: 4 }} align="stretch">
      {leaderboard.map((entry, index) => (
        <StandingCard
          key={entry.playerId}
          entry={entry}
          position={index + 1}
          isWinner={entry.playerId === winnerId}
        />
      ))}
    </VStack>
  )
}
