import { Button, Center, Container, Heading, HStack, Spinner, Text, VStack } from '@chakra-ui/react'
import { useEffect, useState } from 'react'
import { Link } from 'react-router'
import { useQuery } from 'urql'
import { CreateTournamentModal } from '../components/CreateTournamentModal'
import { CreateMatchModal } from '../components/CreateMatchModal'
import { useAuth } from '../hooks/useAuth'
import { tournamentsQuery } from '../queries/tournaments.query'

const Home = () => {
  const { logout } = useAuth()
  const [result] = useQuery({ query: tournamentsQuery })
  const [isModalOpen, setIsModalOpen] = useState(false)
  const [isMatchModalOpen, setIsMatchModalOpen] = useState(false)

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

  const currentTournament = data?.tournaments
    .filter((tournament) => tournament.startDate)
    .sort((a, b) => (b.startDate && a.startDate ? b.startDate.localeCompare(a.startDate) : 0))[0]

  return (
    <Container maxW="4xl" py={8}>
      <VStack gap={6} align="stretch">
        <HStack justify="space-between">
          <Heading size="3xl">Mario Kart Leaderboard</Heading>
          <HStack gap={2}>
            <Button onClick={() => setIsModalOpen(true)} colorScheme="blue">
              Create Tournament
            </Button>
            {currentTournament && (
              <Button onClick={() => setIsMatchModalOpen(true)} colorScheme="green">
                New Match
              </Button>
            )}
            <Button onClick={logout} colorScheme="red">
              Logout
            </Button>
          </HStack>
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
                  <Link key={match.id} to={`/match/${match.id}`} style={{ width: '100%' }}>
                    <HStack p={4} bg="bg.panel" borderRadius="md" borderWidth="1px" justify="space-between" width="full" cursor="pointer" _hover={{ bg: 'bg.subtle' }}>
                      <VStack align="start" gap={0}>
                        <Text fontWeight="semibold">{new Date(match.time).toLocaleString()}</Text>
                      </VStack>
                      <Text fontSize="sm" fontWeight="semibold" color={match.completed ? 'green.500' : 'orange.500'}>
                        {match.completed ? 'Completed' : 'In Progress'}
                      </Text>
                    </HStack>
                  </Link>
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
      <CreateTournamentModal open={isModalOpen} onOpenChange={setIsModalOpen} />
      {currentTournament && (
        <CreateMatchModal
          open={isMatchModalOpen}
          onOpenChange={setIsMatchModalOpen}
          tournamentId={currentTournament.id}
        />
      )}
    </Container>
  )
}

export default Home
