import { Badge, Box, HStack, Text, VStack } from '@chakra-ui/react'
import { LuCrown } from 'react-icons/lu'
import { Avatar } from '../common/Avatar'

type RaceResult = {
  player: {
    id: string
    name: string
    currentTournamentElo: number | null
  }
  position: number
  tournamentEloChange: number | null
}

type RaceResultsDisplayProps = {
  results: RaceResult[]
  trackName?: string
}

const getPositionBadgeColor = (position: number): string => {
  if (position === 1) return 'yellow'
  if (position === 2) return 'gray'
  if (position === 3) return 'orange'
  return 'gray'
}

const formatEloChange = (change: number): string => {
  if (change > 0) return `+${change}`
  return String(change)
}

const getCrownColor = (position: number): string => {
  if (position === 1) return 'yellow.500'
  if (position === 2) return 'gray.500'
  if (position === 3) return 'orange.500'
  return 'gray.500'
}

export const RaceResultsDisplay = ({ results, trackName }: RaceResultsDisplayProps) => {
  const sortedResults = [...results].sort((a, b) => a.position - b.position)

  return (
    <Box p={{ base: 5, md: 6 }} bg="bg.panel" borderRadius="card" borderWidth="1px" borderColor="brand.400" boxShadow="card-hover">
      <VStack gap={{ base: 4, md: 5 }} align="stretch">
        <VStack gap={1} align="start">
          <Text fontSize={{ base: 'md', md: 'lg' }} fontWeight="bold" color="gray.900">
            Race Results
          </Text>
          {trackName && (
            <Text fontSize={{ base: 'sm', md: 'md' }} color="gray.600">
              {trackName}
            </Text>
          )}
        </VStack>

        <VStack gap={{ base: 2, md: 3 }} align="stretch">
          {sortedResults.map((result) => (
            <HStack
              key={result.player.id}
              gap={{ base: 3, md: 4 }}
              justify="space-between"
              p={{ base: 2, md: 3 }}
              borderRadius="button"
              bg={result.position === 1 ? 'yellow.50' : 'white'}
              borderWidth="1px"
              borderColor={result.position === 1 ? 'yellow.200' : 'gray.100'}
            >
              <HStack gap={{ base: 2, md: 3 }} flex={1} minW={0}>
                <Badge
                  colorScheme={getPositionBadgeColor(result.position)}
                  fontSize={{ base: 'lg', md: 'xl' }}
                  px={{ base: 2, md: 3 }}
                  py={1}
                  borderRadius="md"
                  fontWeight="bold"
                  display="flex"
                  alignItems="center"
                  justifyContent="center"
                >
                  {result.position <= 3 ? (
                    <Box color={getCrownColor(result.position)}>
                      <LuCrown size={20} fill="currentColor" />
                    </Box>
                  ) : (
                    result.position
                  )}
                </Badge>

                <Avatar name={result.player.name} size="sm" />

                <Text fontSize={{ base: 'sm', md: 'md' }} fontWeight="medium" truncate>
                  {result.player.name}
                </Text>
              </HStack>

              <Text fontSize={{ base: 'md', md: 'lg' }} fontWeight="bold" color={(result.tournamentEloChange ?? 0) >= 0 ? 'green.600' : 'red.600'} flexShrink={0}>
                {formatEloChange(result.tournamentEloChange ?? 0)}
              </Text>
            </HStack>
          ))}
        </VStack>
      </VStack>
    </Box>
  )
}
