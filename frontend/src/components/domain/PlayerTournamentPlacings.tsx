import { Badge, Box, HStack, Text, VStack } from '@chakra-ui/react'
import { LuCrown } from 'react-icons/lu'
import { Link } from 'react-router'

type Placing = {
  tournamentId: string
  startDate?: string | null
  endDate?: string | null
  placing: number
  totalPlayers: number
  eloRating: number
}

type PlayerTournamentPlacingsProps = {
  placings: ReadonlyArray<Placing>
}

const formatTournamentLabel = (startDate?: string | null, endDate?: string | null): string => {
  const format = (dateStr: string) => new Date(dateStr).toLocaleDateString('en-US', { month: 'short', year: 'numeric' })

  if (startDate) {
    return format(startDate)
  }
  if (endDate) {
    return format(endDate)
  }
  return 'Tournament'
}

const getPlacingColor = (placing: number): string => {
  if (placing === 1) {
    return 'yellow'
  }
  if (placing === 2) {
    return 'gray'
  }
  if (placing === 3) {
    return 'orange'
  }
  return 'gray'
}

export const PlayerTournamentPlacings = ({ placings }: PlayerTournamentPlacingsProps) => {
  if (placings.length === 0) {
    return <Text color="fg.subtle">No previous tournaments</Text>
  }

  return (
    <VStack gap={{ base: 3, md: 4 }} align="stretch">
      {placings.map((placing) => {
        const label = formatTournamentLabel(placing.startDate, placing.endDate)
        const isWinner = placing.placing === 1

        return (
          <Link key={placing.tournamentId} to={`/tournament/${placing.tournamentId}`} style={{ textDecoration: 'none', width: '100%' }}>
            <Box
              p={{ base: 4, md: 5 }}
              bg="bg.panel"
              borderRadius="card"
              borderWidth="1px"
              borderColor="gray.200"
              boxShadow="card"
              cursor="pointer"
              _hover={{ borderColor: 'brand.400', boxShadow: 'card-hover', transform: 'translateY(-2px)' }}
              transition="all 0.2s"
            >
              <HStack justify="space-between" gap={{ base: 3, md: 4 }}>
                <HStack gap={{ base: 3, md: 4 }} flex={1} minW={0}>
                  <Badge
                    colorScheme={getPlacingColor(placing.placing)}
                    variant={isWinner ? 'solid' : 'subtle'}
                    fontSize={{ base: 'lg', md: 'xl' }}
                    px={3}
                    py={1}
                    borderRadius="md"
                    fontWeight="bold"
                    display="flex"
                    alignItems="center"
                    gap={1}
                  >
                    {isWinner && <LuCrown size={18} fill="currentColor" />}#{placing.placing}
                  </Badge>
                  <VStack align="start" gap={0} minW={0}>
                    <Text fontWeight="bold" fontSize={{ base: 'md', md: 'lg' }} color="gray.900" truncate>
                      {label}
                    </Text>
                    <Text fontSize={{ base: 'xs', md: 'sm' }} color="gray.600">
                      #{placing.placing} of {placing.totalPlayers}
                    </Text>
                  </VStack>
                </HStack>

                <VStack align="end" gap={0}>
                  <Text fontSize={{ base: 'xs', md: 'sm' }} color="gray.600">
                    Tournament ELO
                  </Text>
                  <Text fontSize={{ base: 'md', md: 'lg' }} fontWeight="bold" color="gray.900">
                    {placing.eloRating}
                  </Text>
                </VStack>
              </HStack>
            </Box>
          </Link>
        )
      })}
    </VStack>
  )
}
