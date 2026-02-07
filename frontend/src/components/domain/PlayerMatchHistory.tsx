import { Badge, Box, HStack, Text, VStack } from '@chakra-ui/react'
import { Link } from 'react-router'

type MatchHistoryEntry = {
  matchId: string
  matchTime: string
  position: number
  eloChange: number
  tournamentEloChange: number
}

type PlayerMatchHistoryProps = {
  matches: ReadonlyArray<MatchHistoryEntry>
}

const formatMatchDate = (dateString: string) => {
  const date = new Date(dateString)
  const now = new Date()
  const diffInHours = (now.getTime() - date.getTime()) / (1000 * 60 * 60)

  if (diffInHours < 24) {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  }
  return date.toLocaleDateString([], { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })
}

const getPositionColor = (position: number): string => {
  if (position === 1) return 'yellow'
  if (position === 2) return 'gray'
  if (position === 3) return 'orange'
  return 'gray'
}

const formatEloChange = (change: number): string => (change >= 0 ? `+${change}` : `${change}`)

const EloChangeDisplay = ({ label, change }: { label: string; change: number }) => (
  <VStack align="end" gap={0}>
    <Text fontSize={{ base: 'xs', md: 'sm' }} color="gray.600">
      {label}
    </Text>
    <Text fontSize={{ base: 'md', md: 'lg' }} fontWeight="bold" color={change >= 0 ? 'green.600' : 'red.600'}>
      {formatEloChange(change)}
    </Text>
  </VStack>
)

export const PlayerMatchHistory = ({ matches }: PlayerMatchHistoryProps) => {
  if (matches.length === 0) {
    return <Text color="fg.subtle">No matches played yet</Text>
  }

  return (
    <VStack gap={{ base: 3, md: 4 }} align="stretch">
      {matches.map((match) => (
        <Link key={match.matchId} to={`/match/${match.matchId}`} style={{ width: '100%', textDecoration: 'none' }}>
          <Box
            p={{ base: 4, md: 5 }}
            bg="bg.panel"
            borderRadius="card"
            borderWidth="1px"
            borderColor="gray.200"
            boxShadow="card"
            cursor="pointer"
            _hover={{ boxShadow: 'card-hover', transform: 'translateY(-2px)', borderColor: 'brand.400' }}
            transition="all 0.2s"
          >
            <HStack justify="space-between" gap={{ base: 3, md: 4 }}>
              <HStack gap={{ base: 3, md: 4 }} flex={1}>
                <Badge colorScheme={getPositionColor(match.position)} fontSize={{ base: 'lg', md: 'xl' }} px={3} py={1} borderRadius="md" fontWeight="bold">
                  #{match.position}
                </Badge>
                <VStack align="start" gap={0}>
                  <Text fontWeight="bold" fontSize={{ base: 'md', md: 'lg' }} color="gray.900">
                    {formatMatchDate(match.matchTime)}
                  </Text>
                  <Text fontSize={{ base: 'xs', md: 'sm' }} color="gray.600">
                    {new Date(match.matchTime).toLocaleDateString()}
                  </Text>
                </VStack>
              </HStack>

              <HStack gap={{ base: 4, md: 6 }}>
                <EloChangeDisplay label="Tournament" change={match.tournamentEloChange} />
                <EloChangeDisplay label="All Time" change={match.eloChange} />
              </HStack>
            </HStack>
          </Box>
        </Link>
      ))}
    </VStack>
  )
}
