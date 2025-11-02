import { Button, Center, Container, Heading, HStack, Spinner, Text, VStack } from '@chakra-ui/react'
import { useEffect } from 'react'
import { useQuery } from 'urql'
import { useAuth } from '../hooks/useAuth'
import { tournamentsQuery } from '../queries/tournaments.query'

const Home = () => {
  const { logout } = useAuth()
  const [result] = useQuery({ query: tournamentsQuery })

  useEffect(() => {
    document.title = 'Mario Kart Leaderboard'
  }, [])

  const { data, fetching, error } = result

  if (fetching) {
    return (
      <Center h="100vh">
        <Spinner size="xl" />
      </Center>
    )
  }

  if (error) {
    return (
      <Container maxW="4xl" py={8}>
        <Text color="red.500">Error loading tournament data: {error.message}</Text>
      </Container>
    )
  }

  const currentTournament = data?.tournaments.reduce(
    (latest, tournament) => (!latest || (tournament.startDate && (!latest.startDate || tournament.startDate > latest.startDate)) ? tournament : latest),
    undefined
  )

  return (
    <Container maxW="4xl" py={8}>
      <VStack gap={6} align="stretch">
        <HStack justify="space-between">
          <Heading size="3xl">Mario Kart Leaderboard</Heading>
          <Button onClick={logout} colorScheme="red">
            Logout
          </Button>
        </HStack>

        {currentTournament ? (
          <>
            <VStack gap={2} align="start">
              <Heading size="xl">Current Tournament</Heading>
              {currentTournament.startDate && <Text color="fg.subtle">Started: {currentTournament.startDate}</Text>}
              {currentTournament.endDate && <Text color="fg.subtle">Ended: {currentTournament.endDate}</Text>}
            </VStack>

            <VStack gap={4} align="stretch">
              <Heading size="lg">Leaderboard</Heading>
              {currentTournament.leaderboard.length > 0 ? (
                currentTournament.leaderboard.map((entry, index) => (
                  <HStack
                    key={entry.playerId}
                    p={4}
                    bg={index === 0 ? 'yellow.50' : 'bg.panel'}
                    borderRadius="md"
                    borderWidth="1px"
                    borderColor={index === 0 ? 'yellow.300' : 'border'}
                    justify="space-between"
                  >
                    <HStack gap={4}>
                      <Text fontWeight="bold" fontSize="xl" minW="8">
                        #{index + 1}
                      </Text>
                      <VStack align="start" gap={0}>
                        <Text fontWeight="semibold" fontSize="lg">
                          {entry.playerName}
                        </Text>
                        <Text fontSize="sm" color="fg.subtle">
                          ELO: {entry.eloRating}
                        </Text>
                      </VStack>
                    </HStack>
                    <Text fontWeight="bold" fontSize="xl">
                      {entry.totalScore} pts
                    </Text>
                  </HStack>
                ))
              ) : (
                <Text color="fg.subtle">No players yet</Text>
              )}
            </VStack>

            <VStack gap={2} align="start">
              <Heading size="lg">Matches</Heading>
              {currentTournament.matches.length > 0 ? (
                currentTournament.matches.map((match) => (
                  <HStack key={match.id} p={4} bg="bg.panel" borderRadius="md" borderWidth="1px" justify="space-between" width="full">
                    <VStack align="start" gap={0}>
                      <Text fontWeight="semibold">Match {match.id}</Text>
                      <Text fontSize="sm" color="fg.subtle">
                        {match.time}
                      </Text>
                    </VStack>
                    <Text fontSize="sm" fontWeight="semibold" color={match.completed ? 'green.500' : 'orange.500'}>
                      {match.completed ? 'Completed' : 'In Progress'}
                    </Text>
                  </HStack>
                ))
              ) : (
                <Text color="fg.subtle">No matches yet</Text>
              )}
            </VStack>
          </>
        ) : (
          <Text color="fg.subtle">No tournament data available</Text>
        )}
      </VStack>
    </Container>
  )
}

export default Home
